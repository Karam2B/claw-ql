use claw_ql::{
    migration::{migrate, migrate_relation},
    prelude::Execute,
    statements::create_table_st::{header, CreateTableSt},
};
use claw_ql_macros::{relation, Collection};
use sqlx::{Sqlite, SqlitePool};

#[derive(Collection, Debug, PartialEq)]
pub struct Todo {
    pub title: String,
    pub done: bool,
    pub description: Option<String>,
}

#[derive(Collection, Debug, PartialEq)]
pub struct Category {
    pub title: String,
}

#[derive(Collection, Debug, PartialEq)]
pub struct Tag {
    pub title: String,
}

relation!(optional_to_many Todo Category);

#[cfg(test)]
#[tokio::test]
async fn main() {
    let pool = SqlitePool::connect("sqlite::memory:").await.unwrap();

        tracing_subscriber::fmt()
            .with_max_level(tracing::Level::DEBUG)
            .init();

    migrate::<_, Todo>(&pool).await;
    migrate::<_, Category>(&pool).await;
    migrate::<_, Tag>(&pool).await;
    migrate_relation::<_, Todo, Category>(&pool).await;
}
