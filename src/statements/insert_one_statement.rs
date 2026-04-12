use std::ops::Not;

use crate::{
    query_builder::{
        Expression, ManyExpressions, OpExpression, QueryBuilder,
        syntax::{
            close_paranthesis, comma_join, empty, end_of_statement, open_paranthesis, space_join,
        },
    },
    sql_syntax,
};

pub struct InsertStatement<TableName, Identifiers, Values, Returning> {
    pub table_name: TableName,
    pub identifiers: Identifiers,
    pub values: Values,
    pub returning: Returning,
}

pub struct InsertManyStatement<TableName, Identifiers, Values, Returning> {
    pub table_name: TableName,
    pub identifiers: Identifiers,
    pub values: Vec<Values>,
    pub returning: Returning,
}

impl<TableName, Identifiers, Values, Returning> OpExpression
    for InsertStatement<TableName, Identifiers, Values, Returning>
{
}

sql_syntax!(insert_start = "INSERT INTO ");
sql_syntax!(values_join = " VALUES ");
sql_syntax!(returning_join = " RETURNING ");

#[allow(non_camel_case_types)]
pub struct values_for_insert<T>(pub T);

impl<T> OpExpression for values_for_insert<T> {}

impl<'q, S, T> Expression<'q, S> for values_for_insert<T>
where
    T: ManyExpressions<'q, S> + 'q,
{
    fn expression(self, ctx: &mut QueryBuilder<'q, S>)
    where
        S: crate::database_extention::DatabaseExt,
    {
        ctx.syntax(&open_paranthesis);
        self.0.expression(&empty, &comma_join, ctx);
        ctx.syntax(&close_paranthesis);
    }
}

#[allow(non_camel_case_types)]
pub struct values_for_insert_vec<T>(pub Vec<T>);

impl<T> OpExpression for values_for_insert_vec<T> {}

impl<'q, S, T> Expression<'q, S> for values_for_insert_vec<T>
where
    T: ManyExpressions<'q, S> + 'q,
{
    fn expression(mut self, ctx: &mut crate::query_builder::QueryBuilder<'q, S>)
    where
        S: crate::database_extention::DatabaseExt,
    {
        let last = self.0.pop();
        for each in self.0 {
            ctx.syntax(&open_paranthesis);
            each.expression(&empty, &comma_join, ctx);
            ctx.syntax(&close_paranthesis);
            ctx.syntax(&comma_join);
        }
        if let Some(last) = last {
            ctx.syntax(&open_paranthesis);
            last.expression(&empty, &comma_join, ctx);
            ctx.syntax(&close_paranthesis);
        }
    }
}

pub struct OneDefault<T>(pub T);

impl<'q, S, TableName, Identifiers, Values, Returning> Expression<'q, S>
    for InsertStatement<TableName, Identifiers, OneDefault<Values>, Returning>
where
    TableName: Expression<'q, S> + 'q,
    Identifiers: ManyExpressions<'q, S> + 'q,
    Values: ManyExpressions<'q, S> + 'q,
    Returning: ManyExpressions<'q, S> + 'q,
{
    fn expression(self, ctx: &mut crate::query_builder::QueryBuilder<'q, S>)
    where
        S: crate::database_extention::DatabaseExt,
    {
        if self.identifiers.is_op().not() || self.values.0.is_op().not() {
            panic!("insert_statment with empty values")
        }
        ctx.syntax(&insert_start);
        self.table_name.expression(ctx);
        ctx.syntax(&space_join);
        ctx.syntax(&open_paranthesis);
        self.identifiers.expression(&empty, &comma_join, ctx);
        ctx.syntax(&close_paranthesis);
        ctx.syntax(&values_join);
        ctx.syntax(&open_paranthesis);
        self.values.0.expression(&empty, &comma_join, ctx);
        ctx.syntax(&close_paranthesis);

        if self.returning.is_op() {
            self.returning.expression(&returning_join, &comma_join, ctx);
        }

        ctx.syntax(&end_of_statement);
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
    fn expression(mut self, ctx: &mut crate::query_builder::QueryBuilder<'q, S>)
    where
        S: crate::database_extention::DatabaseExt,
    {
        ctx.syntax(&insert_start);
        self.table_name.expression(ctx);
        ctx.syntax(&space_join);
        ctx.syntax(&open_paranthesis);
        self.identifiers.expression(&empty, &comma_join, ctx);
        ctx.syntax(&close_paranthesis);
        ctx.syntax(&values_join);

        let pop = self.values.pop();
        for each in self.values {
            ctx.syntax(&open_paranthesis);
            each.expression(&empty, &comma_join, ctx);
            ctx.syntax(&close_paranthesis);
            ctx.syntax(&comma_join);
        }

        if let Some(last) = pop {
            ctx.syntax(&open_paranthesis);
            last.expression(&empty, &comma_join, ctx);
            ctx.syntax(&close_paranthesis);
        }

        if self.returning.is_op() {
            self.returning.expression(&returning_join, &comma_join, ctx);
        }

        ctx.syntax(&end_of_statement);
    }
}
