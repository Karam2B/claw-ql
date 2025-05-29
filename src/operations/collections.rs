use sqlx::Database;

use crate::{
    QueryBuilder, statements::create_table_st::CreateTableSt, statements::select_st::SelectSt,
};

pub trait Collection<Q>: Sized + Send + Sync {
    type PartailCollection;
    fn on_migrate(stmt: &mut CreateTableSt<Q>)
    where
        Q: QueryBuilder;
    // fn on_update(
    //     stmt: &mut UpdateSt<S>,
    //     this: Self::PartailCollection,
    // ) -> Result<(), String>
    // where
    //     S: Database + SupportNamedBind;
    fn on_select(stmt: &mut SelectSt<Q>)
    where
        Q: QueryBuilder;

    fn members() -> &'static [&'static str];
    fn members_scoped() -> &'static [&'static str];
    fn table_name() -> &'static str;

    fn from_row_noscope(row: &Q::Row) -> Self
    where
        Q: Database;
    fn from_row_scoped(row: &Q::Row) -> Self
    where
        Q: Database;
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
