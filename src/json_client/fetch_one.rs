use crate::{
    links::DynamicLink,
    operations::BoxedOperation,
    query_builder::{ToStaticExpressions, functional_expr::StaticExpression},
};
use serde::{Serialize, de::DeserializeOwned};
use serde_json::{from_value, to_value};
use sqlx::Database;
use std::{any::Any, sync::Arc};

use crate::{
    expressions::scoped_column,
    from_row::pre_alias,
    json_client::{JsonValue, json_collection::JsonCollection},
    links::Link,
    operations::{Operation, fetch_one::LinkFetchOne},
};

pub trait JsonLinkFetchOne<S>: 'static + Send
where
    S: Database,
{
    fn non_aggregating_select_items(&self) -> Vec<scoped_column<String, String>>;
    fn non_duplicating_joins(&self) -> Vec<Box<dyn StaticExpression<S> + Send>>;
    fn wheres(&self) -> Vec<Box<dyn StaticExpression<S> + Send>>;
    fn sub_op(
        &self,
        row: pre_alias<'_, <S as Database>::Row>,
    ) -> (Box<dyn BoxedOperation<S> + Send>, Box<dyn Any + Send>);
    fn take(self: Box<Self>, sub_op: Box<dyn Any + Send>, inner: Box<dyn Any + Send>) -> JsonValue;
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
    fn non_aggregating_select_items(&self) -> Vec<scoped_column<String, String>> {
        <T as LinkFetchOne<S>>::non_aggregating_select_items(self)
    }
    fn non_duplicating_joins(&self) -> Vec<Box<dyn StaticExpression<S> + Send>> {
        <T as LinkFetchOne<S>>::non_duplicating_joins(self).to_static_expr()
    }
    fn wheres(&self) -> Vec<Box<dyn StaticExpression<S> + Send>> {
        <T as LinkFetchOne<S>>::wheres(self).to_static_expr()
    }
    fn sub_op(
        &self,
        row: pre_alias<'_, <S as sqlx::Database>::Row>,
    ) -> (
        Box<dyn BoxedOperation<S> + std::marker::Send + 'static>,
        Box<dyn Any + Send + 'static>,
    ) {
        let su = <T as LinkFetchOne<S>>::sub_op(self, row);
        (Box::new(su.0), Box::new(su.1))
    }
    fn take(self: Box<Self>, sub_op: Box<dyn Any + Send>, inner: Box<dyn Any + Send>) -> JsonValue {
        let inner = inner
            .downcast::<<T as LinkFetchOne<S>>::Inner>()
            .expect("bug: blanket impl should be consistant");

        let sub_op = sub_op
            .downcast::<<<T as LinkFetchOne<S>>::SubOp as Operation<S>>::Output>()
            .expect("bug: blanket impl should be consistant");

        to_value(<T as LinkFetchOne<S>>::take(*self, *sub_op, *inner))
            .expect("bug: serializing should not fail")
    }
}

impl<S: Database + 'static> LinkFetchOne<S> for Box<dyn JsonLinkFetchOne<S>> {
    type Joins = Vec<Box<dyn StaticExpression<S> + Send>>;
    type Wheres = Vec<Box<dyn StaticExpression<S> + Send>>;

    fn non_aggregating_select_items(&self) -> Vec<scoped_column<String, String>> {
        JsonLinkFetchOne::non_aggregating_select_items(&**self)
    }
    fn non_duplicating_joins(&self) -> Vec<Box<dyn StaticExpression<S> + Send>> {
        JsonLinkFetchOne::non_duplicating_joins(&**self)
    }
    fn wheres(&self) -> Vec<Box<dyn StaticExpression<S> + Send>> {
        JsonLinkFetchOne::wheres(&**self)
    }

    // fn extend_select(
    //     &self,
    // ) -> crate::operations::fetch_one::SelectStatementExtendableParts<
    //     Vec<crate::expressions::scoped_column<String, String>>,
    //     Self::Joins,
    //     Self::Wheres,
    // > {
    //     JsonLinkFetchOne::extend1(&**self)
    // }

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

/// reflexive impl -- errors has been cleared on JsonLink::on_request* and JsonLink::create_link
impl<S> Link<Arc<dyn JsonCollection<S>>> for Box<dyn JsonLinkFetchOne<S>> {
    type Spec = Self;

    fn spec(self, _: &Arc<dyn JsonCollection<S>>) -> Self::Spec {
        self
    }
}

/// JsonLink extention
pub(super) fn on_fetch_one_request<S, T>(
    jc: &T,
    base: Arc<dyn JsonCollection<S>>,
    input: JsonValue,
) -> Result<Box<dyn JsonLinkFetchOne<S>>, JsonValue>
where
    S: Database,
    T: DynamicLink<Arc<dyn JsonCollection<S>>, S>,
    T::OnRequestInput: DeserializeOwned,
    T::OnRequestError: Serialize,
    T::OnRequest: JsonLinkFetchOne<S>,
{
    let req = jc
        .on_request(base.clone(), from_value(input).unwrap())
        .map_err(|e| to_value(e).unwrap())
        .unwrap();
    Ok(Box::new(req))
}
