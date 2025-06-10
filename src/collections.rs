use sqlx::{Database, Executor};

use crate::{QueryBuilder, prelude::stmt::InsertOneSt, statements::select_st::SelectSt};

pub trait CollectionBasic {
    fn table_name(&self) -> &'static str;
}

pub trait HasHandler {
    type Handler: Default;
}

// 
pub trait Collection<S>: Sized + Send + Sync + CollectionBasic {
    type PartailCollection;
    type Data;
    // fn on_migrate(&self, stmt: &mut CreateTableSt<Q>)
    // where
    //     Q: QueryBuilder;
    // fn on_update(
    //     stmt: &mut UpdateSt<S>,
    //     this: Self::PartailCollection,
    // ) -> Result<(), String>
    // where
    //     S: Database + SupportNamedBind;
    fn on_select(&self, stmt: &mut SelectSt<S>)
    where
        S: QueryBuilder;

    fn on_insert(&self, this: Self::Data, stmt: &mut InsertOneSt<S>)
    where
        S: sqlx::Database;

    fn members(&self) -> Vec<String>;
    // fn members_scoped(&self) -> &'static [&'static str];
    // fn table_name(&self) -> &'static str;
    fn from_row_noscope(&self, row: &S::Row) -> Self::Data
    where
        S: Database;
    fn from_row_scoped(&self, row: &S::Row) -> Self::Data
    where
        S: Database;
}

pub trait OnMigrate<S> {
    fn custom_migration<'e>(
        &self,
        exec: impl for<'q> Executor<'q, Database = S> + Clone,
    ) -> impl Future<Output = ()>
    where
        S: QueryBuilder;
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
