mod impl_connect_in_memory {
    use crate::connect_in_memory::ConnectInMemory;
    use sqlx::{Pool, Sqlite, SqlitePool};

    impl ConnectInMemory for Sqlite {
        fn connect_in_memory() -> impl Future<Output = Pool<Self>> {
            async { SqlitePool::connect("sqlite::memory:").await.unwrap() }
        }
    }
}

mod impl_database_extention {
    use crate::{database_extention::DatabaseExt, query_builder::sanitize::sanitize_by_quote};
    use sqlx::Sqlite;

    impl DatabaseExt for Sqlite {
        fn sanitize(string: &str, into: &mut String) {
            sanitize_by_quote(string, into);
        }
    }
}
