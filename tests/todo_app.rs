use std::sync::Arc;

use claw_ql::{
    Schema,
    filters::by_id_mod::by_id,
    json_client::JsonClient,
    links::{relation::Relation, set_id::SetId, set_new::SetNew},
    migration::migrate_on_empty_database,
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

// mod dddd {
//     #![allow(non_camel_case_types)]

//     use claw_ql::{
//         links::LinkData,
//         prelude::macro_relation::{OptionalToMany, OptionalToManyInverse},
//     };

//     use crate::{category, tag, todo};

//     pub struct may_have_a_category {
//         // non-public for reason
//         foriegn_key: &'static str,
//     }

//     pub struct have_a_category {
//         // non-public for reason
//         foriegn_key: &'static str,
//     }

//     #[allow(non_camel_case_types)]
//     pub struct has_many_todos {
//         // non-public for reason
//         foriegn_key: &'static str,
//     }

//     #[allow(non_camel_case_types)]
//     pub struct todo_has_optional_to_many_tag {
//         // non-public for reason
//         foriegn_key: &'static str,
//     }

//     impl Default for todo_has_optional_to_many_tag {
//         fn default() -> Self {
//             todo_has_optional_to_many_tag {
//                 foriegn_key: "cat_id",
//             }
//         }
//     }

//     impl LinkData<super::todo> for todo_has_optional_to_many_tag {
//         type Spec = OptionalToMany<todo, tag>;

//         fn spec(self, _from: super::todo) -> Self::Spec
//         where
//             Self: Sized,
//         {
//             OptionalToMany {
//                 foriegn_key: self.foriegn_key.to_string(),
//                 from: todo,
//                 to: tag,
//             }
//         }
//     }

//     pub trait DefaultRelation {
//         type Normal;
//         type Inverse;
//     }

//     impl DefaultRelation for (todo, category) {
//         type Normal = may_have_a_category;
//         type Inverse = has_many_todos;
//     }

//     impl DefaultRelation for (category, todo) {
//         type Normal = has_many_todos;
//         type Inverse = may_have_a_category;
//     }
// }

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

    let schema = Schema {
        collections: (todo, category, tag),
        links: (Relation::link(todo, tag), Relation::link(todo, category)),
    };

    migrate_on_empty_database(&schema, &pool).await;

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
            ..Default::default()
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

    let schema = Schema {
        collections: (todo, category, tag),
        links: (Relation::link(todo, category), Relation::link(todo, tag)),
    };

    // migrate_on_empty_database(&schema, &pool).await;

    dumpy_data(pool.clone()).await;

    let jc = Arc::new(JsonClient::from_schema(schema, pool));

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
