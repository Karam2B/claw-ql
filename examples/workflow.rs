use claw_ql::prelude::*;
use sqlx::SqlitePool;

#[tokio::main]
async fn main() {
    let conn = SqlitePool::connect("sqlite://:memory:").await.unwrap();
    let mut hi = stmt::SelectSt::init("Users");

    hi.select(col("ehlo"));

    let op = hi.fetch_one(&conn, |r| Ok(r)).await.unwrap();
}

