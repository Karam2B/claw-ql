use std::{marker::PhantomData, pin::Pin};

use serde::Serialize;
use sqlx::{Executor, Pool};

use crate::{
    QueryBuilder,
    build_tuple::BuildTuple,
    links::{LinkData, relation::Relation},
    statements::insert_one_st::InsertOneSt,
};

pub trait InsertOneFragment<S: QueryBuilder>: Sync + Send {
    type Inner: Default + Send + Sync;
    type Output;
    fn on_insert(&self, data: &mut Self::Inner, st: &mut InsertOneSt<S>);
    fn from_row(&self, data: &mut Self::Inner, row: &S::Row);
    fn sub_op<'this, E: for<'q> Executor<'q, Database = S> + Clone>(
        &'this self,
        data: &'this mut Self::Inner,
        exec: E,
    ) -> impl Future<Output = ()> + Send + use<'this, Self, S, E>;
    fn take(self, data: Self::Inner) -> Self::Output;
}

pub trait InsertOneJsonFragment<S: QueryBuilder>: Send + Sync {
    fn on_insert(&mut self, st: &mut InsertOneSt<S>);
    fn from_row(&mut self, row: &S::Row);
    fn sub_op<'this>(
        &'this mut self,
        pool: Pool<S>,
    ) -> Pin<Box<dyn Future<Output = ()> + Send + 'this>>;
    fn take(self: Box<Self>) -> serde_json::Value;
}

impl<S: QueryBuilder, T> InsertOneJsonFragment<S> for (T, T::Inner)
where
    T::Output: Serialize,
    T: InsertOneFragment<S>,
    for<'c> &'c mut S::Connection: Executor<'c, Database = S>,
{
    #[inline]
    fn on_insert(&mut self, st: &mut InsertOneSt<S>) {
        self.0.on_insert(&mut self.1, st)
    }

    #[inline]
    fn from_row(&mut self, row: &<S>::Row) {
        self.0.from_row(&mut self.1, row)
    }

    #[inline]
    fn sub_op<'this>(
        &'this mut self,
        pool: Pool<S>,
    ) -> Pin<Box<dyn Future<Output = ()> + Send + 'this>> {
        Box::pin(async move { self.0.sub_op(&mut self.1, &pool).await })
    }

    #[inline]
    fn take(self: Box<Self>) -> serde_json::Value {
        let taken = self.0.take(self.1);
        serde_json::to_value(taken).unwrap()
    }
}

pub fn insert_one<S, Base>(collection: Base) -> InsertOne<S, Base, ()> {
    InsertOne {
        _pd: PhantomData,
        collection,
        links: (),
    }
}

pub struct InsertOne<S, C, L> {
    collection: C,
    links: L,
    _pd: PhantomData<(S,)>,
}

impl<S, Base, L> InsertOne<S, Base, L>
where
    Base: Clone,
    S: QueryBuilder,
    L: BuildTuple,
{
    pub fn relation<To>(
        self,
        to: To,
    ) -> InsertOne<S, Base, L::Bigger<<Relation<Base, To> as LinkData<Base>>::Spec>>
    where
        Relation<Base, To>: LinkData<Base, Spec: InsertOneFragment<S> + Send>,
    {
        let from = self.collection.clone();
        self.link(Relation { from, to })
    }
    pub fn link<D>(self, ty: D) -> InsertOne<S, Base, L::Bigger<D::Spec>>
    where
        D: LinkData<Base, Spec: InsertOneFragment<S> + Send>,
    {
        let spec = ty.spec(self.collection.clone());
        InsertOne {
            links: self.links.into_bigger(spec),
            collection: self.collection,
            _pd: PhantomData,
        }
    }
}

// todo: impl InsertOneFragment for tupels

// todo: exec_op

// todo impl insert_one on json_client
