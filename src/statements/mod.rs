use crate::query_builder::OpExpression;

// waiting for second rewrite !!
pub mod create_table_statement;
// pub mod delete_st;
// pub mod insert_one_st;
pub mod insert_one_statement;
pub mod select_statement;
pub mod update_statement;
// pub mod update_st;

pub trait Inverse {
    type InverseStatement;
    fn inverse(&self) -> Self::InverseStatement;
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
    Member: crate::collections::Member<CollectionHandler = Table>,
{
    type InverseStatement = DropColumn<Table, Member>;
    fn inverse(&self) -> Self::InverseStatement {
        todo!()
    }
}

impl<Table, ColDef> OpExpression for AddColumn<Table, ColDef> {}

mod impl_for_sqlx_fo {
    use crate::{
        database_extention::DatabaseExt,
        query_builder::{Expression, QueryBuilder, syntax::end_of_statement},
        sql_syntax,
        statements::AddColumn,
    };
    use sqlx::Sqlite;

    sql_syntax!(alter_table = "ALTER TABLE ");
    sql_syntax!(add_column = " ADD COLUMN ");

    impl<'q, Table, ColDef> Expression<'q, Sqlite> for AddColumn<Table, ColDef>
    where
        Table: Expression<'q, Sqlite> + 'q,
        ColDef: Expression<'q, Sqlite> + 'q,
    {
        fn expression(self, ctx: &mut QueryBuilder<'q, Sqlite>)
        where
            Sqlite: DatabaseExt,
        {
            ctx.syntax(&alter_table);
            self.table.expression(ctx);
            ctx.syntax(&add_column);
            self.col_def.expression(ctx);
            ctx.syntax(&end_of_statement);
        }
    }
}
