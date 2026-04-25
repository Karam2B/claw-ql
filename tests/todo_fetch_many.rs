#![allow(unused)]
// 1. remove skip from json_cient
// 2. remove skip from relation_optional_to_many::impl_on_migrate

use claw_ql::connect_in_memory::ConnectInMemory;
use claw_ql::operations::{LinkedOutput, Operation, SafeOperation};
use claw_ql::row_utils::RowToJson;
use claw_ql::test_module::{Todo, todo_members};
use serde_json::json;
use sqlx::sqlite::SqliteRow;
use sqlx::{FromRow, Row, Sqlite, query};
use std::marker::PhantomData;

fn understands_lifetime<'a>(_: &'a str) {}

#[claw_ql_macros::skip]
mod test {
    use sqlx::{Sqlite, query};

    use claw_ql::{
        connect_in_memory::ConnectInMemory,
        operations::{
            LinkedOutput, Operation, SafeOperation,
            fetch_many_cursor_multi_col::{FetchMany, ManyOutput},
        },
        test_module::{self, Todo, todo_members},
    };

    #[tokio::test]
    async fn main() {
        let mut conn = Sqlite::connect_in_memory_2().await;

        query(
            "
        CREATE TABLE Todo (
            id INTEGER PRIMARY KEY AUTOINCREMENT,
            title TEXT NOT NULL,
            done BOOLEAN NOT NULL,
            description TEXT
        );

        INSERT INTO Todo (title, done, description) VALUES 
            ('non_unique', true, 'description_1'),
            ('second_todo', false, 'description_2'),
            ('third_todo', true, 'description_3'),
            ('non_unique', false, 'description_4'),
            ('fifth_todo', true, 'description_5'),
            ('sixth_todo', false, 'description_6');
    ",
        )
        .execute(&mut conn)
        .await
        .unwrap();

        let safe_op = FetchMany {
            base: test_module::todo,
            wheres: (),
            links: (),
            cursor_order_by: todo_members::title,
            cursor_first_item: Some((4, String::from("non_unique"))),
            limit: 2,
        }
        .safety_check()
        .unwrap();

        let output = Operation::<Sqlite>::exec_operation(safe_op, &mut conn).await;

        pretty_assertions::assert_eq!(
            output,
            ManyOutput {
                items: vec![
                    LinkedOutput {
                        id: 4,
                        attributes: Todo {
                            title: "non_unique".to_string(),
                            done: false,
                            description: Some("description_4".to_string()),
                        },
                        links: (),
                    },
                    LinkedOutput {
                        id: 2,
                        attributes: Todo {
                            title: "second_todo".to_string(),
                            done: false,
                            description: Some("description_2".to_string()),
                        },
                        links: (),
                    },
                ],
                next_item: Some((6, String::from("sixth_todo"))),
            }
        );
    }
}
