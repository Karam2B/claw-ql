use crate::QueryBuilder;
use crate::builder_pattern::{AddCollection, AddLink, Finish, InitializeContext};
use sqlx::{Database, Executor, Pool, Sqlite};
use std::{marker::PhantomData, pin::Pin};

pub trait OnMigrate<S> {
    fn custom_migration<'e>(
        &self,
        exec: impl for<'q> Executor<'q, Database = S> + Clone,
    ) -> impl Future<Output = ()>
    where
        S: QueryBuilder;
}

#[allow(non_camel_case_types)]
pub struct to_migrate<S>(pub S);



impl Clone for to_migrate<Sqlite> {
    fn clone(&self) -> Self {
        to_migrate(Sqlite)
    }
}

type ThisContext<S> = Vec<Box<dyn OnMigrateDyn<S>>>;

impl<S> InitializeContext for to_migrate<S> {
    type Context = ThisContext<S>;
    fn initialize_context(self) -> Self::Context {
        Default::default()
    }
}

impl<N, S> AddCollection<N> for to_migrate<S>
where
    N: OnMigrate<S> + Clone + 'static,
    S: Database,
    for<'c> &'c mut <S as sqlx::Database>::Connection: sqlx::Executor<'c, Database = S>,
{
    type This = to_migrate<S>;
    type Context = ThisContext<S>;
    type NextContext = Self::Context;
    fn build_component(collection: &N, mut ctx: Self::Context) -> Self::NextContext {
        ctx.push(Box::new(collection.clone()));
        ctx
    }
}

impl<N, S> AddLink<N> for to_migrate<S>
where
    N: OnMigrate<S> + Clone + 'static,
    S: Database,
    for<'c> &'c mut <S as sqlx::Database>::Connection: sqlx::Executor<'c, Database = S>,
{
    type This = to_migrate<S>;
    type Context = ThisContext<S>;
    type NextContext = Self::Context;
    fn build_component(link: &N, mut ctx: Self::Context) -> Self::NextContext {
        ctx.push(Box::new(link.clone()));
        ctx
    }
}

impl<S> Finish for to_migrate<S> {
    type Result = MigrateResult<S>;
    type Context = ThisContext<S>;

    fn build_component(ctx: Self::Context) -> Self::Result {
        MigrateResult {
            pd: PhantomData,
            migrations: ctx,
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
    migrations: Vec<Box<dyn OnMigrateDyn<S>>>,
    pd: PhantomData<S>,
}

#[cfg(feature = "inventory")]
impl MigrateResult<sqlx::Any> {
    pub fn new_from_inventory() -> MigrateResult<sqlx::Any> {
        use crate::inventory::Migration;
        use inventory::iter;

        let mut migrations = vec![];

        for each in inventory::iter::<Migration> {
            migrations.push((each.obj)());
        }

        MigrateResult {
            migrations,
            pd: PhantomData,
        }
    }
}

impl<S> MigrateResult<S>
where
    S: Database,
    for<'c> &'c mut <S as sqlx::Database>::Connection: sqlx::Executor<'c, Database = S>,
{
    pub async fn migrate(&self, exec: Pool<S>)
    where
        S: crate::QueryBuilder,
    {
        for each in self.migrations.iter() {
            each.custom_migration(exec.clone()).await;
        }
    }
}

