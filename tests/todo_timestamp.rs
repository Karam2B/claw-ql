use claw_ql::{
    clear_double_space::ClearDoubleSpace,
    connect_in_memory::ConnectInMemory,
    links::timestamp::{Timestamp, TimestampOutput},
    on_migrate::OnMigrate,
    operations::{
        LinkedOutput, Operation,
        fetch_many::{FetchMany, ManyOutput, SortOnlyById},
    },
    query_builder::{
        StatementBuilder,
        functional_expr::{ManyImplExpression, ManyPossible},
    },
    test_module::{self, Todo},
};
use sqlx::Sqlite;

#[tokio::test]
async fn test_fetch_many() {
    let mut conn = Sqlite::connect_in_memory_2().await;

    Timestamp {
        collection: test_module::todo,
    }
    .statments();

    sqlx::query(
        "
    CREATE TABLE Todo (
        id INTEGER PRIMARY KEY AUTOINCREMENT,
        title TEXT NOT NULL,
        done BOOLEAN NOT NULL,
        description TEXT,
        created_at TEXT NOT NULL,
        updated_at TEXT NOT NULL
    );

    INSERT INTO Todo (title, done, description, created_at, updated_at) VALUES 
        ('Test', true, 'Test description', 'created_at_test', 'updated_at_test'),
        ('Test 2', false, 'Test description 2', 'created_at_test_2', 'updated_at_test_2'),
        ('Test 3', true, 'Test description 3', 'created_at_test_3', 'updated_at_test_3');

    ",
    )
    .execute(&mut conn)
    .await
    .unwrap();

    let output = Operation::<Sqlite>::exec_operation(
        FetchMany {
            base: test_module::todo,
            wheres: (),
            links: Timestamp {
                collection: test_module::todo,
            },
            cursor_order_by: SortOnlyById,
            cursor_first_item: None,
            limit: 10,
        },
        &mut conn,
    )
    .await;

    pretty_assertions::assert_eq!(
        output,
        ManyOutput {
            items: vec![
                LinkedOutput {
                    id: 1,
                    attributes: Todo {
                        title: "Test".to_string(),
                        done: true,
                        description: Some("Test description".to_string()),
                    },
                    links: TimestampOutput {
                        created_at: "created_at_test".to_string(),
                        updated_at: "updated_at_test".to_string(),
                    }
                },
                LinkedOutput {
                    id: 2,
                    attributes: Todo {
                        title: "Test 2".to_string(),
                        done: false,
                        description: Some("Test description 2".to_string()),
                    },
                    links: TimestampOutput {
                        created_at: "created_at_test_2".to_string(),
                        updated_at: "updated_at_test_2".to_string(),
                    }
                },
                LinkedOutput {
                    id: 3,
                    attributes: Todo {
                        title: "Test 3".to_string(),
                        done: true,
                        description: Some("Test description 3".to_string()),
                    },
                    links: TimestampOutput {
                        created_at: "created_at_test_3".to_string(),
                        updated_at: "updated_at_test_3".to_string(),
                    }
                },
            ],
            next_item: None
        }
    );
}

#[test]
fn on_migrate_for_sqlite() {
    let many = ManyImplExpression::new(
        ManyPossible(
            Timestamp {
                collection: test_module::todo,
            }
            .statments(),
        ),
        "",
        " ",
    )
    .unwrap();
    let s = StatementBuilder::<Sqlite>::new(many);

    pretty_assertions::assert_eq!(
        s.stmt(),
        ClearDoubleSpace::new(
            "
ALTER TABLE `Todo` ADD COLUMN `created_at` TEXT NOT NULL DEFAULT CURRENT_TIMESTAMP; 
ALTER TABLE `Todo` ADD COLUMN `updated_at` TEXT NOT NULL DEFAULT CURRENT_TIMESTAMP; 
CREATE TRIGGER `update_timestamp` AFTER UPDATE ON `Todo` BEGIN 
UPDATE {table} SET `updated_at` = CURRENT_TIMESTAMP WHERE `id` = NEW.`id`; END;"
                .trim()
                .chars()
                .map(|c| if c == '\n' { ' ' } else { c })
                .map(|c| if c == '`' { '"' } else { c })
        )
        .consume()
    );
}
