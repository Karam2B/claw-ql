//! todo list
//!
//! - [ ] add where clase
//! - [ ] make sql macro
//! - [ ] clear out junk out of codebase
//! - [ ] basic migrate function
//! - [ ] make readme
//!
//!
//! - [ ] MAJOR REALEASE
//!
//! - [ ] figure out nested where op
//! - [ ] figure out nested links
//! - [ ] json_client create link
//! - [ ] json_client modify link
//! - [ ] add many_to_many link type
//! - [ ] add one_to_many link type
//! - [ ] add date link type
//! - [ ] add fetch many operation
//! - [ ] add insert operation
//! - [ ] add update operation
//! - [ ] add delete operation
//! - [ ] add more where operations

// pub mod build_tuple;
pub mod collections;
pub mod execute;
// pub mod expressions;
// pub mod filters;
// #[cfg(feature = "http")]
// pub mod http;
// mod identity_management;
// #[cfg(feature = "inventory")]
// pub mod inventory;
// pub mod json_client;
#[path = "./json_client2/mod.rs"]
pub mod json_client;
// #[cfg(feature = "serde")]
// pub mod json_query;
// pub mod json_value_cmp;
pub mod links;
// pub mod migration;
pub mod from_row;
pub mod prelude;
#[path = "./owned_query_builder/mod.rs"]
pub mod query_builder;
pub use query_builder::*;
// pub mod query_builder;
#[path = "./operations_2/mod.rs"]
pub mod operations;
pub mod update_mod;
// pub mod operations;
// pub mod on_migrate;
// pub mod quick_query;
// mod schema;
// pub mod statements;
// pub mod update_mod;
// pub mod ident;
// pub mod macros {
//     pub use claw_ql_macros::*;
// }
pub mod connect_in_memory;
pub use connect_in_memory::*;
pub mod database_extention;
pub use database_extention::*;
#[rustfmt::skip]
pub mod count;
pub mod extend_sqlite;
pub use serde_json::Value as JsonValue;
pub use sqlx;
pub mod extentions;
#[path = "./on_migrate2.rs"]
pub mod on_migrate;
pub mod zero_sized_default;
