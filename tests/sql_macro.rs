use claw_ql::connect_in_memory::ConnectInMemory;
use claw_ql::operations::LinkedOutput;

use claw_ql_macros::sql;
use sqlx::Sqlite;

#[tokio::test]
#[allow(unused)]
async fn main() {
    let pool = Sqlite::connect_in_memory().await;

    use claw_ql::expressions::col_eq;
    use claw_ql::links::relation_optional_to_many::optional_to_many;
    use claw_ql::links::set_new_mod::set_new;
    use claw_ql::test_module::{Category, Todo, category, todo, todo_members};

    sql!(MIGRATE todo).await;
    sql!(MIGRATE category).await;
    sql!(MIGRATE optional_to_many {
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
    .execute(&pool)
    .await
    .unwrap();
    // sql!(
    //     INSERT Todo { title:"first_todo".to_string(), done: false, description: None }
    //         LINK set_new(Category { title: "cat_1".to_string() })
    // )
    // .await;

    let result = sql!(
        SELECT FROM todo t
        LINK category
        WHERE t.title.col_eq("first_todo".to_string())
    )
    .await;

    pretty_assertions::assert_eq!(
        result,
        Some(LinkedOutput {
            id: 0,
            attributes: Todo {
                title: "first_todo".to_string(),
                done: false,
                description: None
            },
            links: (None,),
            // links: (Some(CollectionOutput {
            //     id: 1,
            //     attributes: Category {
            //         title: "cat_1".to_string()
            //     }
            // }),),
        })
    );
}
