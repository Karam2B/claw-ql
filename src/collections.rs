use sqlx::{Database, Executor, IntoArguments};

use crate::{
    QueryBuilder,
    execute::Execute,
    statements::{
        create_table_st::{CreateTableSt, header},
        select_st::SelectSt,
    },
};

pub trait CollectionBasic {
    fn table_name(&self) -> &'static str;
}

pub trait Collection<Q>: Sized + Send + Sync + CollectionBasic {
    type PartailCollection;
    type Yeild;
    fn on_migrate(&self, stmt: &mut CreateTableSt<Q>)
    where
        Q: QueryBuilder;
    // fn on_update(
    //     stmt: &mut UpdateSt<S>,
    //     this: Self::PartailCollection,
    // ) -> Result<(), String>
    // where
    //     S: Database + SupportNamedBind;
    fn on_select(&self, stmt: &mut SelectSt<Q>)
    where
        Q: QueryBuilder;

    // fn members(&self) -> &'static [&'static str];
    // fn members_scoped(&self) -> &'static [&'static str];
    // fn table_name(&self) -> &'static str;
    fn from_row_noscope(&self, row: &Q::Row) -> Self::Yeild
    where
        Q: Database;
    fn from_row_scoped(&self, row: &Q::Row) -> Self::Yeild
    where
        Q: Database;
}

pub trait OnMigrate<S> {
    fn custom_migration<'e>(
        &self,
        exec: impl for<'q> Executor<'q, Database = S> + Clone,
    ) -> impl Future<Output = ()>
    where
        S: QueryBuilder;
}

pub struct MigrateCollection<C>(pub C);

impl<C, S> OnMigrate<S> for MigrateCollection<C>
where
    S: QueryBuilder<Output = <S as Database>::Arguments<'static>>,
    C: Collection<S>,
    for<'q> S::Arguments<'q>: IntoArguments<'q, S>,
{
    fn custom_migration<'e>(
        &self,
        exec: impl for<'q> Executor<'q, Database = S> + Clone,
    ) -> impl Future<Output = ()>
    where
        S: QueryBuilder,
    {
        async move {
            let mut c = CreateTableSt::init(header::create, self.0.table_name());
            self.0.on_migrate(&mut c);
            c.execute(exec).await.unwrap();
        }
    }
}

// #[rustfmt::skip]
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
        fn custom_migration<'e>(
            &self,
            exec: impl for<'q> Executor<'q, Database = S> + Clone,
        ) -> impl Future<Output = ()>
        where
            S: QueryBuilder,
        {
            async move {$(
                paste!(self.$part).custom_migration(exec.clone()).await;
            )*}
        }
    }
        }}

    impl<S, R0> OnMigrate<S> for (R0,)
    where
        S: QueryBuilder<Output = <S as Database>::Arguments<'static>>,
        R0: OnMigrate<S>,
    {
        fn custom_migration<'e>(
            &self,
            exec: impl for<'q> Executor<'q, Database = S> + Clone,
        ) -> impl Future<Output = ()>
        where
            S: QueryBuilder,
        {
            async move {
                self.0.custom_migration(exec.clone()).await;
            }
        }
    }
    implt!();
    implt!([R0, 0], [R1, 1]);
    implt!([R0, 0], [R1, 1], [R2, 2]);
}

pub trait Filters<Q, C>: Sync + Send {
    fn on_select(self, st: &mut SelectSt<Q>)
    where
        Q: QueryBuilder;
}

#[rustfmt::skip]
mod filters_tuple_impls {
    use super::Filters;
    use crate::{QueryBuilder, statements::select_st::SelectSt};
    use paste::paste;

        macro_rules! implt {
        ($([$ty:ident, $part:literal],)*) => {
    #[allow(unused)]
    impl
        <S,C, $($ty,)* >
    Filters<S,C>
    for
        ($($ty,)*)
    where

        S: QueryBuilder,
        $($ty:  Filters<S, C>,)*
    {
        fn on_select(self, st: &mut SelectSt<S>) {
            $(paste!(self.$part.on_select(st));)*
        }
    }
        }}

    implt!();
    implt!([R0, 0],);
    implt!([R0, 0], [R1, 1],);
}
