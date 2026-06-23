use std::ops::Not;

use crate::sqlx_query_builder::{Expression, ManyExpressions, OpExpression};

pub struct InsertStatement<TableName, Identifiers, Values, Returning> {
    pub table_name: TableName,
    pub identifiers: Identifiers,
    pub values: Values,
    pub returning: Returning,
}

impl<TableName, Identifiers, Values, Returning> OpExpression
    for InsertStatement<TableName, Identifiers, Values, Returning>
{
}

/// used to avoid conflicting implementaion with Vec<T> in generic implementations like `Expression`
pub struct One<T>(pub T);

impl<'q, S, TableName, Identifiers, Values, Returning> Expression<'q, S>
    for InsertStatement<TableName, Identifiers, One<Values>, Returning>
where
    TableName: Expression<'q, S> + 'q,
    Identifiers: ManyExpressions<'q, S> + 'q,
    Values: ManyExpressions<'q, S> + 'q,
    Returning: ManyExpressions<'q, S> + 'q,
{
    fn expression(self, ctx: &mut crate::sqlx_query_builder::StatementBuilder<'q, S>)
    where
        S: crate::database_extention::DatabaseExt,
    {
        if self.identifiers.is_op().not() || self.values.0.is_op().not() {
            panic!("insert_statment with empty values")
        }
        ctx.syntax("INSERT INTO ");
        self.table_name.expression(ctx);
        ctx.syntax(" ");
        ctx.syntax("(");
        self.identifiers.expression("", ", ", ctx);
        ctx.syntax(")");
        ctx.syntax(" VALUES ");
        ctx.syntax("(");
        self.values.0.expression("", ", ", ctx);
        ctx.syntax(")");

        if self.returning.is_op() {
            self.returning.expression(" RETURNING ", ", ", ctx);
        }

        ctx.syntax(";");
    }
}

impl<'q, S, TableName, Identifiers, Values, Returning> Expression<'q, S>
    for InsertStatement<TableName, Identifiers, Vec<Values>, Returning>
where
    TableName: Expression<'q, S> + 'q,
    Identifiers: ManyExpressions<'q, S> + 'q,
    Values: ManyExpressions<'q, S> + 'q,
    Returning: ManyExpressions<'q, S> + 'q,
{
    fn expression(mut self, ctx: &mut crate::sqlx_query_builder::StatementBuilder<'q, S>)
    where
        S: crate::database_extention::DatabaseExt,
    {
        ctx.syntax("INSERT INTO ");
        self.table_name.expression(ctx);
        ctx.syntax(" ");
        ctx.syntax("(");
        self.identifiers.expression("", ", ", ctx);
        ctx.syntax(")");
        ctx.syntax(" VALUES ");

        let pop = self.values.pop();
        for each in self.values {
            ctx.syntax("(");
            each.expression("", ", ", ctx);
            ctx.syntax(")");
            ctx.syntax(", ");
        }

        if let Some(last) = pop {
            ctx.syntax("(");
            last.expression("", ", ", ctx);
            ctx.syntax(")");
        }

        if self.returning.is_op() {
            self.returning.expression(" RETURNING ", ", ", ctx);
        }

        ctx.syntax(";");
    }
}
