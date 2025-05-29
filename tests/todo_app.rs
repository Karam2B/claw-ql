use claw_ql::migration::{migrate, migrate_relation};
use claw_ql_macros::{Collection, relation};
use sqlx::SqlitePool;
use tracing::Level;

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

#[tokio::test]
async fn main() {
    let pool = SqlitePool::connect("sqlite::memory:").await.unwrap();

    tracing_subscriber::fmt()
        .with_max_level(Level::DEBUG)
        .init();

    migrate::<_, Todo>(&pool).await;
    migrate::<_, Category>(&pool).await;
    migrate::<_, Tag>(&pool).await;
    migrate_relation::<_, Todo, Category>(&pool).await;
}
