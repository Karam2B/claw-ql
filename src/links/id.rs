#![allow(unused)]
use crate::{
    QueryBuilder, operations::insert_one_op::InsertOneFragment, prelude::stmt::InsertOneSt,
};

use super::{
    LinkData,
    relation::Relation,
    relation_many_to_many::ManyToMany,
    relation_optional_to_many::{OptionalToMany, OptionalToManyInverse},
};

pub struct SetId<I, C> {
    pub id: I,
    pub to: C,
}

pub trait Mutli<T1, T2> {}
pub trait Single<T1, T2> {}
impl<T1, T2> Single<T1, T2> for OptionalToMany<T1, T2> {}
impl<T1, T2> Mutli<T1, T2> for OptionalToManyInverse<T1, T2> {}
impl<T1, T2> Mutli<T1, T2> for ManyToMany<T1, T2> {}

pub struct SetIdSpec<F, T, I> {
    from: F,
    to: T,
    id: I,
}

impl<C, To> LinkData<C> for SetId<Vec<i64>, To>
where
    Relation<C, To>: LinkData<C, Spec: Mutli<C, To>>,
{
    type Spec = SetIdSpec<C, To, Vec<i64>>;

    fn spec(self, from: C) -> Self::Spec
    where
        Self: Sized,
    {
        SetIdSpec {
            from,
            to: self.to,
            id: self.id,
        }
    }
}
impl<C, To> LinkData<C> for SetId<i64, To>
where
    Relation<C, To>: LinkData<C, Spec: Single<C, To>>,
{
    type Spec = SetIdSpec<C, To, i64>;

    fn spec(self, _from: C) -> Self::Spec
    where
        Self: Sized,
    {
        todo!()
    }
}

// impl<S, To> InsertOneFragment<S> for Id<Vec<i64>, To>
// {}
impl<S, C, To> InsertOneFragment<S> for SetIdSpec<C, To, Vec<i64>>
where
    S: QueryBuilder,
    To: Send + Sync,
    C: Send + Sync,
{
    type Inner = ();

    type Output = Vec<i64>;

    fn on_insert(&self, data: &mut Self::Inner, st: &mut InsertOneSt<S>) {
        todo!()
    }

    fn returning(&self) -> Vec<String> {
        todo!()
    }

    fn from_row(&self, data: &mut Self::Inner, row: &S::Row) {
        todo!()
    }

    fn sub_op<'this, E: for<'q> sqlx::Executor<'q, Database = S> + Clone>(
        &'this self,
        data: &'this mut Self::Inner,
        exec: E,
    ) -> impl Future<Output = ()> + Send + use<'this, To, C, S, E> {
        async { todo!() }
    }

    fn take(self, data: Self::Inner) -> Self::Output {
        todo!()
    }
}
impl<S, C, To> InsertOneFragment<S> for SetIdSpec<C, To, i64>
where
    S: QueryBuilder,
    To: Send + Sync,
    C: Send + Sync,
{
    type Inner = ();

    type Output = i64;

    fn on_insert(&self, data: &mut Self::Inner, st: &mut InsertOneSt<S>) {
        todo!()
    }

    fn returning(&self) -> Vec<String> {
        todo!()
    }

    fn from_row(&self, data: &mut Self::Inner, row: &S::Row) {
        todo!()
    }

    fn sub_op<'this, E: for<'q> sqlx::Executor<'q, Database = S> + Clone>(
        &'this self,
        data: &'this mut Self::Inner,
        exec: E,
    ) -> impl Future<Output = ()> + Send + use<'this, To, C, S, E> {
        async { todo!() }
    }

    fn take(self, data: Self::Inner) -> Self::Output {
        todo!()
    }
}
