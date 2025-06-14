use super::LinkedOutput;
use super::collections::Collection;
use crate::Accept;
use crate::links::relation::Relation;
use crate::operations::CollectionOutput;
use crate::statements::delete_st::DeleteSt;
use crate::{QueryBuilder, build_tuple::BuildTuple, links::LinkData};
use sqlx::{ColumnIndex, Decode, Executor, Type};
use std::marker::PhantomData;

pub struct DeleteOne<S, H: Collection<S>, L> {
    handler: H,
    id: i64,
    links: L,
    _pd: PhantomData<S>,
}

pub fn delete_one<S, H>(id: i64, from_collection: H) -> DeleteOne<S, H, ()>
where
    H: Collection<S>,
{
    DeleteOne {
        _pd: PhantomData,
        id,
        handler: from_collection,
        links: (),
    }
}

pub trait DeleteOneFragment<S: QueryBuilder>: Sync + Send {
    type Output;
    type Inner: Default + Send + Sync;
    fn first_sup_op<'this, E: for<'q> Executor<'q, Database = S> + Clone>(
        &'this mut self,
        data: &'this mut Self::Inner,
        exec: E,
        id: i64,
    ) -> impl Future<Output = ()> + Send + use<'this, Self, S, E>;
    fn returning(&self) -> Vec<String>;
    fn from_row(&mut self, data: &mut Self::Inner, row: &S::Row);
    fn take(self, data: Self::Inner) -> Self::Output;
}

impl<S, H: Collection<S>, L> DeleteOne<S, H, L>
where
    S: QueryBuilder,
    L: BuildTuple,
{
    pub fn relation<D>(self, ty: D) -> DeleteOne<S, H, L::Bigger<
    <Relation<H, D> as LinkData<
    H>>
    ::Spec
    >>
    where
        H: Clone,
        Relation<H, D>: LinkData<H, Spec: DeleteOneFragment<S>>,
    {
        let spec = Relation {
            from: self.handler.clone(),
            to: ty,
        }
        .spec(self.handler.clone());

        DeleteOne {
            links: self.links.into_bigger(spec),
            handler: self.handler,
            _pd: PhantomData,
            id: self.id,
        }
    }
    pub fn link<D>(self, ty: D) -> DeleteOne<S, H, L::Bigger<D::Spec>>
    where
        H: Clone,
        D: LinkData<H, Spec: DeleteOneFragment<S>>,
    {
        let spec = ty.spec(self.handler.clone());
        DeleteOne {
            links: self.links.into_bigger(spec),
            handler: self.handler,
            _pd: PhantomData,
            id: self.id,
        }
    }
    pub async fn exec_op(
        mut self,
        db: impl for<'e> Executor<'e, Database = S> + Clone,
    ) -> Option<LinkedOutput<H::Data, L::Output>>
    where
        S: Accept<i64>,
        i64: for<'e> Decode<'e, S> + Type<S>,
        for<'s> &'s str: ColumnIndex<S::Row>,
        L: DeleteOneFragment<S>,
        for<'c> &'c mut S::Connection: sqlx::Executor<'c, Database = S>,
        DeleteSt<S>: crate::execute::Execute<S>,
    {
        use crate::execute::Execute;
        use sqlx::Row;

        let handler = self.handler;

        let st = DeleteSt::init_where_id_eq(handler.table_name().to_string(), self.id);

        let mut worker_data = L::Inner::default();

        self.links
            .first_sup_op(&mut worker_data, db.clone(), self.id)
            .await;

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
DeleteOneFragment<S>
for
    ($($ty,)*)
where
    S: QueryBuilder,
    $($ty: Send + DeleteOneFragment<S>,)*
{
    type Output = ($($ty::Output,)*);
    type Inner = ($($ty::Inner,)*);
    async fn first_sup_op<'this, E: for<'q> Executor<'q, Database = S> + Clone>(
        &'this mut self,
        data: &'this mut Self::Inner,
        exec: E,
        id: i64
    ) {
         $(paste::paste!(self.$part.first_sup_op(&mut data.$part, exec.clone(), id).await);)*
    }
    fn returning(&self) -> Vec<String> {
        let mut rt = Vec::new();

        $(rt.extend(paste::paste!(self.$part.returning()));)*

        rt
    }
    fn from_row(&mut self, data: &mut Self::Inner, row: &S::Row) {
        $(paste::paste!(self.$part.from_row(&mut data.$part, row));)*
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
