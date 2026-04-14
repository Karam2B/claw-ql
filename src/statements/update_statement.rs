use crate::database_extention::DatabaseExt;
use crate::query_builder::{Expression, ManyExpressions, OpExpression, StatementBuilder};

pub struct UpdateStatement<TableName, Values, Wheres, Returning> {
    pub table_name: TableName,
    pub values: Values,
    pub wheres: Wheres,
    pub returning: Returning,
}

impl<TableName, Values, Wheres, Returning> OpExpression
    for UpdateStatement<TableName, Values, Wheres, Returning>
{
}

impl<'q, S, TableName, Values, Wheres, Returning> Expression<'q, S>
    for UpdateStatement<TableName, Values, Wheres, Returning>
where
    S: DatabaseExt,
    TableName: Expression<'q, S> + 'q,
    Values: ManyExpressions<'q, S> + 'q,
    Wheres: ManyExpressions<'q, S> + 'q,
    Returning: ManyExpressions<'q, S> + 'q,
{
    fn expression(self, ctx: &mut StatementBuilder<'q, S>) {
        ctx.syntax("UPDATE ");
        self.table_name.expression(ctx);
        ctx.syntax(" SET ");
        self.values.expression("", ", ", ctx);
        self.wheres.expression(" WHERE ", " AND ", ctx);
        self.returning.expression(" RETURNING ", ", ", ctx);

        ctx.syntax(";");
    }
}
