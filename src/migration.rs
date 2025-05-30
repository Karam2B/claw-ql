use std::marker::PhantomData;

use crate::execute::Execute;
use crate::operations::collections::OnMigrate;
use crate::operations::{LinkData, Relation};
use crate::{
    QueryBuilder,
    operations::collections::Collection,
    statements::create_table_st::{CreateTableSt, header},
};
use sqlx::Executor;

pub async fn migrate_relation<S, C, T>(
    exec: impl for<'q> Executor<'q, Database = S> + Clone,
    relation: Relation<C, T>,
) where
    S: QueryBuilder,
    CreateTableSt<S>: Execute<S>,
    PhantomData<C>: Collection<S>,
    Relation<C, T>: LinkData<C, Spec: OnMigrate<S>>,
{
    let spec = relation.spec();
    spec.custom_migration(exec).await;
}

pub async fn migrate<S: QueryBuilder, C: Collection<S>>(
    collection: C,
    exec: impl for<'q> Executor<'q, Database = S>,
) where
    CreateTableSt<S>: Execute<S>,
{
    let mut c = CreateTableSt::<S>::init(header::create, collection.table_name());
    collection.on_migrate(&mut c);
    c.execute(exec).await.unwrap();
}
