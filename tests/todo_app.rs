use claw_ql::{
    prelude::Execute,
    statements::create_table_st::{CreateTableSt, header},
};
use claw_ql_macros::Collection;
use sqlx::{Sqlite, SqlitePool};

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

mod migrate {
    use claw_ql::execute::Execute;
    use claw_ql::{
        QueryBuilder,
        operations::collections::Collection,
        statements::create_table_st::{CreateTableSt, header},
    };
    use sqlx::Executor;

    pub async fn migrate<S: QueryBuilder, C: Collection<S>>(
        exec: impl for<'q> Executor<'q, Database = S>,
    ) where
        CreateTableSt<S>: Execute<S>,
    {
        let mut c = CreateTableSt::<S>::init(C::table_name(), header::create);
        C::on_migrate(&mut c);

        c.execute(exec).await.unwrap();
    }
}

#[tokio::test]
async fn main() {
    let pool = SqlitePool::connect("sqlite::memory:").await.unwrap();

    migrate::migrate::<_, Todo>(&pool).await;
    migrate::migrate::<_, Category>(&pool).await;
    migrate::migrate::<_, Tag>(&pool).await;
}
