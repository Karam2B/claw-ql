use serde::Serialize;
use sqlx::{Database, Pool};

pub mod fetch_one;
// pub mod delete_one;
// pub mod insert_one;
// pub mod fetch_many;
// pub mod update_one;

pub trait Operation<S>: Send {
    type Output: Send;
    fn exec(self, pool: Pool<S>) -> impl Future<Output = Self::Output> + Send
    where
        S: Database;
}

#[derive(Clone, Debug, PartialEq, Serialize)]
pub struct LinkedOutput<Id, C, L> {
    pub id: Id,
    pub attributes: C,
    pub link: L,
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
    use futures::FutureExt;
    use std::{any::Any, pin::Pin};

    use crate::from_row::pre_alias;
    use crate::operations::{
        Operation,
        fetch_one::{LinkFetchOne, SelectStatementExtendableParts},
    };
    use crate::query_builder::functional_expr::ZeroOrMoreImplPossible;
    use paste::paste;
    use sqlx::{Database, Pool};

    pub trait BoxedOperation<S: Database>: Send {
        fn exec(
            self: Box<Self>,
            pool: Pool<S>,
        ) -> Pin<Box<dyn Future<Output = Box<dyn Any + Send>> + Send>>;
    }

    impl<T, S> BoxedOperation<S> for T
    where
        T: Send + Operation<S, Output: Send> + 'static,
        S: Database,
    {
        fn exec(
            self: Box<Self>,
            pool: Pool<S>,
        ) -> Pin<Box<dyn Future<Output = Box<dyn Any + Send>> + Send>> {
            Box::pin(async {
                Operation::exec(*self, pool)
                    .map(|f| Box::new(f) as Box<dyn Any + Send>)
                    .await
            })
        }
    }

    impl<S> Operation<S> for Box<dyn BoxedOperation<S> + Send> {
        type Output = Box<dyn Any + Send>;
        fn exec(self, pool: Pool<S>) -> impl Future<Output = Self::Output> + Send
        where
            S: Database,
        {
            BoxedOperation::exec(self, pool)
        }
    }

    impl<S, T> Operation<S> for Vec<T>
    where
        T: Operation<S, Output: Send> + Send,
    {
        type Output = Vec<T::Output>;

        async fn exec(self, pool: sqlx::Pool<S>) -> Self::Output
        where
            S: sqlx::Database,
        {
            let mut v = vec![];
            for each in self {
                v.push(each.exec(pool.clone()).await);
            }
            v
        }
    }

    impl<T, S> LinkFetchOne<S> for Vec<T>
    where
        T: LinkFetchOne<S, SubOp: Operation<S>, Joins: IntoIterator, Wheres: IntoIterator>,
    {
        type Joins = Vec<<T::Joins as IntoIterator>::Item>;

        type Wheres = Vec<<T::Wheres as IntoIterator>::Item>;

        fn extend_select(
            &self,
        ) -> SelectStatementExtendableParts<
            Vec<crate::expressions::scoped_column<String, String>>,
            Self::Joins,
            Self::Wheres,
        > {
            let mut parts = SelectStatementExtendableParts {
                non_aggregating_select_items: vec![],
                non_duplicating_joins: vec![],
                wheres: vec![],
            };
            for each in self {
                let each = each.extend_select();
                parts
                    .non_aggregating_select_items
                    .extend(each.non_aggregating_select_items);
                parts
                    .non_duplicating_joins
                    .extend(each.non_duplicating_joins);
                parts.wheres.extend(each.wheres);
            }
            parts
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

    async fn exec(self, pool: sqlx::Pool<S>) -> Self::Output
    where
        S: sqlx::Database,
    {
        ($(
            paste!(self.$part).exec(pool.clone()).await,
        )*)
    }
}

impl <$($t,)* S> LinkFetchOne<S> for ($($t,)*)
where
    $($t: LinkFetchOne<S, SubOp: Operation<S>>,)*
{
    type Joins = (
        $(ZeroOrMoreImplPossible<$t::Joins>,)*
    );

    type Wheres = (
        $(ZeroOrMoreImplPossible<$t::Wheres>,)*
    );

    fn extend_select(
        &self,
    ) -> SelectStatementExtendableParts<
        Vec<crate::expressions::scoped_column<String, String>>,
        Self::Joins,
        Self::Wheres,
    > {
        let mut select_items = vec![];
        $(
            paste!{let  [<each_ $part>]  = self.$part.extend_select();}
            select_items.extend(paste!( [<each_ $part>] ).non_aggregating_select_items);
        )*

        SelectStatementExtendableParts {
            non_aggregating_select_items: select_items,
            non_duplicating_joins: ($(
                ZeroOrMoreImplPossible {
                    start: "",
                    join: ", ",
                    expressions: paste!([<each_ $part>]).non_duplicating_joins,
                },
            )*),
            wheres: ($(
                ZeroOrMoreImplPossible {
                    start: "",
                    join: ", ",
                    expressions: paste!( [<each_ $part>] ).wheres,
                },
            )*),
        }
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
    };
}

    implt!();
    implt!([T0, 0]);
    implt!([T0, 0] [T1, 1]);
    implt!([T0, 0] [T1, 1] [T2, 2]);
    implt!([T0, 0] [T1, 1] [T2, 2] [T3, 3]);
}
