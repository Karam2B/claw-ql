#![allow(unexpected_cfgs)]
#![allow(unused)]
#![warn(unused_must_use)]
#![allow(non_camel_case_types)]
#![allow(non_upper_case_globals)]
#![deny(unsafe_code)]

#[track_caller]
fn asert_in_json<T1: Serialize + 'static, T2: Serialize + 'static>(t1: T1, t2: T2) {
    let t1 = serde_json::to_value(t1).unwrap();
    let t2 = serde_json::to_value(t2).unwrap();
    pretty_assertions::assert_eq!(t1, t2);
    if t1.type_id() != t2.type_id() {
        panic!("two types equal in value and mismatch in type")
    }
}

use std::{any::Any, collections::HashMap, ffi::FromVecWithNulError, mem, sync::Arc};

use claw_ql::{
    ConnectInMemory, DatabaseExt, Expression, QueryBuilder,
    collections::{Collection, Member, MemberBasic},
    execute::Executable,
    expressions::{col, col_def_for_collection_member, col_eq},
    functional_expr::boxed_expr,
    json_client::{
        FetchOneInput, JsonClient, json_collection::JsonCollection, json_link::JsonLink,
    },
    links::{
        Link,
        relation_optional_to_many::{impl_dynamic_link::OptionalToManyLinks, optional_to_many},
    },
    on_migrate::OnMigrate,
    operations::{
        Operation,
        fetch_one::{FetchOne, execute_fetch_one},
    },
    run_expression,
    statements::{CreateTable, create_if_not_exist, create_table},
};
use claw_ql::{ZeroOrMoreExpressions, use_executor};
use futures::{StreamExt, TryStreamExt};
use serde::{Deserialize, Serialize};
use serde_json::{from_value, json, to_value};
use sqlx::{Database, Executor, Pool, Sqlite};

use crate::todo_members::description;

#[derive(
    claw_ql_macros::Collection,
    claw_ql_macros::OnMigrate,
    Serialize,
    Deserialize,
    claw_ql_macros::FromRowAlias,
)]
pub struct Todo {
    pub title: String,
    pub done: bool,
    pub description: Option<String>,
}

#[derive(
    claw_ql_macros::Collection,
    claw_ql_macros::OnMigrate,
    Serialize,
    Deserialize,
    claw_ql_macros::FromRowAlias,
)]
pub struct Category {
    pub title: String,
}

impl Link<todo> for category {
    type Spec = optional_to_many<String, todo, category>;
    fn spec(self, _: &todo) -> Self::Spec {
        optional_to_many {
            foriegn_key: String::from("category_id"),
            from: todo,
            to: self,
        }
    }
}

async fn execute_statements<Db, Statements>(
    pool: &Pool<Db>,
    s: Statements,
) -> Result<(), sqlx::Error>
where
    Db: DatabaseExt,
    Statements: ZeroOrMoreExpressions<'static, Db>,
    for<'c> &'c mut <Db as sqlx::Database>::Connection: Executor<'c, Database = Db>,
{
    static HI: () = ();
    let mut b = QueryBuilder::<'_, Db>::default();
    s.expression("", " ", &mut b);
    // let stmt = b.stmt.clone();

    let s = use_executor!(fetch_optional(pool, b));

    Ok(())
}

#[tokio::test]
async fn main() {
    let pool = Sqlite::connect_in_memory().await;

    execute_statements(
        &pool,
        vec![
            boxed_expr(todo.statments()),
            boxed_expr(category.statments()),
            boxed_expr(
                optional_to_many {
                    from: todo,
                    to: category,
                    foriegn_key: String::from("category_id"),
                }
                .statments(),
            ),
        ],
    )
    .await
    .unwrap();

    Executor::execute(
        &pool,
        Executable {
            string: "
                INSERT INTO Category (id, title)
                VALUES
                    (3, 'cat_1');

                INSERT INTO Todo (id, title, done, description, category_id)
                VALUES
                    (7, 'second_todo', false, NULL, NULL),
                    (6, 'first_todo', true, 'description_1', 3)
                ;
            ",
            arguments: Default::default(),
            db: std::marker::PhantomData,
        },
    )
    .await
    .expect("bug: hardcoded dumpy insert statments");

    // sql!(
    //     SELECT FROM todo
    //     WHERE todo.title.eq("first_todo")
    //     LINK category
    // );

    let op = FetchOne {
        base: todo,
        wheres: (col(todo_members::title).eq("first_todo")),
        link: (category,),
    };

    let out = op.exec(pool).await;

    asert_in_json(
        out,
        json!({
            "attributes": {
                "title": "first_todo",
                "done": true,
                "description": "description_1"
            },
            "id": 6,
            "link": [{
                "attributes": { "title": "cat_1" },
                "id": 3,
            }]
        }),
    );
}

#[tokio::test]
async fn json_client() {
    let pool = Sqlite::connect_in_memory().await;

    execute_statements(
        &pool,
        vec![
            boxed_expr(todo.statments()),
            boxed_expr(category.statments()),
            boxed_expr(
                optional_to_many {
                    from: todo,
                    to: category,
                    foriegn_key: String::from("category_id"),
                }
                .statments(),
            ),
        ],
    )
    .await
    .unwrap();

    Executor::execute(
        &pool,
        Executable {
            string: "
                INSERT INTO Category (id, title)
                VALUES
                    (3, 'cat_1');

                INSERT INTO Todo (id, title, done, description, category_id)
                VALUES
                    (6, 'first_todo', true, 'description_1', 3),
                    (7, 'second_todo', false, NULL, NULL)
                ;
            ",
            arguments: Default::default(),
            db: std::marker::PhantomData,
        },
    )
    .await
    .expect("bug: hardcoded dumpy insert statments");

    let mut jc = JsonClient {
        collections: HashMap::from([(
            "todo".to_string(),
            Arc::new(todo) as Arc<dyn JsonCollection<Sqlite>>,
        )]),
        links: HashMap::from([(
            "optional_to_many".to_string(),
            Arc::new(OptionalToManyLinks {
                all: vec![optional_to_many {
                    foriegn_key: "category_id".to_string(),
                    from: Arc::new(todo) as Arc<dyn JsonCollection<Sqlite>>,
                    to: Arc::new(category) as Arc<dyn JsonCollection<Sqlite>>,
                }],
            }) as Arc<dyn JsonLink<Sqlite>>,
        )]),
        pool,
    };

    let out = jc
        .fetch_one(FetchOneInput {
            base: String::from("todo"),
            wheres: vec![],
            link: vec![
                from_value(json!({
                       "id": "category_id",
                       "ty": "optional_to_many",
                       "to": "category",
                   }
                ))
                .unwrap(),
            ],
        })
        .await
        .unwrap();

    asert_in_json(
        out,
        json!({
            "attributes": {
                "title": "first_todo",
                "done": true,
                "description": "description_1"
            },
            "id": 6,
            "link": [{
                "attributes": { "title": "cat_1" },
                "id": 3,
            }]
        }),
    );
}
