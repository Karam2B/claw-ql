use std::marker::PhantomData;

use serde::{Deserialize, Serialize};

use crate::{
    BindItem, QueryBuilder,
    collections::{Collection, Filter},
    expressions::ColEq,
    json_client::JsonCollection,
    prelude::{col, stmt::SelectSt},
    statements::{delete_st::DeleteSt, update_st::UpdateSt},
};

#[allow(non_camel_case_types)]
#[derive(Clone, Serialize, Deserialize)]
#[serde(from = "by_id_map")]
#[serde(into = "by_id_map")]
pub struct by_id(#[serde(rename = "id")] pub i64);

#[allow(non_camel_case_types)]
#[derive(Serialize, Deserialize)]
struct by_id_map {
    id: i64,
}

impl From<by_id> for by_id_map {
    fn from(value: by_id) -> Self {
        by_id_map { id: value.0 }
    }
}
impl From<by_id_map> for by_id {
    fn from(value: by_id_map) -> Self {
        by_id(value.id)
    }
}

impl<S> Filter<S, dyn JsonCollection<S>> for by_id
where
    ColEq<i64>: BindItem<S>,
    S: QueryBuilder,
{
    fn on_delete(self, handler: &dyn JsonCollection<S>, st: &mut DeleteSt<S>)
    where
        S: QueryBuilder,
    {
        st.where_(col("id").table(handler.table_name_js()).eq(self.0));
    }
    fn on_update(self, handler: &dyn JsonCollection<S>, st: &mut UpdateSt<S>)
    where
        S: QueryBuilder,
    {
        st.where_(col("id").table(handler.table_name_js()).eq(self.0));
    }
    fn on_select(self, handler: &dyn JsonCollection<S>, st: &mut SelectSt<S>) {
        st.where_(col("id").table(handler.table_name_js()).eq(self.0));
    }
}

impl<S, C> Filter<S, C> for by_id
where
    ColEq<i64>: BindItem<S>,
    C: Collection<S> + ?Sized,
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

mod attempt_to_liq {
    // pub struct JsonFilter<T, C>(T, PhantomData<(T, C)>);
    // pub trait DynamicFilterSelector<S, C> {
    //     fn what_is_this(jc: &dyn JsonCollection<S>) -> &C;
    //     fn errors() -> String;
    // }
    // impl<S, T, C> Filter<S, dyn JsonCollection<S>> for JsonFilter<T, C>
    // where
    //     C: Send + Sync,
    //     C: DynamicFilterSelector<S, C>,
    //     T: Filter<S, C>,
    // {
    //     fn on_delete(self, handler: &dyn JsonCollection<S>, st: &mut DeleteSt<S>)
    //     where
    //         S: QueryBuilder,
    //     {
    //         self.0.on_delete(C::what_is_this(handler), st);
    //     }
    //     fn on_update(self, handler: &dyn JsonCollection<S>, st: &mut UpdateSt<S>)
    //     where
    //         S: QueryBuilder,
    //     {
    //         todo!()
    //     }
    //     fn on_select(self, handler: &dyn JsonCollection<S>, st: &mut SelectSt<S>)
    //     where
    //         S: QueryBuilder,
    //     {
    //         todo!()
    //     }
    // }
}
