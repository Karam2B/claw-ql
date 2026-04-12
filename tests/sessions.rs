#![allow(unused)]
#![deny(unused_must_use)]

use claw_ql::connect_in_memory::ConnectInMemory;
use sqlx::{FromRow, Row, Sqlite};

#[tokio::test]
async fn main() {
    let pool = Sqlite::connect_in_memory().await;

    sqlx::query(
        "
        CREATE TABLE IF NOT EXISTS Category (
            id INTEGER PRIMARY KEY AUTOINCREMENT,
            title TEXT NOT NULL
        );
        INSERT INTO Category (title) VALUES ('cat_1'), ('cat_2');
        ",
    )
    .execute(&pool)
    .await
    .unwrap();

    let mut s: sqlx::Transaction<'_, Sqlite> = pool.begin().await.unwrap();

    sqlx::query(
        "
        INSERT INTO Category (title) VALUES ('cat_3');
        ",
    )
    .execute(s.as_mut())
    .await
    .unwrap();

    s.rollback().await.unwrap();

    let r = sqlx::query(
        "
        SELECT title FROM Category;
        ",
    )
    .fetch_all(&pool)
    .await
    .unwrap();

    let rows: Vec<String> = r.into_iter().map(|r| r.get(0)).collect();

    pretty_assertions::assert_eq!(rows, vec!["cat_1", "cat_2"]);
}
