use claw_ql::{
    builder_pattern::BuilderPattern, json_client::{builder_pattern::to_json_client, JsonClient},
    links::relation::Relation, migration::to_migrate,
};
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

    let (migrator, client) = {
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
            .finish()
    };

    migrator.migrate(pool.clone()).await;
    let _: JsonClient<Sqlite> = client.unwrap();
}
