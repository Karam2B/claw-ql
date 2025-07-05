use claw_ql::{
    builder_pattern::BuilderPattern,
    filters::by_id_mod::by_id,
    json_client::JsonClient,
    links::{relation::Relation, set_id::SetId, set_new::SetNew},
    migration::MigratorBuilder,
    operations::{
        CollectionOutput, LinkedOutput, delete_one_op::delete_one, insert_one_op::insert_one,
        select_one_op::select_one, update_one_op::update_one,
    },
    update_mod::update,
};
use claw_ql_macros::{Collection, relation};
use serde::{Deserialize, Serialize};
use serde_json::json;
use sqlx::{Pool, Sqlite, SqlitePool};

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

async fn dumpy_data(db: Pool<Sqlite>) {
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
    .execute(&db)
    .await
    .unwrap();
}

#[tokio::test]
async fn _workflow_generic() {
    tracing_subscriber::fmt()
        .with_max_level(tracing::Level::DEBUG)
        .init();

    let pool = SqlitePool::connect("sqlite::memory:").await.unwrap();

    let mut schema = BuilderPattern::default()
        .build_component(MigratorBuilder::default())
        .start_mut();

    schema.add_collection(&category);
    schema.add_collection(&tag);
    schema.add_collection(&todo);
    schema.add_link(&Relation::link(todo, tag));
    schema.add_link(&Relation::link(todo, category));

    let schema = schema.finish().0;

    schema.migrate(pool.clone()).await;

    dumpy_data(pool.clone()).await;

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

    let res = delete_one(4, todo).relation(category).exec_op(&pool).await;

    assert_eq!(
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

    assert!(
        sqlx::query("SELECT title FROM Todo WHERE id = 4;")
            .fetch_optional(&pool)
            .await
            .unwrap()
            .is_none()
    );
}

#[tokio::test]
async fn workflow_dynamic() {
    let pool = SqlitePool::connect("sqlite::memory:").await.unwrap();

    let mut schema = BuilderPattern::default()
        .build_component(JsonClient::builder(pool.clone()))
        .build_component(MigratorBuilder::default())
        .start_mut();

    schema.add_collection(&category);
    schema.add_collection(&tag);
    schema.add_collection(&todo);
    schema.add_link(&Relation::link(todo, category));

    let schema = schema.finish();

    let jc = schema.0.unwrap();
    schema.1.migrate(pool.clone()).await;
    dumpy_data(pool).await;

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
