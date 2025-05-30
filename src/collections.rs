use sqlx::{Database, Executor, IntoArguments};

use crate::{
    QueryBuilder,
    execute::Execute,
    statements::{
        create_table_st::{CreateTableSt, header},
        select_st::SelectSt,
    },
};

pub trait Collection<Q>: Sized + Send + Sync {
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

    fn members(&self) -> &'static [&'static str];
    fn members_scoped(&self) -> &'static [&'static str];
    fn table_name(&self) -> &'static str;
    fn from_row_noscope(&self, row: &Q::Row) -> Self::Yeild
    where
        Q: Database;
    fn from_row_scoped(&self, row: &Q::Row) -> Self::Yeild
    where
        Q: Database;
}

pub trait OnMigrate<S> {
    fn custom_migration<'e>(
        self,
        exec: impl for<'q> Executor<'q, Database = S> + Clone,
    ) -> impl Future<Output = ()>
    where
        S: QueryBuilder;
}

impl<C, S> OnMigrate<S> for C
where
    S: QueryBuilder<Output = <S as Database>::Arguments<'static>>,
    C: Collection<S>,
    for<'q> S::Arguments<'q>: IntoArguments<'q, S>,
{
    fn custom_migration<'e>(
        self,
        exec: impl for<'q> Executor<'q, Database = S> + Clone,
    ) -> impl Future<Output = ()>
    where
        S: QueryBuilder,
        for<'q> <S>::Arguments<'q>: IntoArguments<'q, S>,
    {
        async move {
            let mut c = CreateTableSt::init(header::create, self.table_name());
            self.on_migrate(&mut c);
            c.execute(exec).await.unwrap();
        }
    }
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
