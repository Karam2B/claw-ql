use paste::paste;
use serde::Serialize;
use sqlx::{ColumnIndex, Database, Decode, Executor, Pool, Type};
use std::{marker::PhantomData, pin::Pin};

use crate::{
    QueryBuilder, build_tuple::BuildTuple, execute::Execute, links::LinkData,
    operations::CollectionOutput, prelude::stmt, statements::insert_one_st::InsertOneSt,
};

use super::{
    LinkedOutput,
    collections::{Collection, HasHandler},
};

pub trait InsertOneFragment<S: Database>: Sync + Send {
    type Inner: Default + Send + Sync;
    type Output;
    fn on_insert(&mut self, data: &mut Self::Inner, st: &mut InsertOneSt<S>);
    fn returning(&mut self) -> Vec<String>;
    fn from_row(&mut self, data: &mut Self::Inner, row: &S::Row);
    fn second_sub_op<'this, E: for<'q> Executor<'q, Database = S> + Clone>(
        &'this mut self,
        data: &'this mut Self::Inner,
        exec: E,
    ) -> impl Future<Output = ()> + Send + use<'this, Self, S, E>;

    fn first_sub_op<'this, E: for<'q> Executor<'q, Database = S> + Clone>(
        &'this mut self,
        data: &'this mut Self::Inner,
        exec: E,
    ) -> impl Future<Output = ()> + Send + use<'this, Self, S, E>;
    fn take(self, data: Self::Inner) -> Self::Output;
}

pub trait InsertOneJsonFragment<S: QueryBuilder>: Send + Sync {
    fn on_insert(&mut self, st: &mut InsertOneSt<S>);
    fn from_row(&mut self, row: &S::Row);
    fn first_sub_op<'this>(
        &'this mut self,
        pool: Pool<S>,
    ) -> Pin<Box<dyn Future<Output = ()> + Send + 'this>>;
    fn second_sub_op<'this>(
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
    fn first_sub_op<'this>(
        &'this mut self,
        pool: Pool<S>,
    ) -> Pin<Box<dyn Future<Output = ()> + Send + 'this>> {
        Box::pin(async move { self.0.first_sub_op(&mut self.1, &pool).await })
    }

    #[inline]
    fn second_sub_op<'this>(
        &'this mut self,
        pool: Pool<S>,
    ) -> Pin<Box<dyn Future<Output = ()> + Send + 'this>> {
        Box::pin(async move { self.0.second_sub_op(&mut self.1, &pool).await })
    }

    #[inline]
    fn take(self: Box<Self>) -> serde_json::Value {
        let taken = self.0.take(self.1);
        serde_json::to_value(taken).unwrap()
    }
}

pub struct InsertOne<S, C: Collection<S>, L> {
    data: C::Data,
    handler: C,
    links: L,
    _pd: PhantomData<(S,)>,
}

pub fn insert_one<S, C: HasHandler>(collection: C) -> InsertOne<S, C::Handler, ()>
where
    C: HasHandler<Handler: Collection<S, Data = C>>,
{
    InsertOne {
        _pd: PhantomData,
        data: collection,
        handler: Default::default(),
        links: (),
    }
}

impl<S, H: Collection<S>, L> InsertOne<S, H, L>
where
    S: QueryBuilder,
    L: BuildTuple,
{
    pub fn link<D>(self, ty: D) -> InsertOne<S, H, L::Bigger<D::Spec>>
    where
        H: Clone,
        D: LinkData<H, Spec: InsertOneFragment<S> + Send>,
    {
        let spec = ty.spec(self.handler.clone());
        InsertOne {
            links: self.links.into_bigger(spec),
            data: self.data,
            handler: self.handler,
            _pd: PhantomData,
        }
    }

    pub async fn exec_op(
        mut self,
        db: impl for<'e> Executor<'e, Database = S> + Clone,
    ) -> LinkedOutput<H::Data, L::Output>
    where
        L: InsertOneFragment<S> + Send,
        i64: Type<S> + for<'e> Decode<'e, S>,
        for<'s> &'s str: ColumnIndex<S::Row>,
    {
        let handler = self.handler;

        let mut st = stmt::InsertOneSt::init(handler.table_name().to_string());

        handler.on_insert(self.data, &mut st);

        let mut worker_data = L::Inner::default();

        self.links.first_sub_op(&mut worker_data, db.clone()).await;

        self.links.on_insert(&mut worker_data, &mut st);

        let mut s: Vec<String> = handler.members();

        s.extend(self.links.returning());

        s.push(String::from("id"));

        let s = st
            .returning(s)
            .fetch_one(db.clone(), |r| {
                use sqlx::Row;
                let id: i64 = r.get("id");
                let attr = handler.from_row_noscope(&r);
                self.links.from_row(&mut worker_data, &r);
                return Ok(CollectionOutput { id, attr });
            })
            .await
            .unwrap();

        self.links.second_sub_op(&mut worker_data, db).await;

        let links = self.links.take(worker_data);

        return LinkedOutput {
            id: s.id,
            attr: s.attr,
            links,
        };
    }
}

macro_rules! implt {
    ($([$ty:ident, $part:literal]),*) => {
#[allow(unused)]
impl
    <S, $($ty,)* >
InsertOneFragment<S>
for
    ($($ty,)*)
where
    S: QueryBuilder,
    $($ty: Send + InsertOneFragment<S>,)*
{
    type Output = ($($ty::Output,)*);
    type Inner = ($($ty::Inner,)*);
    fn on_insert(&mut self, data: &mut Self::Inner, st: &mut InsertOneSt<S>) {
        $(paste!(self.$part.on_insert(&mut data.$part, st));)*
    }
    fn returning(&mut self) -> Vec<String> {
        let mut rt = Vec::new();

        $(rt.extend(paste!(self.$part.returning()));)*

        rt
    }
    fn from_row(&mut self, data: &mut Self::Inner, row: &S::Row) {
        $(paste!(self.$part.from_row(&mut data.$part, row));)*
    }
    async fn first_sub_op<'this, E: for<'q> Executor<'q, Database = S> + Clone>(
        &'this mut self,
        data: &'this mut Self::Inner,
        exec: E,
    ) {
        $(paste!(self.$part.first_sub_op(&mut data.$part, exec.clone()).await);)*
    }
    async fn second_sub_op<'this, E: for<'q> Executor<'q, Database = S> + Clone>(
        &'this mut self,
        data: &'this mut Self::Inner,
        exec: E,
    ) {
        $(paste!(self.$part.second_sub_op(&mut data.$part, exec.clone()).await);)*
    }
    fn take(self, data: Self::Inner) -> Self::Output {
        ($(paste!(self.$part.take(data.$part)),)*)
    }
}
    }
    }

#[rustfmt::skip]
const _: () = {
    implt!();
    implt!([R0, 0]);
    implt!([R0, 0], [R1, 1]);
    implt!([R0, 0], [R1, 1], [R2, 2]);
    implt!([R0, 0], [R1, 1], [R2, 2], [R3, 3]);
    implt!([R0, 0], [R1, 1], [R2, 2], [R3, 3], [R4, 4]);
    implt!([R0, 0], [R1, 1], [R2, 2], [R3, 3], [R4, 4], [R5, 5]);
    implt!([R0, 0], [R1, 1], [R2, 2], [R3, 3], [R4, 4], [R5, 5], [R6, 6]);
    implt!([R0, 0], [R1, 1], [R2, 2], [R3, 3], [R4, 4], [R5, 5], [R6, 6], [R7, 7]);
    implt!([R0, 0], [R1, 1], [R2, 2], [R3, 3], [R4, 4], [R5, 5], [R6, 6], [R7, 7], [R8, 8]);
    implt!([R0, 0], [R1, 1], [R2, 2], [R3, 3], [R4, 4], [R5, 5], [R6, 6], [R7, 7], [R8, 8], [R9, 9]);
    implt!([R0, 0], [R1, 1], [R2, 2], [R3, 3], [R4, 4], [R5, 5], [R6, 6], [R7, 7], [R8, 8], [R9, 9], [R10, 10]);
    implt!([R0, 0], [R1, 1], [R2, 2], [R3, 3], [R4, 4], [R5, 5], [R6, 6], [R7, 7], [R8, 8], [R9, 9], [R10, 10], [R11, 11]);
};

// todo impl insert_one on json_client
