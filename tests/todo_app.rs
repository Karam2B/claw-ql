use claw_ql::{
    links::group_by::count,
    operations::{
        Relation, SimpleOutput,
        select_one::{SelectOneOutput, get_one},
    },
    schema::DynamicClient,
};
use claw_ql_macros::{Collection, relation};
use serde::{Deserialize, Serialize};
use sqlx::{Sqlite, SqlitePool};
use tracing::Level;

#[derive(Collection, Debug, PartialEq, Serialize, Deserialize)]
pub struct Todo {
    pub title: String,
    pub done: bool,
    pub description: Option<String>,
}

#[derive(Collection, Debug, PartialEq, Serialize, Deserialize)]
pub struct Tag {
    pub title: String,
}

#[derive(Collection, Debug, PartialEq, Serialize, Deserialize)]
pub struct Category {
    pub title: String,
}

relation!(optional_to_many Todo Category);

#[tokio::test]
async fn main() {
    let pool = SqlitePool::connect("sqlite::memory:").await.unwrap();

    tracing_subscriber::fmt()
        .with_max_level(Level::DEBUG)
        .init();

    let schema = DynamicClient::default()
        .infer_db::<Sqlite>()
        .catch_errors_early()
        .add_relation(Relation {
            from: todo,
            to: category,
        })
        .add_collection(category)
        .add_collection(tag)
        .add_collection(todo);

    schema.migrate(&pool).await;

    sqlx::query(
        r#"
            INSERT INTO Tag (title) VALUES 
                ('tag_1'), ('tag_2'), ('tag_3');

            INSERT INTO Category (title) VALUES ('category_1'), ('category_2'), ('category_3');
            

            INSERT INTO Todo (title, done, category_id) VALUES
                ('todo_1', 1, 3),
                ('todo_2', 0, 3),
                ('todo_3', 1, NULL),
                ('todo_4', 0, 1),
                ('todo_5', 1, NULL);
            "#,
    )
    .execute(&pool)
    .await
    .unwrap();

    let res = get_one(todo).relation(category).exec_op(pool.clone()).await;

    pretty_assertions::assert_eq!(
        res,
        Some(SelectOneOutput {
            id: 1,
            attr: Todo {
                title: "todo_1".to_string(),
                done: true,
                description: None
            },
            links: (Some(SimpleOutput {
                id: 3,
                attr: Category {
                    title: "category_3".to_string()
                }
            }),),
        })
    );

    let client = schema.create_json_client(pool.clone());
}
