use crate::query_builder::{Expression, ManyExpressions, OpExpression};

pub struct DeleteStatement<TableName, Wheres, Returning> {
    pub table_name: TableName,
    pub wheres: Wheres,
    pub returning: Returning,
}

impl<T, W, R> OpExpression for DeleteStatement<T, W, R> {}

impl<'q, S, TableName, Wheres, Returning> Expression<'q, S>
    for DeleteStatement<TableName, Wheres, Returning>
where
    TableName: Expression<'q, S> + 'q,
    Wheres: ManyExpressions<'q, S> + 'q,
    Returning: ManyExpressions<'q, S> + 'q,
{
    fn expression(self, ctx: &mut crate::query_builder::StatementBuilder<'q, S>)
    where
        S: crate::database_extention::DatabaseExt,
    {
        ctx.syntax("DELETE FROM ");
        self.table_name.expression(ctx);
        self.wheres.expression(" WHERE ", " AND ", ctx);
        self.returning.expression(" RETURNING ", ", ", ctx);
        ctx.syntax(";");
    }
}
