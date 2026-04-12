use crate::database_extention::DatabaseExt;
use crate::query_builder::syntax::{and_join, comma_join, empty, end_of_statement};
use crate::query_builder::{
    Expression, ManyExpressions, OpExpression, PossibleExpression, QueryBuilder,
};
use crate::sql_syntax;
use crate::statements::insert_one_statement::returning_join;
use crate::statements::select_statement::where_join;

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

sql_syntax!(update_start = "UPDATE ");
sql_syntax!(set_join = " SET ");

impl<'q, S, TableName, Values, Wheres, Returning> Expression<'q, S>
    for UpdateStatement<TableName, Values, Wheres, Returning>
where
    S: DatabaseExt,
    TableName: Expression<'q, S> + 'q,
    Values: ManyExpressions<'q, S> + 'q,
    Wheres: ManyExpressions<'q, S> + 'q,
    Returning: PossibleExpression<'q, S> + 'q,
{
    fn expression(self, ctx: &mut QueryBuilder<'q, S>) {
        ctx.syntax(&update_start);
        self.table_name.expression(ctx);
        ctx.syntax(&set_join);
        self.values.expression(&empty, &comma_join, ctx);
        self.wheres.expression(&where_join, &and_join, ctx);

        if self.returning.is_op() {
            ctx.syntax(&returning_join);
            self.returning.expression(ctx);
        }

        ctx.syntax(&end_of_statement);
    }
}
