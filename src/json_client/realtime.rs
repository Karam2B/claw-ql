use std::{collections::HashMap, marker::PhantomData};

use serde_json::Value;
use sqlx::{Database, Decode, Encode, Pool, prelude::Type};

use crate::{
    QueryBuilder,
    json_client::{DynamicLinkRT, JsonCollection, JsonSelector},
    prelude::{
        col,
        stmt::{InsertOneSt, SelectSt},
    },
    statements::update_st::UpdateSt,
};

// pub struct JsonRealtimeClient<S: Database> {
//     pub collections: HashMap<String, DynamicCollection<S>>,
//     pub links: HashMap<JsonSelector, Box<dyn DynamicLinkRT<S>>>,
//     pub db: Pool<S>,
// }

pub trait Encoder<S>: Sync + Send + 'static {}

impl<S: Database, T> Encoder<S> for PhantomData<T> where
    T: Encode<'static, S> + Decode<'static, S> + Type<S> + Send + Sync + 'static
{
}

// pub struct DynamicCollection<S> {
//     table_name: String,
//     fields: HashMap<String, Box<dyn Encoder<S>>>,
// }

// impl<S> JsonCollectionB for DynamicCollection<S> {
//     fn members_(&self) -> Vec<String> {
//         self.fields.keys().cloned().collect()
//     }
//     fn table_name_(&self) -> &str {
//         &self.table_name
//     }
// }

// impl<S: QueryBuilder + 'static> JsonCollection<S> for DynamicCollection<S> {
//     fn table_name(&self) -> &str {
//         self.table_name.as_str()
//     }

//     fn members(&self) -> Vec<String> {
//         self.fields.keys().cloned().collect()
//     }

//     fn on_select(&self, stmt: &mut SelectSt<S>)
//     where
//         S: QueryBuilder,
//     {
//         for (key, val) in self.fields.iter() {
//             stmt.select(col(key))
//         }
//     }

//     fn on_insert(&self, this: Value, stmt: &mut InsertOneSt<S>) -> Result<(), String>
//     where
//         S: sqlx::Database,
//     {
//         todo!()
//     }

//     fn on_update(&self, this: Value, stmt: &mut UpdateSt<S>) -> Result<(), String>
//     where
//         S: QueryBuilder,
//     {
//         todo!()
//     }

//     fn from_row_noscope(&self, row: &<S>::Row) -> Value
//     where
//         S: Database,
//     {
//         todo!()
//     }

//     fn from_row_scoped(&self, row: &<S>::Row) -> Value
//     where
//         S: Database,
//     {
//         todo!()
//     }
// }
