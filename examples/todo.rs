use std::marker::PhantomData;

use claw_ql::operations::select_one::{SelectOneOutput, get_one};
use claw_ql_macros::{Collection, relation};
use sqlx::SqlitePool;

#[derive(Collection, Debug, PartialEq)]
pub struct Todo {
    pub title: String,
    pub done: bool,
    pub description: Option<String>,
}

#[derive(Collection, Debug, PartialEq)]
pub struct Category {
    pub title: String,
}

#[derive(Collection, Debug, PartialEq)]
pub struct Tag {
    pub title: String,
}

relation!(optional_to_many Todo Category);
// relation!(Todo optional_toi_many Catproc-macro map is missing error entry for crate Crate(Id(401e))egory);

#[tokio::main]
async fn main() {
    let pool = SqlitePool::connect("sqlite::memory:").await.unwrap();

    panic!();

    sqlx::query(
        r#"
            CREATE TABLE Todo (
                id INTEGER PRIMARY KEY,
                title TEXT NOT NULL,
                done BOOLEAN NOT NULL,
                description TEXT,
                category_id INTEGER,
                FOREIGN KEY (category_id) REFERENCES Category (id) ON DELETE SET NULL
            );
            CREATE TABLE Tag (
                id INTEGER PRIMARY KEY,
                tag_title TEXT NOT NULL
            );
            CREATE TABLE Category (
                id INTEGER PRIMARY KEY,
                cat_title TEXT NOT NULL
            );

            CREATE TABLE TodoTag (
                todo_id INTEGER NOT NULL,
                tag_id INTEGER NOT NULL,
                PRIMARY KEY (todo_id, tag_id),
                FOREIGN KEY (todo_id) REFERENCES Todo (id) ON DELETE CASCADE,
                FOREIGN KEY (tag_id) REFERENCES Tag (id) ON DELETE CASCADE
            );

            INSERT INTO Tag (tag_title) VALUES 
                ('tag_1'), ('tag_2'), ('tag_3');
            INSERT INTO Category (cat_title) VALUES ('category_1'), ('category_2'), ('category_3');

            INSERT INTO Todo (title, done, category_id) VALUES
                ('todo_1', 1, 3),
                ('todo_2', 0, 3),
                ('todo_3', 1, NULL),
                ('todo_4', 0, 1),
                ('todo_5', 1, NULL);
    
            INSERT INTO TodoTag (todo_id, tag_id) VALUES
                (1, 1),         (1, 3),
                (2, 1), (2, 2),
                (3, 1), 
                                (4, 3),
                        (5, 2)        ;

            "#,
    )
    .execute(&pool)
    .await
    .unwrap();

    let op = get_one(PhantomData::<Todo>); //.relation(PhantomData::<Category>);

    let res = op.exec_op(pool).await;

    let expected = Some(SelectOneOutput {
        id: 0,
        attr: Todo {
            title: String::from("description"),
            done: false,
            description: Some(String::from("description")),
        },
        links: (),
    });

    pretty_assertions::assert_eq!(res, expected,)
}
