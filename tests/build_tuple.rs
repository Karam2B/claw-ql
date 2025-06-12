use claw_ql::builder_pattern::on_migrate_builder::to_migrate;
use claw_ql::builder_pattern::{BuilderPattern, on_json_client::to_json_client};
use claw_ql::links::relation::Relation;
use claw_ql_macros::{Collection, relation};
use serde::{Deserialize, Serialize};
use sqlx::{Sqlite, SqlitePool};

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
async fn test() {
    tracing_subscriber::fmt()
        .with_max_level(tracing::Level::DEBUG)
        .init();

    let pool = SqlitePool::connect("sqlite::memory:").await.unwrap();

    let client = {
        BuilderPattern::default()
            .build_mode(to_migrate(Sqlite))
            .build_mode(to_json_client(pool.clone()))
            .add_collection(todo)
            .add_collection(tag)
            .add_collection(category)
            .add_link(Relation {
                from: todo,
                to: tag,
            })
            .add_link(Relation {
                from: todo,
                to: category,
            })
    };

    let results = client.finish();

    results.0.migrate(pool.clone()).await;
    // let jc = results.1.unwrap();
}
