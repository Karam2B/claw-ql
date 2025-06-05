use std::marker::PhantomData;

use crate::{
    execute::Execute,
    links::{LinkData, relation::Relation},
    prelude::col,
};
use serde::Serialize;
use sqlx::{ColumnIndex, Decode, Encode,  Pool, Row, Type};

use crate::{
    QueryBuilder,
    build_tuple::BuildTuple,
    operations::collections::{Collection, Filters},
    prelude::stmt,
    statements::select_st::SelectSt,
};

pub trait SelectOneFragment<S: QueryBuilder>: Sync + Send {
    type Inner: Default + Send + Sync;
    type Output;
    fn on_select(&self, data: &mut Self::Inner, st: &mut SelectSt<S>);
    fn from_row(&self, data: &mut Self::Inner, row: &S::Row);
    fn sub_op<'this>(
        &'this self,
        data: &'this mut Self::Inner,
        pool: Pool<S>,
    ) -> impl Future<Output = ()> + Send + use<'this, Self, S>;
    fn take(self, data: Self::Inner) -> Self::Output;
}

pub fn select_one<S, Base>(collection: Base) -> SelectOne<S, Base, (), ()> {
    SelectOne {
        _pd: PhantomData,
        collection,
        links: (),
        filters: (),
    }
}

pub struct SelectOne<S, C, L, F> {
    collection: C,
    links: L,
    filters: F,
    _pd: PhantomData<(S,)>,
}

#[derive(Clone, Debug, PartialEq, Serialize)]
pub struct SelectOneOutput<C, D> {
    pub id: i64,
    pub attr: C,
    pub links: D,
}

impl<S, Base, L, F> SelectOne<S, Base, L, F>
where
    Base: Clone,
    S: QueryBuilder,
    L: BuildTuple,
    F: BuildTuple,
{
    pub fn relation<To>(
        self,
        to: To,
    ) -> SelectOne<S, Base, L::Bigger<<Relation<Base, To> as LinkData<Base>>::Spec>, F>
    where
        Relation<Base, To>: LinkData<Base, Spec: SelectOneFragment<S> + Send>,
    {
        // let spec = <Relation<Base, To> as LinkData<Base>>::init_spec(self.collection.clone());
        SelectOne {
            links: self.links.into_bigger(
                Relation {
                    from: self.collection.clone(),
                    to,
                }
                .spec(self.collection.clone()),
            ),
            filters: self.filters,
            collection: self.collection,
            _pd: PhantomData,
        }
    }
    pub fn link<D>(self, ty: D) -> SelectOne<S, Base, L::Bigger<D::Spec>, F>
    where
        D: LinkData<Base, Spec: SelectOneFragment<S> + Send>,
    {
        let spec = ty.spec(self.collection.clone());
        SelectOne {
            links: self.links.into_bigger(spec),
            filters: self.filters,
            collection: self.collection,
            _pd: PhantomData,
        }
    }
    pub fn filter<N>(self, ty: N) -> SelectOne<S, Base, L, F::Bigger<N>>
    where
        N: Filters<S, Base>,
    {
        SelectOne {
            links: self.links,
            filters: self.filters.into_bigger(ty),
            collection: self.collection,
            _pd: PhantomData,
        }
    }
}

mod get_one_worker_tuple_impls {
    use sqlx::Pool;

    use super::SelectOneFragment;
    use crate::{QueryBuilder, statements::select_st::SelectSt};
    use paste::paste;

    macro_rules! implt {
    ($([$ty:ident, $part:literal]),*) => {
        #[allow(unused)]
        impl
            <S, $($ty,)* >
        SelectOneFragment<S>
        for
            ($($ty,)*)
        where
            S: QueryBuilder,
            $($ty: Sync + Send + SelectOneFragment<S>,)*
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
    implt!([R0, 0]);
    #[allow(unused)]
    impl<S, R0, R1> SelectOneFragment<S> for (R0, R1)
    where
        S: QueryBuilder,
        R0: Sync + Send + SelectOneFragment<S>,
        R1: Sync + Send + SelectOneFragment<S>,
    {
        type Output = (R0::Output, R1::Output);
        type Inner = (R0::Inner, R1::Inner);
        fn on_select(&self, data: &mut Self::Inner, st: &mut SelectSt<S>) {
            paste!(self.0.on_select(&mut data.0, st));
            paste!(self.1.on_select(&mut data.1, st));
        }
        fn from_row(&self, data: &mut Self::Inner, row: &S::Row) {
            paste!(self.0.from_row(&mut data.0, row));
            paste!(self.1.from_row(&mut data.1, row));
        }
        fn sub_op<'a>(
            &'a self,
            data: &'a mut Self::Inner,
            pool: Pool<S>,
        ) -> impl std::future::Future<Output = ()> + Send {
            async move {
                paste!(self.0.sub_op(&mut data.0, pool.clone()).await);
                paste!(self.1.sub_op(&mut data.1, pool.clone()).await);
            }
        }
        fn take(self, data: Self::Inner) -> Self::Output {
            (paste!(self.0.take(data.0)), paste!(self.1.take(data.1)))
        }
    }
}

impl<S, C, L, F> SelectOne<S, C, L, F>
where
    S: QueryBuilder,
    SelectSt<S>: Execute<S>,
    C: Collection<S>,
    L: SelectOneFragment<S> + Send + Sync,
    F: Filters<S, C>,
    // sqlx gasim
    for<'c> &'c mut S::Connection: sqlx::Executor<'c, Database = S>,
    for<'e> i64: Encode<'e, S> + Type<S> + Decode<'e, S>,
    for<'e> &'e str: ColumnIndex<S::Row>,
{
    pub async fn exec_op(self, db: Pool<S>) -> Option<SelectOneOutput<C::Yeild, L::Output>> {
        let mut st = stmt::SelectSt::init(self.collection.table_name().to_string());

        #[rustfmt::skip]
        st.select(
            col("id").
            table(self.collection.table_name()).
            alias("local_id")
        );

        self.collection.on_select(&mut st);
        self.filters.on_select(&mut st);

        let mut worker_data = L::Inner::default();

        self.links.on_select(&mut worker_data, &mut st);

        let res = st
            .fetch_optional(&db, |r| {
                let id: i64 = r.get("local_id");
                let attr = self.collection.from_row_scoped(&r);
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
