//! todo list
//!
//! - [x] add where clase
//! - [x] make sql macro
//! - [x] clear out junk out of codebase
//! - [x] basic migrate function
//! - [ ] make readme
//!
//!
//! - [ ] MAJOR REALEASE
//!
//! - [ ] create internal ticket system!
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

pub mod collections;
pub mod execute;
pub mod expressions;
pub mod from_row;
pub mod json_client;
pub mod json_value_cmp;
pub mod links;
pub mod on_migrate;
pub mod operations;
pub mod prelude;
pub mod query_builder;
pub mod schema;
pub mod statements;
pub mod update_mod;
pub mod valid_syntax;
pub mod macros {
    pub use claw_ql_macros::*;
}
pub mod connect_in_memory;
pub mod database_extention;
pub mod extend_sqlite;
pub mod extentions;
pub mod zero_sized_default;

/// usefull old utils, they all in utils folder, I don't want to delete because I might come back for them!
/// the way I orgnize utils is by placing directly besie lib.rs
#[cfg(feature = "skip_without_comments")]
pub mod utils;
