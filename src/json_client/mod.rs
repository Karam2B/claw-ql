// pub mod as_router;
// pub mod from_inventory;
// pub mod update_one;
// pub mod add_collection;
// pub mod delete_one;
// pub mod insert_one;
pub mod fetch_one;

use serde::{Deserialize, Serialize};
use sqlx::{ColumnIndex, Database, Decode, Executor, Pool, Type};
use std::{collections::HashMap, sync::Arc};

use crate::{
    database_extention::DatabaseExt,
    json_client::{json_collection::JsonCollection, json_link::JsonLink},
    operations::{LinkedOutput, fetch_one::FetchOne},
};

pub type JsonValue = serde_json::Value;

pub struct JsonClient<S: Database> {
    pub collections: HashMap<String, Arc<dyn JsonCollection<S>>>,
    pub links: HashMap<String, Arc<dyn JsonLink<S>>>,
    pub pool: Pool<S>,
}

/// old api
// pub struct JsonClient<S: Database> {
//     pub collections: HashMap<String, Box<dyn JsonCollection<S>>>,
//     pub links: HashMap<String, Box<dyn LiqLinkExt<S>>>,
//     pub type_extentions: HashMap<String, Box<dyn LiqType<S>>>,
//     pub filter_extentions: HashMap<String, Box<dyn LiqFilter<S>>>,
//     pub errors_log: Vec<(ErrorId, serde_json::Value)>,
//     pub error_count: AtomicI64,
//     pub migration: Vec<MigrationStep>,
//     pub db: Pool<S>,
// }

pub mod json_link {
    use crate::{
        json_client::fetch_one::{self, JsonLinkFetchOne},
        links::{CollectionsStore, DynamicLink},
    };
    use serde::{Serialize, de::DeserializeOwned};
    use sqlx::Database;
    use std::{collections::HashMap, sync::Arc};

    use crate::json_client::{JsonValue, json_collection::JsonCollection};

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
            fetch_one::on_fetch_one_request(self, base, input)
        }
    }
}

pub mod json_collection {
    use std::sync::Arc;

    use serde::Serialize;
    use serde_json::to_value;
    use sqlx::Database;

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
        <T as Collection>::Data: Serialize,
        T: Collection<Id = SingleIncremintalInt>,
        T: for<'r> FromRowAlias<'r, S::Row, FromRowData = T::Data>,
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
        type FromRowData = JsonValue;
        fn no_alias(
            &self,
            _: &'r S::Row,
        ) -> Result<Self::FromRowData, crate::from_row::FromRowError> {
            todo!("impl no alias")
        }

        fn pre_alias(
            &self,
            row: crate::from_row::pre_alias<'r, S::Row>,
        ) -> Result<Self::FromRowData, crate::from_row::FromRowError> {
            Ok(JsonCollection::from_row_pre_alias(&**self, row))
        }

        fn post_alias(
            &self,
            _: crate::from_row::post_alias<'r, S::Row>,
        ) -> Result<Self::FromRowData, crate::from_row::FromRowError> {
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

        // take where into account
        None::<()>.unwrap();

        let op = FetchOne {
            base: base.clone(),
            wheres: (),
            links: {
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
        use crate::operations::Operation;

        if let Some(e) = op.exec_operation(self.pool.clone()).await {
            Ok(e)
        } else {
            Err(FetchOneError::NotFound)
        }
    }
}
