use sqlx::{Pool, Sqlite, SqlitePool};

use crate::ConnectInMemory;

impl ConnectInMemory for Sqlite {
    fn connect_in_memory() -> impl Future<Output = Pool<Self>> {
        async { SqlitePool::connect("sqlite::memory:").await.unwrap() }
    }
}
