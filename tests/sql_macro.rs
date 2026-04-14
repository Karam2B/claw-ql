#![allow(unused)]
use claw_ql::connect_in_memory::ConnectInMemory;
use claw_ql::execute::Executable;
use claw_ql::on_migrate::OnMigrate;
use claw_ql::operations::{LinkedOutput, Operation};

use claw_ql::prelude::sql::ExpressionAsOperation;
use claw_ql_macros::sql;
use sqlx::sqlite::SqliteConnectOptions;
use sqlx::{ConnectOptions, Connection, Database, Sqlite, SqliteConnection};

async fn exec_operation_gen<'c, S, E>(e: E)
where
    E: sqlx::Acquire<'c, Database = S>,
    S: Database,
    for<'m> &'m mut <S as sqlx::Database>::Connection: sqlx::Executor<'m, Database = S>,
{
    let mut exec = e.acquire().await.unwrap();
    sqlx::Executor::execute(
        &mut *exec,
        Executable {
            string: "sdf",
            arguments: Default::default(),
        },
    );
}

#[tokio::test]
#[allow(unused)]
#[claw_ql_macros::skip]
async fn main() {
    let mut pool = Sqlite::connect_in_memory_2().await;

    let mut m = SqliteConnectOptions::new()
        .in_memory(true)
        .connect()
        .await
        .unwrap();

    use claw_ql::expressions::col_eq;
    use claw_ql::links::relation_optional_to_many::OptionalToMany;
    use claw_ql::links::set_new_mod::set_new;
    use claw_ql::prelude::sql::*;
    use claw_ql::test_module::{Category, Todo, category, todo, todo_members};

    // Operation::exec_operation(expression_to_operation(todo.statments()), &mut pool).await;

    sql!(MIGRATE todo).await;
    sql!(MIGRATE category).await;
    sql!(MIGRATE OptionalToMany {
        from: todo,
        to: category,
        foriegn_key: "category_id".to_string()
    })
    .await;

    sqlx::query(
        "
    INSERT INTO Todo (title, done, description) VALUES ('first_todo', false, NULL)
    ",
    )
    .execute(&mut pool)
    .await
    .unwrap();
    // sql!(
    //     INSERT Todo { title:"first_todo".to_string(), done: false, description: None }
    //         LINK set_new(Category { title: "cat_1".to_string() })
    // )
    // .await;

    // let res = Operation::<Sqlite>::exec_operation(
    //     FetchOne {
    //         base: todo,
    //         wheres: (),
    //         links: category,
    //     },
    //     &mut pool,
    // )
    // .await;
    // let result = sql!(
    //     SELECT FROM todo t
    //     LINK category
    //     WHERE t.title.col_eq("first_todo".to_string())
    // )
    // .await;

    // pretty_assertions::assert_eq!(
    //     result,
    //     Some(LinkedOutput {
    //         id: 0,
    //         attributes: Todo {
    //             title: "first_todo".to_string(),
    //             done: false,
    //             description: None
    //         },
    //         links: (None,),
    //         // links: (Some(CollectionOutput {
    //         //     id: 1,
    //         //     attributes: Category {
    //         //         title: "cat_1".to_string()
    //         //     }
    //         // }),),
    //     })
    // );
}
