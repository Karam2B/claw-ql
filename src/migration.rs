use crate::{QueryBuilder, Schema, SqlxExtention, json_client::add_collection::SerializableAny};
use sqlx::{Database, Executor, IntoArguments, Pool, Sqlite};
use std::{marker::PhantomData, ops::Not, pin::Pin};

pub trait OnMigrate<S> {
    fn custom_migrate_statements(&self) -> Vec<String>;
}

mod on_migrate_tuple_impls {
    use super::OnMigrate;
    use crate::QueryBuilder;
    use paste::paste;
    use sqlx::{Database, Executor};

    macro_rules! implt {
        ($([$ty:ident, $part:literal]),*) => {
    #[allow(unused)]
    impl<S, $($ty,)*> OnMigrate<S> for ($($ty,)*)
    where
        S: QueryBuilder<Output = <S as Database>::Arguments<'static>>,
        $($ty: OnMigrate<S>,)*
    {
        fn custom_migrate_statements(&self) -> Vec<String> {
            let mut statements = vec![];
            $(statements.extend(paste!(self.$part).custom_migrate_statements());)*
            statements
        }
        // fn custom_migration<'e>(
        //     &self,
        //     exec: impl for<'q> Executor<'q, Database = S> + Clone,
        // ) -> impl Future<Output = ()>
        // where
        //     S: QueryBuilder,
        // {
        //     async move {$(
        //         paste!(self.$part).custom_migration(exec.clone()).await;
        //     )*}
        // }
    }
        }}

    implt!();
    implt!([R0, 0]);
    implt!([R0, 0], [R1, 1]);
    implt!([R0, 0], [R1, 1], [R2, 2]);
}

pub trait LiqOnMigrate<S> {
    fn custom_migration(&self) -> Vec<String>;
}

impl<S, T> LiqOnMigrate<S> for T
where
    T: OnMigrate<S>,
    S: Database,
    for<'c> &'c mut <S as sqlx::Database>::Connection: sqlx::Executor<'c, Database = S>,
{
    fn custom_migration(&self) -> Vec<String> {
        T::custom_migrate_statements(&self)
    }
}

pub async fn migrate_on_empty_database<C: OnMigrate<Sqlite>, L: OnMigrate<Sqlite>>(
    schema: &Schema<C, L>,
    pool: &Pool<Sqlite>,
) {
    let tables = sqlx::query_as::<_, (String,)>("SELECT name FROM sqlite_master")
        .fetch_all(&*pool)
        .await
        .unwrap();

    if tables.is_empty().not() {
        panic!("migrate_on_empty_database function should only run on empty database");
    }

    let mut v = vec![];
    v.extend(schema.collections.custom_migrate_statements());
    v.extend(schema.links.custom_migrate_statements());

    for each in v {
        sqlx::query(&each).execute(pool).await.unwrap();
    }

    sqlx::query("CREATE TABLE migration_history (version INTEGER)")
        .execute(&*pool)
        .await
        .unwrap();
    sqlx::query("INSERT INTO migration_history (version) VALUES (0)")
        .execute(&*pool)
        .await
        .unwrap();
}

#[derive(Debug)]
pub struct MigrationStep {
    pub version: i32,
    pub sql: Vec<String>,
}

trait MigrationStepTrait: SerializableAny {
    fn new(&self, from: serde_json::Value) -> Box<dyn MigrationStepTrait>;
    fn sql(&self) -> String;
    fn rust(&self) -> String; // tokenstream
    fn create_major_version(&self) -> bool;
}

// pub struct MigrationStep {
//     pub version: i32,
//     pub sql: Vec<Box<dyn MigrationStepTrait>>,
// }

pub fn create_from_scrach_migration_step<S, C: OnMigrate<S>, L: OnMigrate<S>>(
    schema: Schema<C, L>,
) -> MigrationStep {
    let mut sql = vec![];

    sql.extend(schema.collections.custom_migrate_statements());
    sql.extend(schema.links.custom_migrate_statements());

    MigrationStep { version: 0, sql }
}

pub fn create_from_scrach_migration_step_2<S, C, L>(schema: Schema<C, L>) -> MigrationStep
where
    C: OnMigrate<S>,
    L: OnMigrate<S>,
{
    let mut sql = vec![];

    sql.extend(schema.collections.custom_migrate_statements());
    sql.extend(schema.links.custom_migrate_statements());

    MigrationStep { version: 0, sql }
}

pub async fn migrate(history: Vec<MigrationStep>, pool: &Pool<Sqlite>) {
    todo!();
}

#[cfg(feature = "unstable")]
mod v2 {
    use crate::{
        QueryBuilder, SqlxExtention,
        builder_pattern::{BuildMutStep, collection, link},
    };
    use sqlx::{Database, Executor, Pool, Sqlite};
    use std::{marker::PhantomData, pin::Pin};
    fn migrations() {
        let what = Update {
            major_version: "2",
            date: "yesterday",
            items: vec![
                // generate sql statements
                // generate webassemblies binaries to deal with new data (or old data)?
                Box::new(ChangeOfType(PhantomData::<i32, i8>, "Todo::id")),
            ],
        };
    }

    #[allow(non_camel_case_types)]
    #[must_use]
    pub struct MigratorBuilder<S>(Vec<Box<dyn LiqOnMigrate<S>>>);

    impl<S> Default for MigratorBuilder<S> {
        fn default() -> Self {
            Self(Default::default())
        }
    }

    impl<N, S> BuildMutStep<collection, N> for MigratorBuilder<S>
    where
        N: OnMigrate<S> + Clone + 'static,
        S: Database,
        for<'c> &'c mut <S as Database>::Connection: Executor<'c, Database = S>,
    {
        fn build_step(&mut self, step: &N) {
            self.0.push(Box::new(step.clone()));
        }
    }

    impl<N, S> BuildMutStep<link, N> for MigratorBuilder<S>
    where
        N: OnMigrate<S> + Clone + 'static,
        S: Database,
        for<'c> &'c mut <S as Database>::Connection: Executor<'c, Database = S>,
    {
        fn build_step(&mut self, step: &N) {
            self.0.push(Box::new(step.clone()));
        }
    }

    pub trait OnMigrateDyn<S> {
        fn custom_migration<'e>(&'e self, exec: Pool<S>) -> Pin<Box<dyn Future<Output = ()> + 'e>>
        where
            S: QueryBuilder;
    }

    impl<S, T> LiqOnMigrate<S> for T
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

    #[cfg(feature = "inventory")]
    impl Migrator<sqlx::Any> {
        pub fn new_from_inventory() -> Migrator<sqlx::Any> {
            use crate::inventory::Migration;
            use inventory::iter;

            let mut migrations = vec![];

            for each in inventory::iter::<Migration> {
                migrations.push((each.obj)());
            }

            Migrator {
                migrations,
                pd: PhantomData,
            }
        }
    }

    impl<S> MigratorBuilder<S>
    where
        S: Database,
        for<'c> &'c mut <S as sqlx::Database>::Connection: sqlx::Executor<'c, Database = S>,
    {
        pub async fn migrate(&self, exec: Pool<S>)
        where
            S: crate::QueryBuilder,
        {
            for each in self.0.iter() {
                each.custom_migration(exec.clone()).await;
            }
        }
    }
}
