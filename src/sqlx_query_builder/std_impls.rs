use std::sync::Arc;

use crate::{
    database_extention::DatabaseExt,
    sqlx_query_builder::{
        Expression, IsOpExpression, ManyExpressions, OpExpression, StatementBuilder,
    },
};

impl OpExpression for String {}
impl<'q, S> Expression<'q, S> for String
where
    S: DatabaseExt,
{
    fn expression(self, ctx: &mut StatementBuilder<'q, S>)
    where
        S: DatabaseExt,
    {
        ctx.sanitize(self.as_str());
    }
}

impl OpExpression for Arc<str> {}
impl<'q, S> Expression<'q, S> for Arc<str>
where
    S: DatabaseExt,
{
    fn expression(self, ctx: &mut StatementBuilder<'q, S>)
    where
        S: DatabaseExt,
    {
        ctx.sanitize(self.as_ref());
    }
}

impl OpExpression for crate::sub_arc::ArcSubStr {}
impl<'q, S> Expression<'q, S> for crate::sub_arc::ArcSubStr
where
    S: DatabaseExt,
{
    fn expression(self, ctx: &mut StatementBuilder<'q, S>)
    where
        S: DatabaseExt,
    {
        ctx.sanitize(self.as_str());
    }
}

impl OpExpression for &'_ str {}
impl<'a, 'q, S> Expression<'q, S> for &'a str
where
    'a: 'q,
    S: DatabaseExt,
{
    fn expression(self, ctx: &mut StatementBuilder<'q, S>)
    where
        S: DatabaseExt,
    {
        ctx.sanitize(self);
    }
}

impl<T> IsOpExpression for &'_ [T]
where
    T: IsOpExpression,
{
    fn is_op(&self) -> bool {
        self.iter().any(|each| each.is_op())
    }
}
impl<'q, S, T> ManyExpressions<'q, S> for &'q [T]
where
    T: Clone + Expression<'q, S> + 'q,
{
    fn expression(self, start: &'static str, join: &'static str, ctx: &mut StatementBuilder<'q, S>)
    where
        S: DatabaseExt,
    {
        for (i, each) in self.into_iter().enumerate() {
            if i == 0 {
                ctx.syntax(start);
            } else {
                ctx.syntax(join);
            }
            each.clone().expression(ctx);
        }
    }
}

impl<T, const N: usize> IsOpExpression for [T; N]
where
    T: IsOpExpression,
{
    fn is_op(&self) -> bool {
        self.iter().any(|each| each.is_op())
    }
}
impl<'q, S, T, const N: usize> ManyExpressions<'q, S> for [T; N]
where
    T: Expression<'q, S> + 'q,
{
    fn expression(self, start: &'static str, join: &'static str, ctx: &mut StatementBuilder<'q, S>)
    where
        S: DatabaseExt,
    {
        for (i, each) in self.into_iter().enumerate() {
            if i == 0 {
                ctx.syntax(start);
            } else {
                ctx.syntax(join);
            }
            each.expression(ctx);
        }
    }
}

impl<T> IsOpExpression for Vec<T>
where
    T: IsOpExpression,
{
    fn is_op(&self) -> bool {
        self.iter().any(|each| each.is_op())
    }
}
impl<'q, S, T> ManyExpressions<'q, S> for Vec<T>
where
    T: Expression<'q, S> + 'q,
{
    fn expression(self, start: &'static str, join: &'static str, ctx: &mut StatementBuilder<'q, S>)
    where
        S: DatabaseExt,
    {
        for each in self {
            if each.is_op() {
                ctx.syntax(start);
            } else {
                ctx.syntax(join);
            }
            each.expression(ctx);
        }
    }
}
