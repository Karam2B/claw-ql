#![allow(unused)]
#![deny(unused_must_use)]
use claw_ql::{
    ConnectInMemory, builder_pattern::BuilderPattern, links::relation::Relation,
    migration::MigratorBuilder,
};
use claw_ql_macros::{Collection, relation};
use serde::{Deserialize, Serialize};
use sqlx::Sqlite;

#[derive(Collection, Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct Todo {
    pub title: String,
    pub done: bool,
    pub description: Option<String>,
}

#[derive(Collection, Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct Category {
    pub title: String,
}

#[derive(Collection, Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct Tag {
    pub title: String,
}

relation!(optional_to_many Todo Category);
relation!(many_to_many Todo Tag);

#[tokio::test]
async fn migration() {
    let pool = Sqlite::connect_in_memory().await;
    let mut schema = BuilderPattern::default()
        .build_component(MigratorBuilder::<Sqlite>::default())
        .start_mut();

    schema.add_collection(&category);
    schema.add_collection(&tag);
    schema.add_collection(&todo);

    let schema = schema.finish().0;
    schema.migrate(pool).await;

    // panic!("hello world")
}
