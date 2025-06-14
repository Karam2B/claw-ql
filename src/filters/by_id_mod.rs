use crate::{
    collections::{Collection, Filter}, expressions::ColEq, prelude::{col, stmt::SelectSt}, statements::{delete_st::DeleteSt, update_st::{ UpdateSt}}, BindItem, QueryBuilder
};

#[allow(non_camel_case_types)]
pub struct by_id(pub i64);

impl<S, C> Filter<S, C> for by_id
where
    ColEq<i64>: BindItem<S>,
    C: Collection<S>,
    S: QueryBuilder,
{
    fn on_delete(self, handler: &C, st: &mut DeleteSt<S>)
    where
        S: QueryBuilder,
    {
        st.where_(col("id").table(handler.table_name()).eq(self.0));
    }
    fn on_update(self, handler: &C, st: &mut UpdateSt<S>)
    where
        S: QueryBuilder,
    {
        st.where_(col("id").table(handler.table_name()).eq(self.0));
    }
    fn on_select(self, handler: &C, st: &mut SelectSt<S>) {
        st.where_(col("id").table(handler.table_name()).eq(self.0));
    }
}
