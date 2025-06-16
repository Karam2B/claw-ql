use crate::{
    QueryBuilder, build_tuple::BuildTuple, execute::Execute, filters::by_id_mod::by_id,
    links::LinkData, operations::CollectionOutput, statements::update_st::UpdateSt,
};
use sqlx::{ColumnIndex, Decode, Executor, prelude::Type};
use std::marker::PhantomData;

use super::{
    LinkedOutput,
    collections::{Collection, Filter, HasHandler},
};

pub trait UpdateOneFragment<S: QueryBuilder>: Sync + Send {
    type Inner: Default + Send + Sync;
    type Output;
    fn on_update(&mut self, data: &mut Self::Inner, st: &mut UpdateSt<S>);
    fn returning(&mut self) -> Vec<String>;
    fn from_row(&mut self, data: &mut Self::Inner, row: &S::Row);
    // // TODO: how to handle the case where update is not performed due
    // // to 'RowNotFound' i.e. where clause did not target any entry
    // // by that point first_sub_op already has executed, how it should
    // // be reverted or what checks should exist to prevent it from executing
    // fn first_sub_op<'this, E: for<'q> Executor<'q, Database = S> + Clone>(
    //     &'this mut self,
    //     data: &'this mut Self::Inner,
    //     exec: E,
    // ) -> impl Future<Output = ()> + Send + use<'this, Self, S, E>;
    fn second_sub_op<'this, E: for<'q> Executor<'q, Database = S> + Clone>(
        &'this mut self,
        data: &'this mut Self::Inner,
        exec: E,
    ) -> impl Future<Output = ()> + Send + use<'this, Self, S, E>;
    fn take(self, data: Self::Inner) -> Self::Output;
}

pub struct UpdateOne<S, C: Collection<S>, L, F> {
    data: C::Partial,
    handler: C,
    links: L,
    filters: F,
    _pd: PhantomData<(S,)>,
}

pub fn update_one_no_id<S, C: HasHandler>(partial_collection: C) -> UpdateOne<S, C::Handler, (), ()>
where
    C: HasHandler<Handler: Collection<S, Partial = C>>,
{
    UpdateOne {
        _pd: PhantomData,
        data: partial_collection,
        handler: Default::default(),
        links: (),
        filters: (),
    }
}
pub fn update_one<S, C: HasHandler>(
    id: i64,
    partial_collection: C,
) -> UpdateOne<S, C::Handler, (), (by_id,)>
where
    C: HasHandler<Handler: Collection<S, Partial = C>>,
{
    UpdateOne {
        _pd: PhantomData,
        data: partial_collection,
        handler: Default::default(),
        links: (),
        filters: (by_id(id),),
    }
}

impl<S, H: Collection<S>, L, F> UpdateOne<S, H, L, F>
where
    S: QueryBuilder,
    L: BuildTuple,
    F: BuildTuple,
{
    pub fn filter<Filter>(self, filter: Filter) -> UpdateOne<S, H, L, F::Bigger<Filter>>
    where
        H: Clone,
        Filter: crate::collections::Filter<S, H>,
    {
        UpdateOne {
            links: self.links,
            data: self.data,
            handler: self.handler,
            filters: self.filters.into_bigger(filter),
            _pd: PhantomData,
        }
    }
    pub fn link<D>(self, ty: D) -> UpdateOne<S, H, L::Bigger<D::Spec>, F>
    where
        H: Clone,
        D: LinkData<H, Spec: UpdateOneFragment<S> + Send>,
    {
        let spec = ty.spec(self.handler.clone());
        UpdateOne {
            links: self.links.into_bigger(spec),
            data: self.data,
            handler: self.handler,
            filters: self.filters,
            _pd: PhantomData,
        }
    }
    pub async fn exec_op(
        mut self,
        db: impl for<'e> Executor<'e, Database = S> + Clone,
    ) -> Option<LinkedOutput<H::Data, L::Output>>
    where
        UpdateSt<S>: Execute<S>,
        for<'c> &'c mut S::Connection: sqlx::Executor<'c, Database = S>,
        L: UpdateOneFragment<S> + Send,
        F: Filter<S, H>,
        i64: Type<S> + for<'e> Decode<'e, S>,
        for<'s> &'s str: ColumnIndex<S::Row>,
    {
        use sqlx::Row;

        let handler = self.handler;

        let mut st = UpdateSt::init(handler.table_name().to_string());

        handler.on_update(self.data, &mut st);

        self.filters.on_update(&handler, &mut st);

        let mut worker_data = L::Inner::default();

        self.links.on_update(&mut worker_data, &mut st);

        let mut s: Vec<String> = handler.members();

        s.extend(self.links.returning());

        s.push(String::from("id"));

        let s = st
            .returning(s)
            .fetch_optional(db.clone(), |r| {
                let id: i64 = r.get("id");
                let attr = handler.from_row_noscope(&r);
                self.links.from_row(&mut worker_data, &r);
                Ok(CollectionOutput { id, attr })
            })
            .await
            .unwrap()?;

        self.links.second_sub_op(&mut worker_data, db).await;

        let links = self.links.take(worker_data);

        Some(LinkedOutput {
            id: s.id,
            attr: s.attr,
            links,
        })
    }
}

macro_rules! implt {
    ($([$ty:ident, $part:literal]),*) => {
#[allow(unused)]
impl
    <S, $($ty,)* >
UpdateOneFragment<S>
for
    ($($ty,)*)
where
    S: QueryBuilder,
    $($ty: Send + UpdateOneFragment<S>,)*
{
    type Output = ($($ty::Output,)*);
    type Inner = ($($ty::Inner,)*);
    fn on_update(&mut self, data: &mut Self::Inner, st: &mut UpdateSt<S>) {
        $(paste::paste!(self.$part.on_update(&mut data.$part, st));)*
    }
    fn returning(&mut self) -> Vec<String> {
        let mut rt = Vec::new();

        $(rt.extend(paste::paste!(self.$part.returning()));)*

        rt
    }
    fn from_row(&mut self, data: &mut Self::Inner, row: &S::Row) {
        $(paste::paste!(self.$part.from_row(&mut data.$part, row));)*
    }
    // async fn first_sub_op<'this, E: for<'q> Executor<'q, Database = S> + Clone>(
    //     &'this mut self,
    //     data: &'this mut Self::Inner,
    //     exec: E,
    // ) {
    //     $(paste::paste!(self.$part.first_sub_op(&mut data.$part, exec.clone()).await);)*
    // }
    async fn second_sub_op<'this, E: for<'q> Executor<'q, Database = S> + Clone>(
        &'this mut self,
        data: &'this mut Self::Inner,
        exec: E,
    ) {
        $(paste::paste!(self.$part.second_sub_op(&mut data.$part, exec.clone()).await);)*
    }
    fn take(self, data: Self::Inner) -> Self::Output {
        ($(paste::paste!(self.$part.take(data.$part)),)*)
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
