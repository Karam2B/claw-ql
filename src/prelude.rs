#![allow(unexpected_cfgs)]

#[cfg(skip_without_comments)]
pub mod stmt {
    pub use crate::statements::create_table_st::CreateTableSt;
    // pub use crate::statements::insert_one_st::InsertOneSt;
    pub use crate::statements::select_st::SelectSt;
}

pub mod on_migrate_derive {
    pub use crate::{
        collections::Collection,
        expressions::col_def_for_collection_member,
        expressions::table_as_expression,
        on_migrate::OnMigrate,
        statements::{CreateTable, create_table},
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

pub mod macro_derive_collection {
    pub mod sqlx_ {
        pub use sqlx::ColumnIndex;
    }
    // pub use crate::EncodeExtention;
    // pub use crate::QueryBuilder;
    // pub use crate::SanitzingMechanisim;
    pub use crate::collections::Collection;
    pub use crate::collections::CollectionBasic;
    pub use crate::collections::HasHandler;
    pub use crate::collections::Member;
    pub use crate::collections::MemberBasic;
    // pub use crate::collections::Queries;
    pub use crate::collections::SingleIncremintalInt;
    // pub use crate::expressions::col;
    // pub use crate::expressions::primary_key::DatabaseDefaultPrimaryKey;
    // pub use crate::sanitize::SanitizeAndHardcode;
    // pub use crate::statements::select_st::SelectSt;
    pub use sqlx::ColumnIndex;
    pub use sqlx::Database;
    pub use sqlx::Decode;
    pub use sqlx::Encode;
    pub use sqlx::Row;
    pub use sqlx::Type;
    // pub use crate::QueryBuilder;
    // pub use crate::collections::Collection;
    // pub use crate::collections::HasHandler;
    // pub use crate::expressions::exports::col_type_check_if_null;
    // pub use crate::expressions::exports::primary_key;
    // pub use crate::expressions::primary_key::DatabaseDefaultPrimaryKey;
    // pub use crate::migration::OnMigrate;
    // pub use crate::operations::collections::Collection;
    // pub use crate::expressions::col;
    // pub use crate::query_builder::Buildable;
    // pub use crate::statements::create_table_st::CreateTableSt;
    // pub use crate::statements::create_table_st::header;
    // pub use crate::statements::insert_one_st::InsertOneSt;
    // pub use crate::statements::select_st::SelectSt;
    // pub use crate::statements::update_st::UpdateSt;
    pub use crate::database_extention::DatabaseExt;
    pub use crate::extentions::Members;
    pub use crate::query_builder::Expression;
    pub use crate::query_builder::QueryBuilder;
    pub use crate::query_builder::SqlSanitize;
    pub use crate::query_builder::expressions::member_as_expression;
    pub use crate::query_builder::functional_expr::BoxedExpression;
    pub use crate::update_mod::update;
    pub use core::future::Future;
    pub use core::marker::PhantomData;
    #[cfg(feature = "serde")]
    pub use serde::Deserialize;
    // pub use sqlx::Database;
    // pub use sqlx::Decode;
    // pub use sqlx::Encode;
    pub use sqlx::Executor;
    pub use sqlx::IntoArguments;
    // pub use sqlx::Row;
    // pub use sqlx::Type;
}

// pub mod macro_relation {
//     pub use crate::collections::Collection;
//     pub use crate::collections::CollectionHandler;
//     pub use crate::links::Link;
//     pub use crate::operations::*;
//     pub use crate::statements::create_table_st::CreateTableSt;
//     pub use std::marker::PhantomData;

//     // supported relations
//     pub use crate::links::relation_many_to_many::*;
//     pub use crate::links::relation_optional_to_many::*;
// }

#[cfg(feature = "inventory")]
pub mod inventory {
    pub use crate::inventory::*;
    pub use crate::links::relation::Relation;
    pub use inventory::submit;
    pub use std::sync::Arc;
}
