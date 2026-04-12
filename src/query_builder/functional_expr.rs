use crate::query_builder::SqlSyntax;
use crate::query_builder::syntax::empty;

use super::QueryBuilder;
use super::{
    DatabaseExt, Expression, IsOpExpression, ManyExpressions, OpExpression, PossibleExpression,
};
use std::ops::Not;

/// Expression is dyn-compatable, but Expression::expression is not callable
/// because of `dyn Expression: !Sized`
/// the blanket implementation for this trait solve this issue
pub trait BoxedExpression<'q, S> {
    fn expression(self: Box<Self>, ctx: &mut QueryBuilder<'q, S>)
    where
        S: DatabaseExt;
}

impl<'q, S, T> BoxedExpression<'q, S> for T
where
    T: Expression<'q, S> + 'q,
{
    fn expression(self: Box<Self>, ctx: &mut QueryBuilder<'q, S>)
    where
        S: DatabaseExt,
    {
        Expression::expression(*self, ctx);
    }
}

impl<'q, S> OpExpression for Box<dyn BoxedExpression<'q, S> + 'q> {}

// full circle!
impl<'q, S: 'q> Expression<'q, S> for Box<dyn BoxedExpression<'q, S> + 'q> {
    fn expression(self, ctx: &mut QueryBuilder<'q, S>)
    where
        S: DatabaseExt,
    {
        BoxedExpression::expression(self, ctx);
    }
}

/// Expression is dyn-compatable, but Expression::expression is not callable
/// because of `dyn Expression: !Sized`
/// the blanket implementation for this trait solve this issue
pub trait StaticExpression<S>: 'static + Send {
    fn expression(self: Box<Self>, ctx: &mut QueryBuilder<'static, S>)
    where
        S: DatabaseExt;
}

impl<S, T> StaticExpression<S> for T
where
    T: Expression<'static, S> + 'static + Send,
{
    fn expression(self: Box<Self>, ctx: &mut QueryBuilder<'static, S>)
    where
        S: DatabaseExt,
    {
        Expression::expression(*self, ctx);
    }
}

impl<'q, S> OpExpression for Box<dyn StaticExpression<S> + Send> {}

// full circle!
impl<S: 'static> Expression<'static, S> for Box<dyn StaticExpression<S> + Send> {
    fn expression(self, ctx: &mut QueryBuilder<'static, S>)
    where
        S: DatabaseExt,
    {
        StaticExpression::expression(self, ctx);
    }
}

pub fn boxed_expr<'q, T, S>(t: T) -> Box<dyn BoxedExpression<'q, S> + 'q>
where
    T: BoxedExpression<'q, S> + 'q,
{
    Box::new(t)
}

impl<T> IsOpExpression for T
where
    T: OpExpression,
{
    fn is_op(&self) -> bool {
        true
    }
}

impl<'q, S, T> PossibleExpression<'q, S> for T
where
    T: Expression<'q, S> + 'q,
{
    fn expression_starting<Start: SqlSyntax + ?Sized>(
        self,
        start: &Start,
        ctx: &mut QueryBuilder<'q, S>,
    ) where
        S: DatabaseExt,
    {
        ctx.syntax(start);
        Expression::expression(self, ctx);
    }
    fn expression(self, ctx: &mut QueryBuilder<'q, S>)
    where
        S: DatabaseExt,
    {
        Expression::expression(self, ctx);
    }
}
impl<T> IsOpExpression for Option<T> {
    fn is_op(&self) -> bool {
        self.is_some()
    }
}

impl<'q, T: 'q, S> PossibleExpression<'q, S> for Option<T>
where
    T: Expression<'q, S> + 'q,
{
    fn expression_starting<Start: SqlSyntax + ?Sized>(
        self,
        start: &Start,
        ctx: &mut QueryBuilder<'q, S>,
    ) where
        S: DatabaseExt,
    {
        if let Some(this) = self {
            ctx.syntax(start);
            this.expression(ctx);
        }
    }
    fn expression(self, ctx: &mut QueryBuilder<'q, S>)
    where
        S: DatabaseExt,
    {
        if let Some(this) = self {
            this.expression(ctx);
        }
    }
}

impl IsOpExpression for () {
    fn is_op(&self) -> bool {
        false
    }
}
impl<'q, S> PossibleExpression<'q, S> for () {
    fn expression_starting<Start: SqlSyntax + ?Sized>(self, _: &Start, _: &mut QueryBuilder<'q, S>)
    where
        S: DatabaseExt,
    {
    }
    fn expression(self, _: &mut QueryBuilder<'q, S>)
    where
        S: DatabaseExt,
    {
    }
}

pub struct ManyPossible<T>(pub T);

mod impl_many_expr {
    use super::super::BoxedExpression;
    use super::ManyPossible;
    use super::PossibleExprImplExpr;
    use crate::database_extention::DatabaseExt;
    use crate::query_builder::QueryBuilder;
    use crate::query_builder::SqlSyntax;
    use crate::query_builder::functional_expr::StaticExpression;
    use crate::query_builder::{
        IsOpExpression, ManyExpressions, PossibleExpression, ToStaticExpressions,
    };
    use paste::paste;

    macro_rules! implt {
        (
            $([$first:ident, $first_part:literal])?;
            $([$last:ident,  $last_part:literal ])?;
            $([$each:ident,  $part:literal      ])*
        ) => {

            #[allow(unused)]
            impl<S, $($first,)? $($each,)* $($last,)?> ToStaticExpressions<S> for ($($first,)? $($each,)* $($last,)?)
            where
                $($first: PossibleExpression<'static, S> + 'static + Send,)?
                $($each:  PossibleExpression<'static, S> + 'static + Send,)*
                $($last:  PossibleExpression<'static, S> + 'static + Send,)?
            {
                fn to_static_expr(self) -> Vec<Box<dyn StaticExpression<S> + Send>>
                where
                    Self: Sized
                {
                    let mut v: Vec<Box<dyn StaticExpression<S> + Send>> = vec![];

                    $(
                        if let Ok(s) = PossibleExprImplExpr::new(paste!(self.$first_part)) {
                            v.push(Box::new(s));
                        }
                    )?
                    $(
                        if let Ok(s) = PossibleExprImplExpr::new(paste!(self.$part)) {
                            v.push(Box::new(s));
                        }
                    )*
                    $(
                        if let Ok(s) = PossibleExprImplExpr::new(paste!(self.$last_part)) {
                            v.push(Box::new(s));
                        }
                    )?

                    v
                }
            }

            impl<$($first,)? $($each,)* $($last,)?> IsOpExpression for ManyPossible<($($first,)? $($each,)* $($last,)?)>
            where
                $($first: IsOpExpression,)?
                $($each:  IsOpExpression,)*
                $($last:  IsOpExpression,)?
            {
                fn is_op(&self) -> bool {
                    // if some are op then the entire tuple is op
                    false
                    $(|| paste!(self.0.$first_part).is_op())?
                    $(|| paste!(self.0.$part).is_op())?
                    $(|| paste!(self.0.$last_part).is_op())?
                }
            }

            #[allow(unused)]
            impl<'q, S, $($first,)? $($each,)* $($last,)?> ManyExpressions<'q, S> for ManyPossible<($($first,)? $($each,)* $($last,)?)>
            where
                $($first: PossibleExpression<'q, S> + 'q,)?
                $($each:  PossibleExpression<'q, S> + 'q,)*
                $($last:  PossibleExpression<'q, S> + 'q,)?
            {
                fn to_expr(self) -> Vec<Box<dyn BoxedExpression<'q, S> + 'q>>
                where Self: Sized
                {
                    let mut v: Vec<Box<dyn BoxedExpression<'_, S>>> = vec![ ];

                    $(
                        if let Ok(s) = PossibleExprImplExpr::new(paste!(self.0.$first_part)) {
                            v.push(Box::new(s));
                        }
                    )?
                    $(
                        if let Ok(s) = PossibleExprImplExpr::new(paste!(self.0.$part)) {
                            v.push(Box::new(s));
                        }
                    )*
                    $(
                        if let Ok(s) = PossibleExprImplExpr::new(paste!(self.0.$last_part)) {
                            v.push(Box::new(s));
                        }
                    )?

                   v
                }

                fn expression<St,J>(self,start: &St, join: &J, ctx: &mut QueryBuilder<'q, S>)
                where
                St: SqlSyntax + ?Sized,
                J: SqlSyntax + ?Sized,
                    S: DatabaseExt,
                {
                    let mut need_to_start = true;
                    $(
                        if paste!(self.0.$first_part).is_op() {
                            if need_to_start {
                                ctx.syntax(start);
                                need_to_start = false
                            }
                            paste!(self.0.$first_part).expression(ctx);
                        }
                    )?
                    $(
                        if paste!(self.0.$part).is_op() {
                            if need_to_start {
                                ctx.syntax(start);
                                need_to_start = false
                            } else {
                                ctx.syntax(join)
                            }
                            paste!(self.0.$part).expression(ctx);
                        }
                    )*
                    $(
                        if paste!(self.0.$last_part).is_op() {
                            if need_to_start {
                                ctx.syntax(start);
                            } else {
                                ctx.syntax(join)
                            }
                            paste!(self.0.$last_part).expression(ctx);
                        }
                    )?
                }
            }
        };
    }

    implt!(;;);
    implt!([R0, 0];;);
    implt!([R0, 0]; [R1, 1];);
    implt!([R0, 0]; [R2, 2]; [R1, 1]);
    implt!([R0, 0]; [R3, 3]; [R1, 1] [R2, 2]);
}

impl<T> IsOpExpression for Vec<T>
where
    T: IsOpExpression,
{
    fn is_op(&self) -> bool {
        self.is_empty().not() && self.iter().all(|e| e.is_op())
    }
}

impl<'q, T, S> ManyExpressions<'q, S> for Vec<T>
where
    T: PossibleExpression<'q, S> + 'q,
{
    fn to_expr(self) -> Vec<Box<dyn BoxedExpression<'q, S> + 'q>>
    where
        Self: Sized,
    {
        self.into_iter()
            .filter_map(|e| {
                PossibleExprImplExpr::new(e)
                    .map(|e| Box::new(e) as Box<dyn BoxedExpression<'_, S>>)
                    .ok()
            })
            .collect()
    }
    fn expression<Start: super::SqlSyntax + ?Sized, Join: super::SqlSyntax + ?Sized>(
        self,
        start: &Start,
        join: &Join,
        ctx: &mut QueryBuilder<'q, S>,
    ) where
        S: DatabaseExt,
    {
        let mut need_to_start = true;

        for each in self {
            if each.is_op().not() {
                continue;
            }

            if need_to_start {
                ctx.syntax(start);
                need_to_start = false
            } else {
                ctx.syntax(join);
            }
            each.expression(ctx);
        }
    }
    // fn expression(self, start: &'static str, join: &'static str, ctx: &mut QueryBuilder<'q, S>)
    // where
    //     S: DatabaseExt,
    // {

    // }
}

#[cfg(test)]
mod test {
    // use sqlx::Sqlite;
    // use crate::{PossibleExpression, expressions::col, functional_expr::ZeroOrMoreImplPossible};
    // #[test]
    // fn main() {
    //     let zomip = ZeroOrMoreImplPossible {
    //         start: "WHERE ",
    //         join: " JOIN ",
    //         expressions: vec![col("index".to_string()).eq(3)],
    //     };
    //     let mut ctx = crate::QueryBuilder::<'_, Sqlite>::default();
    //     // PossibleExpression::expression(zomip, &mut ctx);
    // }
}

pub struct ManyImplPossible<T> {
    pub start: &'static str,
    pub join: &'static str,
    pub expressions: T,
}

impl<T: IsOpExpression> IsOpExpression for ManyImplPossible<T> {
    fn is_op(&self) -> bool {
        self.expressions.is_op()
    }
}

impl<'q, S, T> PossibleExpression<'q, S> for ManyImplPossible<T>
where
    T: ManyExpressions<'q, S> + 'q,
{
    /// double start! maybe that can be intentional!
    fn expression_starting<Start: SqlSyntax + ?Sized>(
        self,
        start: &Start,
        ctx: &mut QueryBuilder<'q, S>,
    ) where
        S: DatabaseExt,
    {
        if self.is_op() {
            ctx.syntax(start);
            self.expressions.expression(&self.start, &self.join, ctx);
        }
    }

    fn expression(self, ctx: &mut QueryBuilder<'q, S>)
    where
        S: DatabaseExt,
    {
        self.expressions.expression(&self.start, &self.join, ctx);
    }
}

pub struct PossibleExprImplExpr<T>(T);

#[derive(Debug, Clone)]
pub struct PossibleExpressionCannotImplExpression;

impl<T> PossibleExprImplExpr<T> {
    pub fn new(possible_expr: T) -> Result<Self, PossibleExpressionCannotImplExpression>
    where
        T: IsOpExpression,
    {
        if IsOpExpression::is_op(&possible_expr) {
            Ok(Self(possible_expr))
        } else {
            Err(PossibleExpressionCannotImplExpression)
        }
    }
}

impl<T> OpExpression for PossibleExprImplExpr<T> {}

impl<'q, S, T> Expression<'q, S> for PossibleExprImplExpr<T>
where
    T: PossibleExpression<'q, S> + 'q,
{
    fn expression(self, ctx: &mut QueryBuilder<'q, S>)
    where
        S: DatabaseExt,
    {
        PossibleExpression::expression(self.0, ctx);
    }
}

pub struct ManyFlat<T>(pub T);

impl<T0> IsOpExpression for ManyFlat<(T0,)>
where
    T0: IsOpExpression,
{
    fn is_op(&self) -> bool {
        self.0.0.is_op()
    }
}

impl<'s, T0, S> ManyExpressions<'s, S> for ManyFlat<(T0,)>
where
    T0: ManyExpressions<'s, S>,
{
    fn to_expr(
        self,
    ) -> Vec<Box<dyn crate::prelude::macro_derive_collection::BoxedExpression<'s, S> + 's>>
    where
        Self: Sized,
    {
        todo!()
    }
    fn expression<Start: SqlSyntax + ?Sized, Join: SqlSyntax + ?Sized>(
        self,
        start: &Start,
        join: &Join,
        ctx: &mut QueryBuilder<'s, S>,
    ) where
        S: DatabaseExt,
    {
        if self.0.0.is_op() {
            ctx.syntax(start);
        }
        self.0.0.expression(&empty, join, ctx);
    }
}

impl<T0, T1> IsOpExpression for ManyFlat<(T0, T1)>
where
    T0: IsOpExpression,
    T1: IsOpExpression,
{
    fn is_op(&self) -> bool {
        self.0.0.is_op() || self.0.0.is_op()
    }
}

impl<'s, T0, T1, S> ManyExpressions<'s, S> for ManyFlat<(T0, T1)>
where
    T0: ManyExpressions<'s, S>,
    T1: ManyExpressions<'s, S>,
{
    fn to_expr(
        self,
    ) -> Vec<Box<dyn crate::prelude::macro_derive_collection::BoxedExpression<'s, S> + 's>>
    where
        Self: Sized,
    {
        todo!()
    }
    fn expression<Start: SqlSyntax + ?Sized, Join: SqlSyntax + ?Sized>(
        self,
        start: &Start,
        join: &Join,
        ctx: &mut QueryBuilder<'s, S>,
    ) where
        S: DatabaseExt,
    {
        let mut need_start = true;
        if need_start && self.0.0.is_op() {
            ctx.syntax(start);
            need_start = false;
        }
        self.0.0.expression(&empty, join, ctx);

        if need_start && self.0.1.is_op() {
            ctx.syntax(start);
        } else {
            ctx.syntax(join);
        }
        self.0.1.expression(&empty, join, ctx);
    }
}

impl<T0, T1, T2> IsOpExpression for ManyFlat<(T0, T1, T2)>
where
    T0: IsOpExpression,
    T1: IsOpExpression,
    T2: IsOpExpression,
{
    fn is_op(&self) -> bool {
        self.0.0.is_op() || self.0.0.is_op()
    }
}

impl<'s, T0, T1, T2, S> ManyExpressions<'s, S> for ManyFlat<(T0, T1, T2)>
where
    T0: ManyExpressions<'s, S>,
    T1: ManyExpressions<'s, S>,
    T2: ManyExpressions<'s, S>,
{
    fn to_expr(
        self,
    ) -> Vec<Box<dyn crate::prelude::macro_derive_collection::BoxedExpression<'s, S> + 's>>
    where
        Self: Sized,
    {
        todo!()
    }
    fn expression<Start: SqlSyntax + ?Sized, Join: SqlSyntax + ?Sized>(
        self,
        start: &Start,
        join: &Join,
        ctx: &mut QueryBuilder<'s, S>,
    ) where
        S: DatabaseExt,
    {
        let mut need_start = true;

        if need_start && self.0.0.is_op() {
            ctx.syntax(start);
            need_start = false;
        }
        self.0.0.expression(&empty, join, ctx);

        if need_start && self.0.1.is_op() {
            ctx.syntax(start);
            need_start = false
        }
        self.0.1.expression(&empty, join, ctx);

        if need_start && self.0.2.is_op() {
            ctx.syntax(start);
        }
        self.0.2.expression(&empty, join, ctx);
    }
}
