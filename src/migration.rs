use crate::{
    QueryBuilder, SqlxExtention,
    builder_pattern::{BuildMutStep, collection, link},
};
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
#[must_use]
pub struct MigratorBuilder<S>(Vec<Box<dyn OnMigrateDyn<S>>>);

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
// mod v0 {
//     use core::hash;
//     use std::collections::HashSet;
//     #[derive(Default)]
//     struct Schema {
//         tables: HashSet<Table>,
//     }
//
//     struct Table {
//         name: String,
//         fields: Vec<String>,
//     }
//
//     impl PartialEq for Table {
//         fn eq(&self, other: &Self) -> bool {
//             self.name == other.name
//         }
//     }
//
//     impl Eq for Table {}
//
//     impl hash::Hash for Table {
//         fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
//             self.name.hash(state)
//         }
//     }
// }
//
// pub mod migrations {
//     use serde::{Serialize, de::DeserializeOwned};
//
//     use sqlx::{Database, Pool};
//
//     use crate::{
//         SqlxExtention,
//         builder_pattern::{
//             AddCollection, BuilderPattern, Finish, NewContext,
//             extend_builder_patter::{AddCollectionMut, MutHasContext},
//         },
//         collections::CollectionBasic,
//     };
//     use std::{
//         collections::{HashMap, HashSet},
//         fmt::Display,
//     };
//
//     use std::any::Any;
//
//     pub trait SerializableTypeId: 'static {
//         fn hopefully_unique_str(&self) -> &'static str;
//         #[cfg(feature = "serde")]
//         fn to_json(&self) -> serde_json::Value;
//         #[cfg(feature = "serde")]
//         fn from_value() -> Box<dyn SerializableTypeId>
//         where
//             Self: Sized + SerializableTypeId2;
//     }
//
//     pub trait SerializableTypeId2 {
//         fn from_value() -> Box<dyn SerializableTypeId>;
//     }
//
//     #[rustfmt::skip]
//     pub enum Issue {
//         ColumnExist { table: String, col: String, ty: Box<dyn SerializableTypeId> },
//         ColumnNeedCreating { table: String, col: String, ty: Box<dyn SerializableTypeId> },
//         TableHasConstraint { table: String, ident: Option<String> , ty: Box<dyn SerializableTypeId> },
//         TableNeedConstraint { table: String, ident: Option<String>, ty: Box<dyn SerializableTypeId> }
//     }
//
//     pub struct Migrator2<S> {
//         pub s: S,
//         pub steps: Vec<Box<dyn MigrationStep<S>>>,
//         pub issues: Vec<Issue>,
//     }
//
//     impl<S> MutHasContext for Migrator2<S> {
//         type Context = Self;
//         type Result = Self;
//         fn new_context(self) -> Self::Context {
//             self
//         }
//         fn finish(ctx: Self::Context) -> Self::Result {
//             ctx
//         }
//     }
//
//     impl<S, E: CollectionBasic> AddCollectionMut<E> for Migrator2<S> {
//         fn build_component(collection: &E, ctx: &mut Self::Context) {
//             // ctx.steps.push
//             // ctx.steps.push(collection.clone())
//         }
//     }
//
//     impl<S: SqlxExtention> Default for Migrator2<S> {
//         fn default() -> Self {
//             Migrator2 {
//                 s: S::phantom(),
//                 steps: Default::default(),
//                 issues: Default::default(),
//             }
//         }
//     }
//
//     impl<S> Migrator2<S>
//     where
//         S: Database,
//     {
//         pub async fn migrate(self, pool: Pool<S>) -> Result<(), Vec<Issue>> {
//             Ok(())
//         }
//     }
//
//     // pub struct Ctx;
//
//     // #[must_use]
//     // pub struct Migrator<S>(S);
//
//     // impl<S: SqlxExtention> MigratorManager<S> {
//     //     pub fn builder() -> MigratorManager<S> {
//     //         MigratorManager {  s: S::phantom() }
//     //     }
//     // }
//
//     // // impl MResult
//     //
//     // impl NewContext for MigrationM {
//     //     type Context = Ctx;
//     //     fn new_context(self) -> Self::Context {
//     //         Ctx
//     //     }
//     // }
//     // impl MutHasContext for MigrationM {
//     //     type Context = Ctx;
//     //     fn new_context(self) -> Self::Context {
//     //         Ctx
//     //     }
//     //     type Result = Migrator;
//     //     fn finish(ctx: Self::Context) -> Self::Result {
//     //         Migrator
//     //     }
//     // }
//     //
//     // impl<E> AddCollectionMut<E> for MigrationM {
//     //     fn build_component(collection: &E, ctx: &mut Self::Context) {}
//     // }
//
//     // enum Components {
//     //     Table { fields: HashSet<String> },
//     //     Index,
//     // }
//     //
//     // struct Table {
//     //     name: String,
//     // }
//     //
//     // struct Schema {
//     //     tables: HashSet<Table>,
//     // }
//
//     // fn create_schema(stpl T) -> Schema {
//     //     todo!()
//     // }
//
//     fn checkout_issues(schema: HashMap<String, Vec<String>>) -> Vec<Issue> {
//         vec![]
//     }
//
//     // ## `fn up` and `fn down`, but `fn change` might be enough
//     //
//     // ## handle data migration (gracefully!)
//     //
//     // fn data_migration(from: String) -> bool {
//     //     from == "" || from.starts_with("delete").not()
//     // }
//     //
//     // `impl Collection`s might be not enough!
//     //
//     // fn migrate(from: impl Collections, to: impl Collections)
//     // migrate(from_empty, s.col(todo).col(tag))
//     // migrate((s.col(todo_old).col(tag_old), track), s.col(todo).col(tag))
//
//     pub trait MigrationStep<S> {
//         fn clone(&self) -> Box<dyn MigrationStep<S>>;
//     }
// }
