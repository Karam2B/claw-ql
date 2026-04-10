use std::ops::Not;

use crate::{
    DatabaseExt, Expression, OpExpression, PossibleExpression, QueryBuilder, ZeroOrMoreExpressions,
};

pub trait Inverse {
    type InverseStatement;
    fn inverse(&self) -> Self::InverseStatement;
}

pub struct SelectStatement<SelectItems, From, Joins, Wheres, Order, Limit> {
    pub select_items: SelectItems,
    pub from: From,
    pub joins: Joins,
    pub wheres: Wheres,
    pub order: Order,
    pub limit: Limit,
}

impl<SelectItems, From, Joins, Wheres, Limit, Order> OpExpression
    for SelectStatement<SelectItems, From, Joins, Wheres, Limit, Order>
{
}

impl<'q, S, SelectItems, From, Joins, Wheres, Limit, Order> Expression<'q, S>
    for SelectStatement<SelectItems, From, Joins, Wheres, Limit, Order>
where
    SelectItems: ZeroOrMoreExpressions<'q, S> + 'q,
    From: Expression<'q, S> + 'q,
    Joins: ZeroOrMoreExpressions<'q, S> + 'q,
    Wheres: ZeroOrMoreExpressions<'q, S> + 'q,
    Limit: PossibleExpression<'q, S> + 'q,
    Order: PossibleExpression<'q, S> + 'q,
{
    #[track_caller]
    fn expression(self, ctx: &mut QueryBuilder<'q, S>)
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
        self.joins.expression(" ", " ", ctx);
        self.wheres.expression(" WHERE ", " AND ", ctx);
        self.order.expression_starting(" ORDER ", ctx);
        self.limit.expression_starting(" LIMIT ", ctx);
        ctx.syntax(";");
    }
}

pub struct CreateTable<Init, TableName, ColDefs> {
    pub init: Init,
    pub name: TableName,
    pub col_defs: ColDefs,
}

pub struct create_table;

impl OpExpression for create_table {}

impl<'q, S> Expression<'q, S> for create_table {
    fn expression(self, ctx: &mut QueryBuilder<'q, S>)
    where
        S: DatabaseExt,
    {
        ctx.syntax("CREATE TABLE");
    }
}

pub struct create_if_not_exist;

impl OpExpression for create_if_not_exist {}

impl<'q, S> Expression<'q, S> for create_if_not_exist {
    fn expression(self, ctx: &mut QueryBuilder<'q, S>)
    where
        S: DatabaseExt,
    {
        ctx.syntax("CREATE TABLE IF NOT EXISTS");
    }
}

impl<Header, Table, Columns> OpExpression for CreateTable<Header, Table, Columns> {}

impl<'q, S, Header, Table, Columns> Expression<'q, S> for CreateTable<Header, Table, Columns>
where
    Header: Expression<'q, S> + 'q,
    Table: Expression<'q, S> + 'q,
    Columns: ZeroOrMoreExpressions<'q, S> + 'q,
{
    fn expression(self, ctx: &mut QueryBuilder<'q, S>)
    where
        S: DatabaseExt,
    {
        let open_b = "(";
        let close_b = ");";
        self.init.expression(ctx);
        ctx.syntax(" ");
        self.name.expression(ctx);
        // ctx.sanitize(self.name);
        ctx.syntax(" ");
        ctx.syntax(open_b);
        self.col_defs.expression("", ", ", ctx);
        ctx.syntax(close_b);
    }
}

mod impl_syntax_for_create_table {
    use crate::{
        ValidSyntax,
        collections::{Collection, Member, SingleIncremintalInt},
        expressions::col_def_for_collection_member,
        statements::{CreateTable, create_if_not_exist, create_table},
        syntax_trait::{ColIdent, CreateTableHeader, TableIdent},
    };

    impl<S, Header, Table, Columns> ValidSyntax<S> for CreateTable<Header, Table, Columns>
    where
        Header: CreateTableHeader<S>,
        Table: TableIdent,
        Columns: ColIdent<Table>,
    {
    }

    impl<S> CreateTableHeader<S> for create_if_not_exist {}
    impl<S> CreateTableHeader<S> for create_table {}

    impl<T> ColIdent<T> for SingleIncremintalInt where T: Collection<Id = Self> {}

    impl<T, C0> ColIdent<T> for (C0,) where C0: ColIdent<T> {}

    impl<Table, C> ColIdent<Table> for col_def_for_collection_member<C> where
        C: Member<Collection = Table>
    {
    }
}

pub struct AddColumn<Table, ColDef> {
    pub table: Table,
    pub col_def: ColDef,
}

pub struct DropColumn<Table, Name> {
    pub table: Table,
    pub col_name: Name,
}

impl<Table, Name, R0, R1> Inverse for AddColumn<Table, super::expressions::col_def<Name, R0, R1>> {
    type InverseStatement = DropColumn<Table, Name>;
    fn inverse(&self) -> Self::InverseStatement {
        todo!()
    }
}

impl<Table, Member> Inverse
    for AddColumn<Table, super::expressions::col_def_for_collection_member<Member>>
where
    Member: crate::collections::Member<Collection = Table>,
{
    type InverseStatement = DropColumn<Table, Member>;
    fn inverse(&self) -> Self::InverseStatement {
        todo!()
    }
}

impl<Table, ColDef> OpExpression for AddColumn<Table, ColDef> {}

mod impl_for_sqlx_fo {
    use crate::{Expression, statements::AddColumn};
    use sqlx::Sqlite;

    impl<'q, Table, ColDef> Expression<'q, Sqlite> for AddColumn<Table, ColDef>
    where
        Table: Expression<'q, Sqlite> + 'q,
        ColDef: Expression<'q, Sqlite> + 'q,
    {
        fn expression(self, ctx: &mut crate::QueryBuilder<'q, Sqlite>)
        where
            Sqlite: crate::DatabaseExt,
        {
            ctx.syntax("ALTER TABLE ");
            self.table.expression(ctx);
            ctx.syntax(" ADD COLUMN ");
            self.col_def.expression(ctx);
            ctx.syntax(";");
        }
    }
}
