use claw_ql::ConnectInMemory;
use claw_ql::StringScope;
use sqlx::Row;
use sqlx::Sqlite;

#[tokio::test]
async fn scoped_row() {
    let pool = Sqlite::connect_in_memory().await;
    let res = sqlx::query("SELECT 1 AS scope_hi")
        .fetch_one(&pool)
        .await
        .unwrap();

    let num: i32 = res.get(StringScope(format!("scope_{}", "hi")));

    assert_eq!(1, num);
}
