use claw_ql::{
    dynamic_client::DynamicClient,
    links::relation::Relation,
    operations::{
        SimpleOutput,
        select_one::{SelectOneOutput, select_one},
    },
};
use claw_ql_macros::{Collection, relation};
use serde::{Deserialize, Serialize};
use serde_json::json;
use sqlx::{Sqlite, SqlitePool};
use tracing::Level;

#[derive(
    Collection, Debug, PartialEq, Serialize, Deserialize,
)]
pub struct Todo {
    pub title: String,
    pub done: bool,
    pub description: Option<String>,
}

#[derive(
    Collection, Debug, PartialEq, Serialize, Deserialize,
)]
pub struct Category {
    pub title: String,
}

#[derive(
    Collection, Debug, PartialEq, Serialize, Deserialize,
)]
pub struct Tag {
    pub title: String,
}

relation!(optional_to_many Todo Category);
relation!(many_to_many Todo Tag);

#[tokio::test]
async fn main() {
    let pool =
        SqlitePool::connect("sqlite::memory:").await.unwrap();

    tracing_subscriber::fmt()
        .with_max_level(Level::DEBUG)
        .init();

    let schema = {
        DynamicClient::default()
            .infer_db::<Sqlite>()
            .add_link(Relation {
                from: todo,
                to: tag,
            })
            .add_link(Relation {
                from: todo,
                to: category,
            })
            .add_collection(category)
            .add_collection(tag)
            .add_collection(todo)
    };

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

    let res = select_one(todo)
        .relation(category)
        .exec_op(pool.clone())
        .await;

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

    let jc = schema.create_json_client(pool.clone()).unwrap();

    let res = jc
        .select_one(json!({
            "collection": "todo",
            "links": { "relation": { "category": {} } }
        }))
        .await
        .unwrap();

    pretty_assertions::assert_eq!(
        res,
        json!({
            "id": 1,
            "attr": {
                "title": "todo_1",
                "done": true,
                "description": null
            },
            "links": {
                "relation": {
                    "category": { "id": 3, "attr": {"title": "category_3"}}
                }
            }
        }),
    );
}
