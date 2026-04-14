mod impl_connect_in_memory {
    use crate::connect_in_memory::ConnectInMemory;
    use sqlx::{ConnectOptions, Pool, Sqlite, SqlitePool, sqlite::SqliteConnectOptions};

    impl ConnectInMemory for Sqlite {
        async fn connect_in_memory_2() -> <Self as sqlx::Database>::Connection {
            SqliteConnectOptions::new()
                .in_memory(true)
                .connect()
                .await
                .unwrap()
        }
        fn connect_in_memory() -> impl Future<Output = Pool<Self>> {
            async { SqlitePool::connect("sqlite::memory:").await.unwrap() }
        }
    }
}

mod impl_database_extention {
    use crate::database_extention::DatabaseExt;
    use sqlx::Sqlite;

    impl DatabaseExt for Sqlite {
        fn sanitize(string: &str, into: &mut String) {
            let mut s = string.chars();
            while let Some(next) = s.next() {
                match next {
                    '"' => {
                        into.push(next);
                        into.push('"');
                    }
                    '\'' => {
                        into.push(next);
                        into.push('\'');
                    }
                    '\\' => {
                        into.push(next);
                        into.push('\\');
                    }
                    n => into.push(n),
                }
            }
        }
        fn sanitize_start(into: &mut String) {
            into.push('"');
        }
        fn sanitize_end(into: &mut String) {
            into.push('"');
        }
    }
}
