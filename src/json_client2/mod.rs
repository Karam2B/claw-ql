#![allow(unused)]
#![warn(unused_must_use)]

use serde::{Deserialize, Serialize};
use sqlx::{ColumnIndex, Database, Decode, Executor, Pool, Sqlite, Type};
use std::{collections::HashMap, sync::Arc};

use crate::{
    DatabaseExt,
    json_client::{json_collection::JsonCollection, json_link::JsonLink},
    operations::{LinkedOutput, Operation, fetch_one::FetchOne},
};

pub type JsonValue = serde_json::Value;

pub struct JsonClient<S: Database> {
    pub collections: HashMap<String, Arc<dyn JsonCollection<S>>>,
    pub links: HashMap<String, Arc<dyn JsonLink<S>>>,
    pub pool: Pool<S>,
}

pub mod json_link {
    use crate::{
        links::dynamic_link::{CollectionsStore, DynamicLink},
        operations::BoxedOperation,
    };
    use serde::{Serialize, de::DeserializeOwned};
    use serde_json::{from_value, to_value};
    use sqlx::Database;
    use std::{any::Any, collections::HashMap, sync::Arc};

    use crate::{
        Expression, OpExpression, ToStaticExpressions, ZeroOrMoreExpressions,
        expressions::scoped_column,
        from_row::pre_alias,
        functional_expr::{BoxedExpression, StaticExpression},
        json_client::{JsonValue, json_collection::JsonCollection},
        links::{Link, relation_optional_to_many::optional_to_many},
        operations::{
            Operation,
            fetch_one::{LinkFetchOne, SelectStatementExtendableParts},
        },
    };

    pub trait JsonLink<S> {
        fn on_fetch_one_request(
            &self,
            base: Arc<dyn JsonCollection<S>>,
            input: JsonValue,
        ) -> Result<Box<dyn JsonLinkFetchOne<S>>, JsonValue>;
    }

    impl<S> CollectionsStore for Arc<dyn JsonCollection<S>> {
        type Store = HashMap<String, Self>;
    }

    impl<T, S> JsonLink<S> for T
    where
        S: Database,
        T: DynamicLink<Arc<dyn JsonCollection<S>>, S>,
        T::OnRequestInput: DeserializeOwned,
        T::OnRequestError: Serialize,
        T::OnRequest: JsonLinkFetchOne<S>,
    {
        fn on_fetch_one_request(
            &self,
            base: Arc<dyn JsonCollection<S>>,
            input: JsonValue,
        ) -> Result<Box<dyn JsonLinkFetchOne<S>>, JsonValue> {
            let req = self
                .on_request(base.clone(), from_value(input).unwrap())
                .map_err(|e| to_value(e).unwrap())
                .unwrap();
            Ok(Box::new(req))
        }
    }

    pub trait JsonLinkFetchOne<S>: 'static + Send
    where
        S: Database,
    {
        fn extend1(
            &self,
        ) -> SelectStatementExtendableParts<
            Vec<scoped_column<String, String>>,
            Vec<Box<dyn StaticExpression<S> + Send>>,
            Vec<Box<dyn StaticExpression<S> + Send>>,
        >;
        fn sub_op(
            &self,
            row: pre_alias<'_, <S as Database>::Row>,
        ) -> (Box<dyn BoxedOperation<S> + Send>, Box<dyn Any + Send>);
        fn take(
            self: Box<Self>,
            sub_op: Box<dyn Any + Send>,
            inner: Box<dyn Any + Send>,
        ) -> JsonValue;
    }

    impl<S, T> JsonLinkFetchOne<S> for T
    where
        T: Send + 'static,
        T: LinkFetchOne<
                S,
                SubOp: Send + Operation<S, Output: Send>,
                Inner: 'static + Send,
                Output: Serialize,
            >,
        T::Joins: ToStaticExpressions<S>,
        T::Wheres: ToStaticExpressions<S>,
        S: 'static + Database,
    {
        fn extend1(
            &self,
        ) -> SelectStatementExtendableParts<
            Vec<scoped_column<String, String>>,
            Vec<Box<dyn StaticExpression<S> + Send>>,
            Vec<Box<dyn StaticExpression<S> + Send>>,
        > {
            let s = <T as LinkFetchOne<S>>::extend_select(self);
            SelectStatementExtendableParts {
                non_aggregating_select_items: s.non_aggregating_select_items,
                non_duplicating_joins: s.non_duplicating_joins.to_static_expr(),
                wheres: s.wheres.to_static_expr(),
            }
        }
        fn sub_op(
            &self,
            row: pre_alias<'_, <S as sqlx::Database>::Row>,
        ) -> (
            Box<(dyn BoxedOperation<S> + std::marker::Send + 'static)>,
            Box<(dyn Any + Send + 'static)>,
        ) {
            let su = <T as LinkFetchOne<S>>::sub_op(self, row);
            (Box::new(su.0), Box::new(su.1))
        }
        fn take(
            self: Box<Self>,
            sub_op: Box<dyn Any + Send>,
            inner: Box<dyn Any + Send>,
        ) -> JsonValue {
            let inner = inner
                .downcast::<<T as LinkFetchOne<S>>::Inner>()
                .expect("bug: blanket impl should be consistant");

            let sub_op = sub_op
                .downcast::<<<T as LinkFetchOne<S>>::SubOp as Operation<S>>::Output>()
                .expect("bug: blanket impl should be consistant");

            to_value(<T as LinkFetchOne<S>>::take(*self, *sub_op, *inner))
                .expect("serializing should not fail")
        }
    }

    // pub struct JsonLinkFetchOneSpec<S> {
    //     pub link: Box<dyn JsonLinkFetchOne<S>>,
    // }

    /// reflexive impl -- errors has been cleared on JsonLink::on_request* and JsonLink::create_link
    impl<S> Link<Arc<dyn JsonCollection<S>>> for Box<dyn JsonLinkFetchOne<S>> {
        type Spec = Self;

        fn spec(self, _: &Arc<dyn JsonCollection<S>>) -> Self::Spec {
            self
        }
    }

    impl<S: Database + 'static> LinkFetchOne<S> for Box<dyn JsonLinkFetchOne<S>> {
        type Joins = Vec<Box<dyn StaticExpression<S> + Send>>;
        type Wheres = Vec<Box<dyn StaticExpression<S> + Send>>;

        fn extend_select(
            &self,
        ) -> crate::operations::fetch_one::SelectStatementExtendableParts<
            Vec<crate::expressions::scoped_column<String, String>>,
            Self::Joins,
            Self::Wheres,
        > {
            JsonLinkFetchOne::extend1(&**self)
        }

        type Inner = Box<dyn Any + Send>;

        type SubOp = Box<dyn BoxedOperation<S> + Send>;

        fn sub_op(
            &self,
            row: crate::from_row::pre_alias<'_, <S as sqlx::Database>::Row>,
        ) -> (Self::SubOp, Self::Inner)
        where
            S: sqlx::Database,
        {
            JsonLinkFetchOne::sub_op(&**self, row)
        }

        type Output = JsonValue;

        fn take(
            self,
            extend: <Self::SubOp as Operation<S>>::Output,
            inner: Self::Inner,
        ) -> Self::Output {
            JsonLinkFetchOne::take(self, extend, inner)
        }
    }
}

pub mod json_collection {
    use std::{marker::PhantomData, sync::Arc};

    use serde::Serialize;
    use serde_json::{map::Iter, to_value};
    use sqlx::{Database, Row};

    use crate::{
        collections::{Collection, CollectionBasic, SingleIncremintalInt},
        extentions::Members,
        from_row::FromRowAlias,
        json_client::JsonValue,
    };

    pub trait JsonCollection<S>: 'static + Send + Sync {
        fn table_name(&self) -> &str;
        fn table_name_lower_case(&self) -> &str;
        fn from_row_pre_alias<'r>(&self, row: crate::from_row::pre_alias<'r, S::Row>) -> JsonValue
        where
            S: Database;
        fn members(&self) -> Vec<String>;
    }

    impl<T, S> JsonCollection<S> for T
    where
        T: Send + Sync + 'static,
        S: Database,
        T::Data: Serialize,
        T: Collection<Id = SingleIncremintalInt>,
        T: for<'r> FromRowAlias<'r, S::Row>,
        T: Members<S>,
    {
        fn table_name(&self) -> &str {
            CollectionBasic::table_name(self)
        }

        fn table_name_lower_case(&self) -> &str {
            CollectionBasic::table_name_lower_case(self)
        }
        fn from_row_pre_alias<'r>(
            &self,
            row: crate::from_row::pre_alias<'r, <S as sqlx::Database>::Row>,
        ) -> JsonValue
        where
            S: Database,
        {
            to_value(T::pre_alias(self, row).expect("sound claw_ql code"))
                .expect("sound value impl")
        }
        fn members(&self) -> Vec<String> {
            Members::members_names(self)
        }
    }

    impl<S: 'static> CollectionBasic for Arc<dyn JsonCollection<S>> {
        fn table_name(&self) -> &str {
            JsonCollection::table_name(&**self)
        }

        fn table_name_lower_case(&self) -> &str {
            JsonCollection::table_name_lower_case(&**self)
        }
    }

    impl<'r, S: Database> FromRowAlias<'r, S::Row> for Arc<dyn JsonCollection<S>> {
        fn no_alias(&self, row: &'r S::Row) -> Result<Self::Data, crate::from_row::FromRowError> {
            todo!("impl no alias")
        }

        fn pre_alias(
            &self,
            row: crate::from_row::pre_alias<'r, S::Row>,
        ) -> Result<Self::Data, crate::from_row::FromRowError> {
            Ok(JsonCollection::from_row_pre_alias(&**self, row))
        }

        fn post_alias(
            &self,
            row: crate::from_row::post_alias<'r, S::Row>,
        ) -> Result<Self::Data, crate::from_row::FromRowError> {
            todo!("impl post alias")
        }
    }

    impl<S: 'static> Collection for Arc<dyn JsonCollection<S>> {
        type Partial = ();

        type Data = JsonValue;

        type Id = SingleIncremintalInt;

        fn id(&self) -> &Self::Id {
            todo!()
        }
    }

    impl<S: 'static> Members<S> for Arc<dyn JsonCollection<S>> {
        fn members_names(&self) -> Vec<String> {
            JsonCollection::members(&**self)
        }
    }
}

#[derive(Debug, Serialize)]
pub enum FetchOneError {
    NotFound,
    NoCollectionWithName(String),
    LinkTypeIsNotRegistered(String),
    RegisteredError(JsonValue),
}

#[derive(Deserialize)]
pub struct FetchOneInput {
    pub base: String,
    #[serde(default)]
    pub wheres: Vec<String>,
    pub link: Vec<LinkSpec>,
}

#[derive(Deserialize)]
pub struct LinkSpec {
    ty: String,
    #[serde(flatten)]
    rest: JsonValue,
}

impl<S> JsonClient<S>
where
    S: Database + DatabaseExt,
    for<'q> &'q str: ColumnIndex<S::Row>,
    for<'c> &'c mut <S as sqlx::Database>::Connection: Executor<'c, Database = S>,
    for<'q> i64: Decode<'q, S> + Type<S>,
{
    pub async fn fetch_one(
        &self,
        input: FetchOneInput,
    ) -> Result<LinkedOutput<i64, JsonValue, Vec<JsonValue>>, FetchOneError> {
        let base = self
            .collections
            .get(&input.base)
            .ok_or(FetchOneError::NoCollectionWithName(input.base))?;

        let op = FetchOne {
            base: base.clone(),
            wheres: (),
            link: {
                let mut v = Vec::default();
                for link_spec in input.link {
                    v.push(
                        self.links
                            .get(&link_spec.ty)
                            .ok_or(FetchOneError::LinkTypeIsNotRegistered(link_spec.ty))?
                            .on_fetch_one_request(base.clone(), link_spec.rest)
                            .map_err(|e| FetchOneError::RegisteredError(e))?,
                    );
                }
                v
            },
        };

        if let Some(e) = op.exec(self.pool.clone()).await {
            Ok(e)
        } else {
            Err(FetchOneError::NotFound)
        }
    }
}
