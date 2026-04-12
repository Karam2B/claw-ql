use serde::Serialize;
use sqlx::{Database, Pool};

pub mod fetch_one;
// pub mod delete_one;
pub mod insert_one;
pub mod update_one;
// pub mod fetch_many;
// pub mod update_one;

pub trait Operation<S>: Send {
    type Output: Send;
    fn exec_operation(self, pool: Pool<S>) -> impl Future<Output = Self::Output> + Send
    where
        S: Database,
        Self: Sized;
}

#[derive(Clone, Debug, PartialEq, Serialize)]
pub struct LinkedOutput<Id, C, L> {
    pub id: Id,
    pub attributes: C,
    pub links: L,
}

#[derive(Clone, Debug, PartialEq, Serialize)]
pub struct CollectionOutput<Id, C> {
    pub id: Id,
    pub attributes: C,
}

#[derive(Clone, Debug, PartialEq, Serialize)]
pub struct IdOutput<Id> {
    pub id: Id,
}

impl<I, C, L> From<LinkedOutput<I, C, L>> for CollectionOutput<I, C> {
    fn from(value: LinkedOutput<I, C, L>) -> Self {
        CollectionOutput {
            id: value.id,
            attributes: value.attributes,
        }
    }
}

impl<I, C, L> From<LinkedOutput<I, C, L>> for IdOutput<I> {
    fn from(value: LinkedOutput<I, C, L>) -> Self {
        IdOutput { id: value.id }
    }
}

impl<I, C> From<CollectionOutput<I, C>> for IdOutput<I> {
    fn from(value: CollectionOutput<I, C>) -> Self {
        IdOutput { id: value.id }
    }
}

pub use functional_impls::BoxedOperation;

mod functional_impls {
    #![allow(unused)]
    #![deny(unused_must_use)]
    use crate::operations::insert_one::LinkInsertOne;
    use futures::FutureExt;
    use std::{any::Any, pin::Pin};

    use crate::expressions::scoped_column;
    use crate::from_row::pre_alias;
    use crate::operations::{Operation, fetch_one::LinkFetchOne};
    use crate::query_builder::functional_expr::ManyImplPossible;
    use paste::paste;
    use sqlx::{Database, Executor, Pool};

    pub trait BoxedOperation<S: Database>: Send {
        fn exec_p(
            self: Box<Self>,
            pool: Pool<S>,
        ) -> Pin<Box<dyn Future<Output = Box<dyn Any + Send>> + Send>>
        where
            Self: 'static;
    }

    impl<T, S> BoxedOperation<S> for T
    where
        T: Send + Operation<S, Output: Send> + 'static,
        S: Database,
    {
        fn exec_p(
            self: Box<Self>,
            pool: Pool<S>,
        ) -> Pin<Box<dyn Future<Output = Box<dyn Any + Send>> + Send>> {
            let p = pool.clone();
            Box::pin(async move {
                Operation::exec_operation(*self, p)
                    .map(|f| Box::new(f) as Box<dyn Any + Send>)
                    .await
            })
        }
    }

    impl<S> Operation<S> for Box<dyn BoxedOperation<S> + Send> {
        type Output = Box<dyn Any + Send>;
        fn exec_operation(self, pool: Pool<S>) -> impl Future<Output = Self::Output> + Send
        where
            S: Database,
        {
            BoxedOperation::exec_p(self, pool)
        }
    }

    impl<S, T> Operation<S> for Vec<T>
    where
        T: Operation<S, Output: Send> + Send,
    {
        type Output = Vec<T::Output>;

        async fn exec_operation(self, pool: sqlx::Pool<S>) -> Self::Output
        where
            S: sqlx::Database,
        {
            let mut v = vec![];
            for each in self {
                v.push(each.exec_operation(pool.clone()).await);
            }
            v
        }
    }

    impl<T, S> LinkFetchOne<S> for Vec<T>
    where
        T: LinkFetchOne<S>,
        T::Joins: IntoIterator,
        T::Wheres: IntoIterator,
    {
        type Joins = Vec<<T::Joins as IntoIterator>::Item>;

        type Wheres = Vec<<T::Wheres as IntoIterator>::Item>;

        fn non_aggregating_select_items(&self) -> Vec<scoped_column<String, String>> {
            let mut items = vec![];
            for each in self {
                items.extend(each.non_aggregating_select_items());
            }
            items
        }

        fn non_duplicating_joins(&self) -> Self::Joins {
            let mut joins = vec![];
            for each in self {
                joins.extend(each.non_duplicating_joins());
            }
            joins
        }

        fn wheres(&self) -> Self::Wheres {
            let mut wheres = vec![];
            for each in self {
                wheres.extend(each.wheres());
            }
            wheres
        }

        type Inner = Vec<T::Inner>;

        type SubOp = Vec<T::SubOp>;

        fn sub_op(
            &self,
            row: crate::from_row::pre_alias<'_, <S as sqlx::Database>::Row>,
        ) -> (Self::SubOp, Self::Inner)
        where
            S: sqlx::Database,
        {
            let mut sup_op = vec![];
            let mut inner = vec![];

            for each in self {
                let each = each.sub_op(pre_alias(row.0, row.1));
                sup_op.push(each.0);
                inner.push(each.1);
            }
            (sup_op, inner)
        }

        type Output = Vec<T::Output>;

        fn take(
            self,
            extend: <Self::SubOp as crate::operations::Operation<S>>::Output,
            inner: Self::Inner,
        ) -> Self::Output {
            let mut this = self.into_iter();
            let mut extend = extend.into_iter();
            let mut inner = inner.into_iter();

            let mut out = vec![];

            loop {
                match (this.next(), extend.next(), inner.next()) {
                    (Some(this), Some(extend), Some(inner)) => {
                        out.push(this.take(extend, inner));
                    }
                    (None, None, None) => break,
                    _ => {
                        panic!("bug: unmatched lenght ")
                    }
                }
            }

            out
        }
    }

    macro_rules! implt {
    ( $([$t:ident, $part:literal])*) => {



impl<S, $($t,)* > Operation<S> for ($($t,)*)
where
    $($t: Operation<S>,)*
{
    type Output = (
        $($t::Output,)*
    );

    async fn exec_operation(self, pool: sqlx::Pool<S>) -> Self::Output
    where
        S: sqlx::Database,
    {
        ($(
            paste!(self.$part).exec_operation(pool.clone()).await,
        )*)
    }
}

impl <$($t,)* S> LinkFetchOne<S> for ($($t,)*)
where
    $($t: LinkFetchOne<S, SubOp: Operation<S>>,)*
{
    type Joins = (
        $(ManyImplPossible<$t::Joins>,)*
    );

    type Wheres = (
        $(ManyImplPossible<$t::Wheres>,)*
    );

    fn non_aggregating_select_items(&self) -> Vec<scoped_column<String, String>> {
        let mut items = vec![];
        $(
            items.extend(paste!(self.$part.non_aggregating_select_items()));
        )*
        items
    }

    fn non_duplicating_joins(&self) -> Self::Joins {
        ($(ManyImplPossible {
            start: "",
            join: ", ",
            expressions: paste!(self.$part.non_duplicating_joins()),
        },)*)
    }

    fn wheres(&self) -> Self::Wheres {
        ($(ManyImplPossible {
            start: "",
            join: ", ",
            expressions: paste!(self.$part.wheres()),
        },)*)
    }

    type Inner = (
        $($t::Inner,)*
    );

    type SubOp = (
        $($t::SubOp,)*
    );

    fn sub_op(
        &self,
        row: pre_alias<'_, <S as sqlx::Database>::Row>,
    ) -> (Self::SubOp, Self::Inner)
    where
        S: sqlx::Database,
    {
        $(paste!{
            let [<each_ $part>] = self.$part.sub_op(pre_alias(row.0, row.1));
        })*

        (
            ($(paste!{[<each_ $part>]}.0,)*),
            ($(paste!{[<each_ $part>]}.1,)*)
        )
    }

    type Output = (
        $($t::Output,)*
    );

    fn take(
        self,
        extend: <Self::SubOp as crate::operations::Operation<S>>::Output,
        inner: Self::Inner,
    ) -> Self::Output {
        (
            $(
                paste!(self.$part.take(extend.$part, inner.$part)),
            )*
        )
    }
}
// impl <$($t,)*> LinkInsertOne for ($($t,)*)
// where
//     $($t: LinkInsertOne,)*
// {
//     fn current_table_cols(&self) -> Vec<String> {
//         todo!()
//     }
//     type FromRow = ();
//     type SubOp = ();
//     fn from_row<S>(&self, _row: &S::Row) -> (Self::FromRow, Self::SubOp)
//     where
//         S: Database,
//     {
//         todo!()
//     }
//     type Output = ();
//     fn take<S>(
//         self,
//         _from_row: Self::FromRow,
//         _sub_op: <Self::SubOp as Operation<S>>::Output,
//     ) -> Self::Output
//     where
//         Self::SubOp: Operation<S>,
//         S: Database,
//     {
//         todo!()
//     }
// }
    };
}

    implt!();
    implt!([T0, 0]);
    implt!([T0, 0] [T1, 1]);
    implt!([T0, 0] [T1, 1] [T2, 2]);
    implt!([T0, 0] [T1, 1] [T2, 2] [T3, 3]);
}

pub mod execute_expression {
    use crate::database_extention::DatabaseExt;
    use crate::use_executor;
    use crate::{
        operations::Operation,
        query_builder::{Expression, QueryBuilder},
    };
    use sqlx::{Database, Executor, Pool};

    #[allow(non_camel_case_types)]
    pub struct expression_to_operation<E>(pub E);

    impl<S, E> Operation<S> for expression_to_operation<E>
    where
        E: for<'q> Expression<'q, S>,
        E: Send,
        S: DatabaseExt,
        for<'c> &'c mut <S as sqlx::Database>::Connection: Executor<'c, Database = S>,
    {
        type Output = ();
        async fn exec_operation(self, pool: Pool<S>) -> Self::Output
        where
            S: Database,
        {
            let mut qb = QueryBuilder::default();
            self.0.expression(&mut qb);
            use_executor!(execute(&pool, qb)).unwrap();
        }
    }
}
