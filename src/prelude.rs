#![allow(unexpected_cfgs)]

#[doc(hidden)]
// used for claw_wl_macros to simplify macro code
pub mod on_migrate_derive {
    pub use crate::{
        collections::Collection,
        expressions::col_def_for_collection_member,
        expressions::table_as_expression,
        on_migrate::OnMigrate,
        statements::create_table_statement::{CreateTable, expressions::*},
    };
}

#[doc(hidden)]
// used for claw_wl_macros to simplify macro code
pub mod from_row_alias {
    pub use crate::from_row::*;
    pub use sqlx::ColumnIndex;
    pub use sqlx::Decode;
    pub use sqlx::Error;
    pub use sqlx::Row;
    pub use sqlx::Type;
}

#[doc(hidden)]
// used for claw_wl_macros to simplify macro code
pub mod sql {
    pub use crate::operations::Operation;
    pub use crate::query_builder::functional_expr::ManyPossible;

    // migrate
    pub use crate::on_migrate::OnMigrate;
    pub use crate::operations::execute_expression::ExpressionAsOperation;

    pub trait AliasAndExpr<A, E> {
        fn aliase_and_expr(alias: A, expr: E) -> Self;
    }
}

#[doc(hidden)]
// used for claw_wl_macros to simplify macro code
pub mod macro_derive_collection {
    pub use crate::collections::Collection;
    pub use crate::collections::CollectionBasic;
    pub use crate::collections::HasHandler;
    pub use crate::collections::CollectionId;
    pub use crate::collections::Member;
    pub use crate::collections::SingleIncremintalInt;
    pub use crate::database_extention::DatabaseExt;
    pub use crate::expressions::member_as_expression;
    pub use crate::extentions::Members;
    pub use crate::query_builder::Expression;
    pub use crate::query_builder::StatementBuilder;
    pub use crate::query_builder::functional_expr::BoxedExpression;
    pub use crate::update_mod::Update;
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
#[doc(hidden)]
// used for claw_wl_macros to simplify macro code
pub mod inventory {
    pub use crate::inventory::*;
    pub use crate::links::relation::Relation;
    pub use inventory::submit;
    pub use std::sync::Arc;
}
