use crate::{
    database_extention::DatabaseExt,
    sqlx_query_builder::{
        Expression, IsOpExpression, ManyExpressions, OpExpression, StatementBuilder,
    },
};

pub trait ManyBoxedExpressions<S> {
    fn dyn_is_op(&self) -> bool;
    fn dyn_boxed_expression<'q>(
        self: Box<Self>,
        start: &'static str,
        join: &'static str,
        ctx: &mut StatementBuilder<'q, S>,
    ) where
        S: DatabaseExt;
}

impl<S, T> ManyBoxedExpressions<S> for T
where
    T: for<'q> ManyExpressions<'q, S> + Send,
{
    fn dyn_is_op(&self) -> bool {
        T::is_op(self)
    }
    fn dyn_boxed_expression<'q>(
        self: Box<Self>,
        start: &'static str,
        join: &'static str,
        ctx: &mut StatementBuilder<'q, S>,
    ) where
        S: DatabaseExt,
    {
        self.expression(start, join, ctx);
    }
}

impl<S> IsOpExpression for Box<dyn ManyBoxedExpressions<S> + Send> {
    fn is_op(&self) -> bool {
        ManyBoxedExpressions::dyn_is_op(&**self)
    }
}

impl<'q, S: 'q> ManyExpressions<'q, S> for Box<dyn ManyBoxedExpressions<S> + Send> {
    fn expression(self, start: &'static str, join: &'static str, ctx: &mut StatementBuilder<'q, S>)
    where
        S: DatabaseExt,
    {
        self.dyn_boxed_expression(start, join, ctx);
    }
}

pub trait BoxedExpression<S: DatabaseExt>: Send {
    fn boxed_expression<'q>(self: Box<Self>, ctx: &mut StatementBuilder<'q, S>);
}
impl<E, S> BoxedExpression<S> for E
where
    S: DatabaseExt,
    E: for<'e> Expression<'e, S> + Send,
{
    fn boxed_expression<'q>(self: Box<Self>, ctx: &mut StatementBuilder<'q, S>) {
        Expression::expression(*self, ctx);
    }
}

impl<S> OpExpression for Box<dyn BoxedExpression<S> + Send> where S: DatabaseExt {}

impl<'q, S> Expression<'q, S> for Box<dyn BoxedExpression<S> + Send>
where
    S: DatabaseExt,
{
    fn expression(self, ctx: &mut StatementBuilder<'q, S>) {
        BoxedExpression::boxed_expression(self, ctx);
    }
}
