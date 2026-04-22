#![allow(unused)]
use claw_ql::{
    connect_in_memory::ConnectInMemory, execute::Executable, fix_executor::ExecutorTrait,
};
use sqlx::{Row, Sqlite};

#[tokio::test]
async fn main() {
    let pool = Sqlite::connect_in_memory();
    let mut conn = Sqlite::connect_in_memory_2().await;

    sqlx::query(
        "
        CREATE TABLE users (
            id INTEGER PRIMARY KEY AUTOINCREMENT,
            name TEXT NOT NULL
        );

        INSERT INTO users (name) VALUES ('John');
        INSERT INTO users (name) VALUES ('Jane');
        INSERT INTO users (name) VALUES ('Jim');
        INSERT INTO users (name) VALUES ('Jill');
        INSERT INTO users (name) VALUES ('Jack');
        INSERT INTO users (name) VALUES ('Jill');
        ",
    )
    .execute(&mut conn)
    .await
    .unwrap();

    let v = Sqlite::fetch_all(
        &mut conn,
        Executable {
            string: "SELECT * FROM users",
            arguments: Default::default(),
        },
    )
    .await
    .unwrap()
    .into_iter()
    .map(|row| row.get::<String, _>("name"))
    .collect::<Vec<_>>();

    pretty_assertions::assert_eq!(v, vec!["John", "Jane", "Jim", "Jill", "Jack", "Jill"]);
}
