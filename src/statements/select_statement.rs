use std::ops::Not;

use crate::{
    database_extention::DatabaseExt,
    query_builder::{
        Expression, ManyExpressions, OpExpression, PossibleExpression, StatementBuilder,
    },
};

pub struct SelectStatement<SelectItems, From, Joins, Wheres, GroupBy, Order, Limit> {
    pub select_items: SelectItems,
    pub from: From,
    pub joins: Joins,
    pub wheres: Wheres,
    pub group_by: GroupBy,
    pub order: Order,
    pub limit: Limit,
}

impl<SelectItems, From, Joins, GroupBy, Wheres, Limit, Order> OpExpression
    for SelectStatement<SelectItems, From, Joins, GroupBy, Wheres, Limit, Order>
{
}

impl<'q, S, SelectItems, From, Joins, Wheres, Limit, Order, GroupBy> Expression<'q, S>
    for SelectStatement<SelectItems, From, Joins, Wheres, GroupBy, Order, Limit>
where
    SelectItems: ManyExpressions<'q, S> + 'q,
    From: Expression<'q, S> + 'q,
    Joins: ManyExpressions<'q, S> + 'q,
    GroupBy: ManyExpressions<'q, S> + 'q,
    Wheres: ManyExpressions<'q, S> + 'q,
    Limit: PossibleExpression<'q, S> + 'q,
    Order: ManyExpressions<'q, S> + 'q,
{
    #[track_caller]
    fn expression(self, ctx: &mut StatementBuilder<'q, S>)
    where
        S: DatabaseExt,
    {
        ctx.syntax("SELECT ");
        if self.select_items.is_op().not() {
            panic!("empty select item")
        }
        self.select_items.expression("", ", ", ctx);

        ctx.syntax(" FROM ");
        self.from.expression(ctx);
        self.joins.expression(" ", ", ", ctx);
        self.wheres.expression(" WHERE ", " AND ", ctx);
        self.group_by.expression(" GROUP BY ", ", ", ctx);
        self.order.expression(" ORDER BY ", ", ", ctx);

        self.limit.expression_starting(" LIMIT ", ctx);
        ctx.syntax(";");
    }
}
