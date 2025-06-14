use std::{marker::PhantomData, pin::Pin};
use crate::{QueryBuilder, collections::OnMigrate};
use sqlx::{Database, Pool, Sqlite};
use crate::builder_pattern::{AddCollection, AddLink, BuildContext, Finish};

#[allow(non_camel_case_types)]
pub struct to_migrate<S>(pub S);

impl Clone for to_migrate<Sqlite> {
    fn clone(&self) -> Self {
        to_migrate(Sqlite)
    }
}

impl<S> BuildContext for to_migrate<S> {
    fn init_context(&self) -> Self::Context {
        Default::default()
    }
    type Context = Vec<Box<dyn OnMigrateDyn<S>>>;
}

impl<T, N, S> AddCollection<T, N> for to_migrate<S>
where
    N: OnMigrate<S> + Clone + 'static,
    S: Database,
    for<'c> &'c mut <S as sqlx::Database>::Connection: sqlx::Executor<'c, Database = S>,
{
    fn add_col(next: &N, ctx: &mut Self::Context) {
        let n = next.clone();
        ctx.push(Box::new(n))
    }
}

impl<T, N, S> AddLink<T, N> for to_migrate<S>
where
    N: OnMigrate<S> + Clone + 'static,
    S: Database,
    for<'c> &'c mut <S as sqlx::Database>::Connection: sqlx::Executor<'c, Database = S>,
{
    fn add_link(next: &N, ctx: &mut Self::Context) {
        let next = next.clone();
        ctx.push(Box::new(next))
    }
}

impl<C, S> Finish<C> for to_migrate<S> {
    type Result = MigrateResult<S>;
    fn finish(self, ctx: Self::Context) -> Self::Result {
        MigrateResult {
            pd: PhantomData,
            collections: ctx,
        }
    }
}

pub trait OnMigrateDyn<S> {
    fn custom_migration<'e>(&'e self, exec: Pool<S>) -> Pin<Box<dyn Future<Output = ()> + 'e>>
    where
        S: QueryBuilder;
}

impl<S, T> OnMigrateDyn<S> for T
where
    T: OnMigrate<S>,
    S: Database,
    for<'c> &'c mut <S as sqlx::Database>::Connection: sqlx::Executor<'c, Database = S>,
{
    fn custom_migration<'e>(&'e self, exec: Pool<S>) -> Pin<Box<dyn Future<Output = ()> + 'e>>
    where
        S: QueryBuilder,
    {
        let pool = exec.clone();
        Box::pin(async move {
            self.custom_migration(&pool).await;
        })
    }
}

#[must_use]
pub struct MigrateResult<S> {
    collections: Vec<Box<dyn OnMigrateDyn<S>>>,
    pd: PhantomData<S>,
}

impl<S> MigrateResult<S>
where
    S: Database,
    for<'c> &'c mut <S as sqlx::Database>::Connection: sqlx::Executor<'c, Database = S>,
{
    pub async fn migrate<'e>(&self, exec: Pool<S>)
    where
        S: crate::QueryBuilder,
    {
        for each in self.collections.iter() {
            each.custom_migration(exec.clone()).await;
        }
    }
}
