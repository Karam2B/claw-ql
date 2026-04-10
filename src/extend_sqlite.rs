mod impl_connect_in_memory {
    use crate::ConnectInMemory;
    use sqlx::{Pool, Sqlite, SqlitePool};

    impl ConnectInMemory for Sqlite {
        fn connect_in_memory() -> impl Future<Output = Pool<Self>> {
            async { SqlitePool::connect("sqlite::memory:").await.unwrap() }
        }
    }
}

mod impl_database_extention {
    use crate::DatabaseExt;
    use sqlx::Sqlite;

    impl DatabaseExt for Sqlite {}
}
