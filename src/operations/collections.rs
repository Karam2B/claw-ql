use sqlx::Database;

use crate::{QueryBuilder, statements::select_st::SelectSt};

pub trait Collection<S>: Sized + Send + Sync {
    type PartailCollection;
    // fn on_migrate(stmt: &mut CreatTableSt<S>)
    // where
    //     S: Database + SupportNamedBind;
    // fn on_update(
    //     stmt: &mut UpdateSt<S>,
    //     this: Self::PartailCollection,
    // ) -> Result<(), String>
    // where
    //     S: Database + SupportNamedBind;
    fn on_select(stmt: &mut SelectSt<S>)
    where
        S: QueryBuilder;

    fn members() -> &'static [&'static str];
    fn members_scoped() -> &'static [&'static str];
    fn table_name() -> &'static str;

    fn from_row_noscope(row: &S::Row) -> Self
    where
        S: Database;
    fn from_row_scoped(row: &S::Row) -> Self
    where
        S: Database;
}

pub trait Filters<S, C>: Sync + Send {
    fn on_select(self, st: &mut SelectSt<S>)
    where
        S: QueryBuilder;
}
