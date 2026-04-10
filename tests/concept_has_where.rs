// this is only a concept: I don't know if it is ok to use Box as
// oppose of having three method one for update, select, and delete

use crate::{
    BindItem, QueryBuilder,
    prelude::stmt::SelectSt,
    statements::{delete_st::DeleteSt, update_st::UpdateSt},
};

use super::Filter;

pub trait BindItemBoxed<S: QueryBuilder> {
    fn bind_item(
        self: Box<Self>,
        ctx: &mut S::Context1,
    ) -> Box<dyn FnOnce(&mut S::Context2) -> String>;
}

impl<S, T> BindItemBoxed<S> for T
where
    S: QueryBuilder,
    T: BindItem<S> + 'static,
{
    fn bind_item(
        self: Box<Self>,
        ctx: &mut S::Context1,
    ) -> Box<dyn FnOnce(&mut S::Context2) -> String> {
        let mov = <T as BindItem<S>>::bind_item(*self, ctx);
        Box::new(move |c| mov(c))
    }
}

impl<S> BindItem<S> for Box<dyn BindItemBoxed<S>>
where
    S: QueryBuilder,
{
    fn bind_item(
        self,
        ctx: &mut <S as QueryBuilder>::Context1,
    ) -> impl FnOnce(&mut <S as QueryBuilder>::Context2) -> String + 'static + use<S> {
        let mov = self.bind_item(ctx);
        move |ctx2| mov(ctx2)
    }
}

pub trait HasWhereClause<S> {
    fn delegate_where(&mut self, item: Box<dyn BindItemBoxed<S>>);
}

impl<S> HasWhereClause<S> for DeleteSt<S>
where
    S: QueryBuilder,
{
    fn delegate_where(&mut self, item: Box<dyn BindItemBoxed<S>>) {
        self.where_(item);
    }
}

impl<S> HasWhereClause<S> for UpdateSt<S>
where
    S: QueryBuilder,
{
    fn delegate_where(&mut self, item: Box<dyn BindItemBoxed<S>>) {
        self.where_(item);
    }
}
impl<S> HasWhereClause<S> for SelectSt<S>
where
    S: QueryBuilder,
{
    fn delegate_where(&mut self, item: Box<dyn BindItemBoxed<S>>) {
        self.where_(item);
    }
}

pub trait FilterGeneric<Q, C>: Sync + Send + LocalizeFilter {
    fn on_where(self, handler: &C, st: &mut dyn HasWhereClause<Q>)
    where
        Q: QueryBuilder;
}

// non-generic trait to solve conflicting impl
pub trait LocalizeFilter {}

impl<S, C, T> Filter<S, C> for T
where
    T: FilterGeneric<S, C> + LocalizeFilter,
{
    fn on_delete(self, handler: &C, st: &mut DeleteSt<S>)
    where
        S: QueryBuilder,
    {
        self.on_where(handler, st);
    }
    fn on_select(self, handler: &C, st: &mut SelectSt<S>)
    where
        S: QueryBuilder,
    {
        self.on_where(handler, st);
    }
    fn on_update(self, handler: &C, st: &mut UpdateSt<S>)
    where
        S: QueryBuilder,
    {
        self.on_where(handler, st);
    }
}
