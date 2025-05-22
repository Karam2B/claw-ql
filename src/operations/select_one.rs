use std::marker::PhantomData;

use crate::execute::Execute;
use serde::Serialize;
use sqlx::{ColumnIndex, Decode, Encode, Pool, Row, Type};

use crate::{
    QueryBuilder,
    build_tuple::BuildTuple,
    operations::collections::{Collection, Filters},
    prelude::normal::stmt,
    statements::select_st::SelectSt,
};

pub fn get_one<S, Base>(_: PhantomData<Base>) -> SelectOne<S, Base, (), ()> {
    SelectOne {
        _pd: PhantomData,
        links: (),
        filters: (),
    }
}

pub struct SelectOne<S, C, L, F> {
    links: L,
    filters: F,
    _pd: PhantomData<(S, C)>,
}

#[derive(Clone, Debug, PartialEq, Serialize)]
pub struct SelectOneOutput<C, D> {
    pub id: i64,
    pub attr: C,
    pub links: D,
}

impl<S, Base, L, Q> SelectOne<S, Base, L, Q>
where
    L: BuildTuple,
    Q: BuildTuple,
{
    pub fn filter<N>(self, ty: N) -> SelectOne<S, N, L, Q::Bigger<N>>
    where
        N: Filters<S, Base>,
    {
        SelectOne {
            links: self.links,
            filters: self.filters.into_bigger(ty),
            _pd: PhantomData,
        }
    }
}

pub trait GetOneWorker<S: QueryBuilder>: Sync + Send {
    type Inner: Default + Send + Sync;
    type Output;
    fn on_select(&self, data: &mut Self::Inner, st: &mut SelectSt<S>);
    fn from_row(&self, data: &mut Self::Inner, row: &S::Row);
    fn sub_op<'this>(
        &'this self,
        data: &'this mut Self::Inner,
        pool: Pool<S>,
    ) -> impl Future<Output = ()> + Send + 'this;
    fn take(self, data: Self::Inner) -> Self::Output;
}

#[rustfmt::skip]
mod get_one_worker_tuple_impls {
    use sqlx::Pool;

    use super::GetOneWorker;
    use crate::{QueryBuilder, statements::select_st::SelectSt};
    use paste::paste;

    macro_rules! implt {
    ($([$ty:ident, $part:literal],)*) => {
        #[allow(unused)]
        impl
            <S, $($ty,)* >
        GetOneWorker<S>
        for
            ($($ty,)*)
        where
            S: QueryBuilder,
            $($ty: Sync + Send + GetOneWorker<S>,)*
        {
            type Output = ($($ty::Output,)*);
            type Inner = ($($ty::Inner,)*);
            fn on_select(&self, data: &mut Self::Inner, st: &mut SelectSt<S>) {
                $(paste!(self.$part.on_select(&mut data.$part, st));)*
            }
            fn from_row(&self, data: &mut Self::Inner, row: &S::Row) {
                $(paste!(self.$part.from_row(&mut data.$part, row));)*
            }
            fn sub_op<'a>(
                &'a self,
                data: &'a mut Self::Inner,
                pool: Pool<S>,
            ) -> impl std::future::Future<Output = ()> + Send {
                async move { 
                    $(
                        paste!(self.$part.sub_op(
                            &mut data.$part, pool.clone()
                        ).await);
                    )*
                }
            }
            fn take(self, data: Self::Inner) -> Self::Output {
                ($(paste!(self.$part.take(data.$part)),)*)
            }
        }
    }
    }

    implt!();
    implt!([R0, 0],);
    implt!([R0, 0], [R1, 1],);
}

impl<S, C, L, F> SelectOne<S, C, L, F>
where
    S: QueryBuilder,
    SelectSt<S>: Execute<S>,
    C: Collection<S>,
    L: GetOneWorker<S> + Send + Sync,
    F: Filters<S, C>,
    // sqlx gasim
    for<'c> &'c mut S::Connection: sqlx::Executor<'c, Database = S>,
    for<'e> i64: Encode<'e, S> + Type<S> + Decode<'e, S>,
    for<'e> &'e str: ColumnIndex<S::Row>,
{
    pub async fn exec_op(self, db: Pool<S>) -> Option<SelectOneOutput<C, L::Output>> {
        let mut st = stmt::SelectSt::init(C::table_name().to_string());

        // st.select_aliased(C::table_name().to_string(), "id".to_string(), "local_id");

        C::on_select(&mut st);
        self.filters.on_select(&mut st);

        let mut worker_data = L::Inner::default();

        self.links.on_select(&mut worker_data, &mut st);

        let res = st
            .fetch_optional(&db, |r| {
                let id: i64 = r.get("local_id");
                let attr = C::from_row_noscope(&r);
                self.links.from_row(&mut worker_data, &r);
                Ok(SelectOneOutput {
                    id,
                    attr,
                    links: (),
                })
            })
            .await
            .unwrap()?;

        self.links.sub_op(&mut worker_data, db).await;
        let data = self.links.take(worker_data);

        return Some(SelectOneOutput {
            id: res.id,
            attr: res.attr,
            links: data,
        });
    }
}
