#![allow(unexpected_cfgs)]

pub mod on_migrate_derive {
    pub use crate::{
        collections::Collection,
        expressions::col_def_for_collection_member,
        expressions::table_as_expression,
        on_migrate::OnMigrate,
        statements::create_table_statement::{CreateTable, expressions::*},
    };
}

pub mod from_row_alias {
    pub use crate::from_row::*;
    pub use sqlx::ColumnIndex;
    pub use sqlx::Decode;
    pub use sqlx::Error;
    pub use sqlx::Row;
    pub use sqlx::Type;
}
pub mod sql {
    pub use crate::operations::Operation;
    pub use crate::operations::fetch_one::FetchOne;
    pub use crate::valid_syntax::is_valid_syntax;
    pub use crate::valid_syntax::temp::*;
}

pub mod macro_derive_collection {
    pub use crate::collections::Collection;
    pub use crate::collections::CollectionBasic;
    pub use crate::collections::HasHandler;
    pub use crate::collections::Id;
    pub use crate::collections::Member;
    pub use crate::collections::MemberBasic;
    pub use crate::collections::SingleIncremintalInt;
    pub use crate::database_extention::DatabaseExt;
    pub use crate::expressions::member_as_expression;
    pub use crate::extentions::Members;
    pub use crate::query_builder::Expression;
    pub use crate::query_builder::QueryBuilder;
    pub use crate::query_builder::SqlSanitize;
    pub use crate::query_builder::functional_expr::BoxedExpression;
    pub use crate::update_mod::update;
    pub use core::future::Future;
    pub use core::marker::PhantomData;
    #[cfg(feature = "serde")]
    pub use serde::Deserialize;
    pub use sqlx::ColumnIndex;
    pub use sqlx::Database;
    pub use sqlx::Decode;
    pub use sqlx::Encode;
    pub use sqlx::IntoArguments;
    pub use sqlx::Row;
    pub use sqlx::Type;
}

#[cfg(feature = "inventory")]
pub mod inventory {
    pub use crate::inventory::*;
    pub use crate::links::relation::Relation;
    pub use inventory::submit;
    pub use std::sync::Arc;
}
