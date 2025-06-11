use claw_ql::{
    dynamic_client::DynamicClient,
    filters::by_id_mod::by_id,
    links::{relation::Relation, set_id::SetId, set_new::SetNew},
    operations::{
        CollectionOutput, LinkedOutput,
        insert_one_op::insert_one,
        select_one_op::select_one,
        update_one_op::{update_one, update_one_no_id},
    },
    update_mod::update,
};
use claw_ql_macros::{Collection, relation};
use serde::{Deserialize, Serialize};
use serde_json::json;
use sqlx::{Sqlite, SqlitePool};
use tracing::Level;

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
async fn main() {
    let pool = SqlitePool::connect("sqlite::memory:").await.unwrap();

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

    let res = insert_one(Todo {
        title: "new todo".to_string(),
        done: false,
        description: None,
    })
    .link(SetNew {
        input: Category {
            title: "new category".to_string(),
        },
    })
    .link(SetId {
        to: tag,
        input: vec![1, 3],
    })
    .exec_op(&pool)
    .await;

    pretty_assertions::assert_eq!(
        res,
        LinkedOutput {
            id: 6,
            attr: Todo {
                title: "new todo".to_string(),
                done: false,
                description: None
            },
            links: (
                CollectionOutput {
                    id: 4,
                    attr: Category {
                        title: "new category".to_string()
                    }
                },
                vec![1, 3],
            ),
        }
    );

    // let res = insert_one(Todo {
    //     title: "new todo".to_string(),
    //     done: false,
    //     description: None,
    // });

    update_one(
        6,
        TodoPartial {
            title: update::set("update title".to_string()),
            done: update::keep,
            description: update::keep,
        },
    )
    .exec_op(&pool)
    .await
    .unwrap();

    assert_eq!(
        sqlx::query_as::<_, (String,)>("SELECT title FROM Todo WHERE id = 6;")
            .fetch_one(&pool)
            .await
            .unwrap(),
        ("update title".to_string(),)
    );

    // using generic operatioin
    let res = select_one(todo)
        .relation(category)
        .filter(by_id(4))
        .exec_op(pool.clone())
        .await;

    pretty_assertions::assert_eq!(
        res,
        Some(LinkedOutput {
            id: 4,
            attr: Todo {
                title: "todo_4".to_string(),
                done: false,
                description: None
            },
            links: (Some(CollectionOutput {
                id: 1,
                attr: Category {
                    title: "category_1".to_string()
                }
            }),),
        })
    );

    let jc = schema.create_json_client(pool.clone()).unwrap();

    // using dynamic operation
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
