use std::ops::Not;

use crate::{
    database_extention::DatabaseExt,
    query_builder::{Expression, ManyExpressions, OpExpression, StatementBuilder},
};
use sqlx::{Encode, Type};

pub struct ColumnNotEqual<Col, Ne> {
    pub col: Col,
    pub ne: Ne,
}

impl<Col, Ne> OpExpression for ColumnNotEqual<Col, Ne> {}

impl<'q, S, Col, Ne> Expression<'q, S> for ColumnNotEqual<Col, Ne>
where
    S: DatabaseExt,
    Ne: 'q + Encode<'q, S> + Type<S>,
    Col: Expression<'q, S> + 'q,
{
    fn expression(self, arg: &mut StatementBuilder<'q, S>) {
        self.col.expression(arg);
        arg.syntax(" != ");
        arg.bind(self.ne);
    }
}

macro_rules! column_compare {
    ($name:ident, $op:literal) => {
        pub struct $name<Col, Val> {
            pub col: Col,
            pub val: Val,
        }

        impl<Col, Val> OpExpression for $name<Col, Val> {}

        impl<'q, S, Col, Val> Expression<'q, S> for $name<Col, Val>
        where
            S: DatabaseExt,
            Val: 'q + Encode<'q, S> + Type<S>,
            Col: Expression<'q, S> + 'q,
        {
            fn expression(self, arg: &mut StatementBuilder<'q, S>) {
                self.col.expression(arg);
                arg.syntax($op);
                arg.bind(self.val);
            }
        }
    };
}

column_compare!(ColumnGreaterThan, " > ");
column_compare!(ColumnGreaterThanOrEqual, " >= ");
column_compare!(ColumnLessThan, " < ");
column_compare!(ColumnLessThanOrEqual, " <= ");

pub struct ColumnContains<Col, Val> {
    pub col: Col,
    pub val: Val,
}

impl<Col, Val> OpExpression for ColumnContains<Col, Val> {}

impl<'q, S, Col, Val> Expression<'q, S> for ColumnContains<Col, Val>
where
    S: DatabaseExt,
    Val: 'q + Encode<'q, S> + Type<S>,
    Col: Expression<'q, S> + 'q,
{
    fn expression(self, arg: &mut StatementBuilder<'q, S>) {
        self.col.expression(arg);
        arg.syntax(" LIKE ");
        arg.bind(self.val);
    }
}

pub struct ColumnIsNull<Col> {
    pub col: Col,
}

impl<Col> OpExpression for ColumnIsNull<Col> {}

impl<'q, S, Col> Expression<'q, S> for ColumnIsNull<Col>
where
    S: DatabaseExt,
    Col: Expression<'q, S> + 'q,
{
    fn expression(self, arg: &mut StatementBuilder<'q, S>) {
        self.col.expression(arg);
        arg.syntax(" IS NULL");
    }
}

pub struct ColumnIsNotNull<Col> {
    pub col: Col,
}

impl<Col> OpExpression for ColumnIsNotNull<Col> {}

impl<'q, S, Col> Expression<'q, S> for ColumnIsNotNull<Col>
where
    S: DatabaseExt,
    Col: Expression<'q, S> + 'q,
{
    fn expression(self, arg: &mut StatementBuilder<'q, S>) {
        self.col.expression(arg);
        arg.syntax(" IS NOT NULL");
    }
}

pub struct FilterAnd<T>(pub T);

impl<T> OpExpression for FilterAnd<T> {}

impl<'q, S, T> Expression<'q, S> for FilterAnd<T>
where
    S: DatabaseExt,
    T: ManyExpressions<'q, S> + 'q,
{
    fn expression(self, ctx: &mut StatementBuilder<'q, S>) {
        if self.0.is_op().not() {
            return;
        }
        ctx.syntax("(");
        self.0.expression("", " AND ", ctx);
        ctx.syntax(")");
    }
}

pub struct FilterOr<T>(pub T);

impl<T> OpExpression for FilterOr<T> {}

impl<'q, S, T> Expression<'q, S> for FilterOr<T>
where
    S: DatabaseExt,
    T: ManyExpressions<'q, S> + 'q,
{
    fn expression(self, ctx: &mut StatementBuilder<'q, S>) {
        if self.0.is_op().not() {
            return;
        }
        ctx.syntax("(");
        self.0.expression("", " OR ", ctx);
        ctx.syntax(")");
    }
}

pub struct FilterGroup<T>(pub T);

impl<T> OpExpression for FilterGroup<T> {}

impl<'q, S, T> Expression<'q, S> for FilterGroup<T>
where
    S: DatabaseExt,
    T: ManyExpressions<'q, S> + 'q,
{
    fn expression(self, ctx: &mut StatementBuilder<'q, S>) {
        if self.0.is_op().not() {
            return;
        }
        ctx.syntax("(");
        self.0.expression("", " AND ", ctx);
        ctx.syntax(")");
    }
}
