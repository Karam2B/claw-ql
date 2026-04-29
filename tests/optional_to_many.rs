use sqlx::{Sqlite, query};

use claw_ql::{
    connect_in_memory::ConnectInMemory,
    extentions::common_expressions::OnInsert,
    links::{Link, set_id_mod::SetId},
    operations::{
        CollectionOutput, LinkedOutput, Operation,
        fetch_many::{FetchMany, ManyOutput},
        insert_one::InsertOne,
    },
    test_module::{self, Category, Todo, category, todo_members},
};

#[tokio::test]
async fn fetch_many() {
    let mut conn = Sqlite::connect_in_memory_2().await;

    query(
        "
        CREATE TABLE Category (
            id INTEGER PRIMARY KEY AUTOINCREMENT,
            title TEXT NOT NULL
        );

        CREATE TABLE Todo (
            id INTEGER PRIMARY KEY AUTOINCREMENT,
            title TEXT NOT NULL,
            done BOOLEAN NOT NULL,
            description TEXT,
            fk_category_def INTEGER,
            FOREIGN KEY (fk_category_def) REFERENCES Category(id)
        );

        INSERT INTO Category (title) VALUES ('category_1'), ('category_2'), ('category_3');

        INSERT INTO Todo (title, done, description, fk_category_def) VALUES 
            ('non_unique', true, 'description_1', 1),
            ('second_todo', false, 'description_2', 3),
            ('third_todo', true, 'description_3', NULL),
            ('non_unique', false, 'description_4', NULL),
            ('fifth_todo', true, 'description_5', 2),
            ('sixth_todo', false, 'description_6', 3);
    ",
    )
    .execute(&mut conn)
    .await
    .unwrap();

    let safe_op = FetchMany {
        base: test_module::todo,
        wheres: (),
        links: { <category as Link<test_module::todo>>::spec(category) },
        cursor_order_by: todo_members::title,
        cursor_first_item: Some((4, {
            todo_members::title.on_insert(String::from("non_unique"))
        })),
        limit: 2,
    };

    // any compile time error beyond this point is a bug

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
                    links: None,
                },
                LinkedOutput {
                    id: 2,
                    attributes: Todo {
                        title: "second_todo".to_string(),
                        done: false,
                        description: Some("description_2".to_string()),
                    },
                    links: Some(CollectionOutput {
                        id: 3,
                        attributes: Category {
                            title: "category_3".to_string()
                        }
                    }),
                },
            ],
            next_item: Some((6, String::from("sixth_todo"))),
        }
    );
}

#[tokio::test]
async fn insert_ops() {
    let mut conn = Sqlite::connect_in_memory_2().await;

    query(
        "
        CREATE TABLE Category (
            id INTEGER PRIMARY KEY AUTOINCREMENT,
            title TEXT NOT NULL
        );

        CREATE TABLE Todo (
            id INTEGER PRIMARY KEY AUTOINCREMENT,
            title TEXT NOT NULL,
            done BOOLEAN NOT NULL,
            description TEXT,
            fk_category_def INTEGER,
            FOREIGN KEY (fk_category_def) REFERENCES Category(id)
        );

        INSERT INTO Category (title) VALUES ('category_1');
    ",
    )
    .execute(&mut conn)
    .await
    .unwrap();

    let safe_op = InsertOne {
        base: test_module::todo,
        data: Todo {
            title: "first_todo".to_string(),
            done: true,
            description: Some("description_1".to_string()),
        },
        links: SetId {
            id: 1,
            relation: <category as Link<test_module::todo>>::spec(category),
        },
    };

    let output = Operation::<Sqlite>::exec_operation(safe_op, &mut conn).await;

    pretty_assertions::assert_eq!(
        output,
        LinkedOutput {
            id: 1,
            attributes: Todo {
                title: "first_todo".to_string(),
                done: true,
                description: Some("description_1".to_string()),
            },
            links: 1
        }
    );

    panic!("continue here")
}
