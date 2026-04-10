use crate::{
    IsOpExpression, ZeroOrMoreExpressions,
    from_row::pre_alias,
    functional_expr::ZeroOrMoreImplPossible,
    links::Link,
    operations::{
        Operation,
        fetch_one::{LinkFetchOne, SelectStatementExtendableParts},
    },
};
use paste::paste;

fn mamdlkadjsfljkasdf() {
    let paste!([<hi eher>]) = "sdfkjsld";
    let ss = hieher;
}

macro_rules! implt {
    ( $([$t:ident, $part:literal])*) => {
        #[allow(unused)]
        impl<$($t,)* Base> Link<Base> for ($($t,)*)
        where
            $($t: Link<Base>,)*
        {
            type Spec = ( $($t::Spec,)* );
            fn spec(self, base: &Base) -> Self::Spec {
                ( $(paste!(self.$part).spec(base),)* )
            }
        }
    };
}

implt!();
implt!([T0, 0]);
implt!([T0, 0] [T1, 1]);
implt!([T0, 0] [T1, 1] [T2, 2]);
implt!([T0, 0] [T1, 1] [T2, 2] [T3, 3]);

impl<Base, T> Link<Base> for Vec<T>
where
    T: Link<Base>,
{
    type Spec = Vec<T::Spec>;

    fn spec(self, base: &Base) -> Self::Spec {
        self.into_iter().map(|e| e.spec(base)).collect()
    }
}

// mod functional_impl_op {
//     use crate::operations::Operation;

//     impl<S, T0, T1> Operation<S> for (T0, T1)
//     where
//         T0: Operation<S>,
//         T1: Operation<S>,
//     {
//         type Output = (T0::Output, T1::Output);
//         async fn exec(self, pool: sqlx::Pool<S>) -> Self::Output
//         where
//             S: sqlx::Database,
//         {
//             (
//                 self.0.exec(pool.clone()).await,
//                 self.1.exec(pool.clone()).await,
//             )
//         }
//     }

// }

pub struct ZeroOrMoreFlatten<T>(pub T);
impl<T> IsOpExpression for ZeroOrMoreFlatten<T> {
    fn is_op(&self) -> bool {
        todo!()
    }
}
impl<'q, S, T> ZeroOrMoreExpressions<'q, S> for ZeroOrMoreFlatten<T>
where
    T: ZeroOrMoreExpressions<'q, S> + 'q,
{
    fn to_expr(self) -> Vec<Box<dyn crate::functional_expr::BoxedExpression<'q, S> + 'q>>
    where
        Self: Sized,
    {
        todo!()
    }

    fn expression(
        self,
        start: &'static str,
        join: &'static str,
        ctx: &mut crate::QueryBuilder<'q, S>,
    ) where
        S: crate::DatabaseExt,
    {
        todo!()
    }
}

// impl<T0, T1, S> LinkFetchOne<S> for (T0, T1)
// where
//     T0: LinkFetchOne<S>,
//     T1: LinkFetchOne<S>,
// {
//     type Joins = (
//         ZeroOrMoreImplPossible<T0::Joins>,
//         ZeroOrMoreImplPossible<T1::Joins>,
//     );

//     type Wheres = (
//         ZeroOrMoreImplPossible<T0::Wheres>,
//         ZeroOrMoreImplPossible<T1::Wheres>,
//     );

//     fn extend_select(
//         &self,
//     ) -> SelectStatementExtendableParts<
//         Vec<crate::expressions::scoped_column<String, String>>,
//         Self::Joins,
//         Self::Wheres,
//     > {
//         let mut select_items = vec![];
//         let each_0 = self.0.extend_select();
//         select_items.extend(each_0.non_aggregating_select_items);
//         let each_1 = self.1.extend_select();
//         select_items.extend(each_1.non_aggregating_select_items);

//         SelectStatementExtendableParts {
//             non_aggregating_select_items: select_items,
//             non_duplicating_joins: (
//                 ZeroOrMoreImplPossible {
//                     start: "",
//                     join: ", ",
//                     expressions: each_0.non_duplicating_joins,
//                 },
//                 ZeroOrMoreImplPossible {
//                     start: "",
//                     join: ", ",
//                     expressions: each_1.non_duplicating_joins,
//                 },
//             ),
//             wheres: (
//                 ZeroOrMoreImplPossible {
//                     start: "",
//                     join: ", ",
//                     expressions: each_0.wheres,
//                 },
//                 ZeroOrMoreImplPossible {
//                     start: "",
//                     join: ", ",
//                     expressions: each_1.wheres,
//                 },
//             ),
//         }
//     }

//     type Inner = (T0::Inner, T1::Inner);

//     type SubOp = (T0::SubOp, T1::SubOp);

//     fn sub_op(
//         &self,
//         row: crate::from_row::pre_alias<'_, <S as sqlx::Database>::Row>,
//     ) -> (Self::SubOp, Self::Inner)
//     where
//         S: sqlx::Database,
//     {
//         todo!()
//     }

//     type Output = (T0::Output, T1::Output);

//     fn take(
//         self,
//         extend: <Self::SubOp as crate::operations::Operation<S>>::Output,
//         inner: Self::Inner,
//     ) -> Self::Output {
//         todo!()
//     }
// }

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
