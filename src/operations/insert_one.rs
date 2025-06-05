use sqlx::Pool;

use crate::{QueryBuilder, prelude::stmt::SelectSt};

pub trait InsertOneFragment<S: QueryBuilder>: Sync + Send {
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

impl<S: QueryBuilder, T: Sync + Send> InsertOneFragment<S> for T {
    type Inner = ();

    type Output = ();

    fn on_select(&self, _data: &mut Self::Inner, _st: &mut SelectSt<S>) {
        todo!()
    }

    fn from_row(&self, _data: &mut Self::Inner, _row: &<S>::Row) {
        todo!()
    }

    fn sub_op<'this>(
        &'this self,
        _data: &'this mut Self::Inner,
        _pool: Pool<S>,
    ) -> impl Future<Output = ()> + Send + use<'this, T, S> {
        async { todo!() }
    }

    fn take(self, _data: Self::Inner) -> Self::Output {
        todo!()
    }
}
