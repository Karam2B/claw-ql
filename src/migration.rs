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

pub async fn migrate_relation<S, C, T>(exec: impl for<'q> Executor<'q, Database = S> + Clone)
where
    S: QueryBuilder,
    CreateTableSt<S>: Execute<S>,
    C: Collection<S>,
    Relation<T>: LinkData<C, Spec: OnMigrate<S>>,
{
    let mut c = CreateTableSt::<S>::init(header::create, C::table_name());
    C::on_migrate(&mut c);
    let spec = Relation(PhantomData::<T>).spec();
    spec.custom_migration(exec).await;
}

pub async fn migrate<S: QueryBuilder, C: Collection<S>>(
    exec: impl for<'q> Executor<'q, Database = S>,
) where
    CreateTableSt<S>: Execute<S>,
{
    let mut c = CreateTableSt::<S>::init(header::create, C::table_name());
    C::on_migrate(&mut c);
    c.execute(exec).await.unwrap();
}
