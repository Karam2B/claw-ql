use claw_ql_macros::simple_enum;
use hyper::StatusCode;
use serde::Serialize;
use serde::de::DeserializeOwned;
use serde_json::{Value, from_value};
use sqlx::{ColumnIndex, Database, Decode, Encode, Pool, Sqlite, prelude::Type};
use std::marker::PhantomData;

use crate::{
    Buildable, QueryBuilder,
    expressions::primary_key,
    statements::{create_table_st::header, select_st::SelectSt},
};
pub trait CollectionBasic {
    fn table_name(&self) -> &str;
    fn table_name_lower_case(&self) -> &str;
}

pub trait Collection: CollectionBasic {
    type Partial;
    type Data;
    type Members;
    fn members(&self) -> &Self::Members;
    type Id;
    fn id(&self) -> &Self::Id;
}

pub trait Queries<S>: Collection {
    // to deprecate, will be replaced with type `OnSelect: IntoQuery<Base = Self>`
    fn on_select(&self, stmt: &mut SelectSt<S>)
    where
        S: QueryBuilder;

    // fn on_insert(&self, this: Self::Data, stmt: &mut InsertOneSt<S>)
    // where
    //     S: sqlx::Database;
    // fn on_update(&self, this: Self::Partial, stmt: &mut UpdateSt<S>)
    // where
    //     S: QueryBuilder;

    fn from_row_scoped(&self, row: &<<S as QueryBuilder>::SqlxDb as Database>::Row) -> Self::Data
    where
        S: QueryBuilder,
        S::SqlxDb: Database;
}

pub trait MemberBasic {
    fn name(&self) -> &str;
}

pub trait Member: MemberBasic {
    type Data;
    type Collection;
}

pub trait HasHandler {
    type Handler: Collection;
}

pub trait Id {
    type SqlIdent;
    fn ident(&self) -> Self::SqlIdent;
}

pub struct SingleIncremintalInt;

impl Id for SingleIncremintalInt {
    type SqlIdent = &'static str;
    fn ident(&self) -> &'static str {
        "id"
    }
}
