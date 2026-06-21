mod impl_connect_in_memory {
    use crate::connect_in_memory::ConnectInMemory;
    use sqlx::{ConnectOptions, Pool, Sqlite, SqlitePool, sqlite::SqliteConnectOptions};

    impl ConnectInMemory for Sqlite {
        async fn in_memory_connection() -> <Self as sqlx::Database>::Connection {
            SqliteConnectOptions::new()
                .in_memory(true)
                .connect()
                .await
                .unwrap()
        }
        fn in_memory_pool() -> impl Future<Output = Pool<Self>> {
            async { SqlitePool::connect("sqlite::memory:").await.unwrap() }
        }
    }
}

mod impl_database_extention {
    use crate::{
        database_extention::DatabaseExt,
        query_builder::{Expression, OpExpression},
    };
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
        type IdExpression = IdExpression;
        fn id_on_create_table_expression() -> Self::IdExpression {
            IdExpression
        }
    }

    pub struct IdExpression;

    impl OpExpression for IdExpression {}
    impl<'q> Expression<'q, Sqlite> for IdExpression {
        fn expression(
            self,
            ctx: &mut crate::prelude::macro_derive_collection::StatementBuilder<'q, Sqlite>,
        ) where
            Sqlite: DatabaseExt,
        {
            ctx.syntax(&"\"id\" INTEGER PRIMARY KEY AUTOINCREMENT");
        }
    }
}
