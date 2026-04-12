use crate::{
    database_extention::DatabaseExt,
    query_builder::{Expression, ManyExpressions, OpExpression, QueryBuilder},
};

pub struct CreateTable<Init, TableName, ColDefs> {
    pub init: Init,
    pub name: TableName,
    pub col_defs: ColDefs,
}

pub mod expressions {
    #![allow(non_camel_case_types)]

    use crate::{
        database_extention::DatabaseExt,
        query_builder::{Expression, OpExpression, QueryBuilder},
    };

    pub struct create_table;

    impl OpExpression for create_table {}

    impl<'q, S> Expression<'q, S> for create_table {
        fn expression(self, ctx: &mut QueryBuilder<'q, S>)
        where
            S: DatabaseExt,
        {
            ctx.syntax(&"CREATE TABLE");
        }
    }

    pub struct create_if_not_exist;

    impl OpExpression for create_if_not_exist {}

    impl<'q, S> Expression<'q, S> for create_if_not_exist {
        fn expression(self, ctx: &mut QueryBuilder<'q, S>)
        where
            S: DatabaseExt,
        {
            ctx.syntax(&"CREATE TABLE IF NOT EXISTS");
        }
    }
}

impl<Header, Table, Columns> OpExpression for CreateTable<Header, Table, Columns> {}

impl<'q, S, Header, Table, Columns> Expression<'q, S> for CreateTable<Header, Table, Columns>
where
    Header: Expression<'q, S> + 'q,
    Table: Expression<'q, S> + 'q,
    Columns: ManyExpressions<'q, S> + 'q,
{
    fn expression(self, ctx: &mut QueryBuilder<'q, S>)
    where
        S: DatabaseExt,
    {
        let open_b = "(";
        let close_b = ");";
        self.init.expression(ctx);
        ctx.syntax(&" ");
        self.name.expression(ctx);
        // ctx.sanitize(self.name);
        ctx.syntax(&" ");
        ctx.syntax(&open_b);
        self.col_defs.expression(&"", &", ", ctx);
        ctx.syntax(&close_b);
    }
}

mod impl_syntax_for_create_table {
    use crate::{
        collections::{Collection, Member, SingleIncremintalInt},
        expressions::col_def_for_collection_member,
        statements::create_table_statement::{
            CreateTable,
            expressions::{create_if_not_exist, create_table},
        },
        valid_syntax::{ColIdent, CreateTableHeader, TableIdent, ValidSyntax},
    };

    impl<Header, Table, Columns> ValidSyntax for CreateTable<Header, Table, Columns>
    where
        Header: CreateTableHeader,
        Table: TableIdent,
        Columns: ColIdent<Table>,
    {
    }

    impl CreateTableHeader for create_if_not_exist {}
    impl CreateTableHeader for create_table {}

    impl<T> ColIdent<T> for SingleIncremintalInt where T: Collection<Id = Self> {}

    impl<T, C0> ColIdent<T> for (C0,) where C0: ColIdent<T> {}

    impl<Table, C> ColIdent<Table> for col_def_for_collection_member<C> where
        C: Member<CollectionHandler = Table>
    {
    }
}

#[cfg(feature = "skip_without_comments")]
mod old_dynamic_statement {
    #![allow(unused)]
    use std::{marker::PhantomData, ops::Not};

    use crate::{
        Buildable, ColumPositionConstraint, Expression, ExpressionToFragment, QueryBuilder,
    };

    #[derive(Debug)]
    pub struct CreateTableSt<S: QueryBuilder> {
        pub(crate) header: String,
        pub(crate) ident: (Option<String>, String),
        pub(crate) columns: Vec<(String, S::Fragment)>,
        pub(crate) constraints: Vec<S::Fragment>,
        pub(crate) verbatim: Vec<String>,
        pub(crate) ctx: S,
        pub(crate) _sqlx: PhantomData<S>,
    }

    #[allow(non_upper_case_globals)]
    pub mod header {
        pub const create: &'static str = "CREATE TABLE";
        pub const create_temp: &'static str = "CREATE TEMP";
        pub const create_temp_if_not_exists: &'static str = "CREATE TEMP IF NOT EXISTS";
        pub const create_table_if_not_exists: &'static str = "CREATE TABLE IF NOT EXISTS";
    }

    impl<S: QueryBuilder> Buildable for CreateTableSt<S> {
        type QueryBuilder = S;

        fn build(self) -> (String, S::Output) {
            S::to_output(self.ctx, |ctx| {
                let mut str = String::from(&self.header);
                str.push(' ');

                if let Some(schema) = self.ident.0 {
                    str.push_str(&schema);
                }

                str.push_str(self.ident.1.as_ref());

                str.push_str(" (");

                let mut clauses = Vec::new();
                for (mut col, constrain) in self.columns {
                    let constrain = S::fragment_to_string(ctx, constrain);
                    // constrain can be () which build back to ""
                    if constrain.is_empty().not() {
                        col.push_str(&format!(" {}", constrain))
                    }
                    clauses.push(col);
                }
                for constraint in self.constraints {
                    let item = S::fragment_to_string(ctx, constraint);
                    clauses.push(item);
                }

                for verbatim in self.verbatim {
                    clauses.push(verbatim);
                }
                if clauses.is_empty() {
                    panic!("columns is empty");
                }
                str.push_str(&clauses.join(", "));
                str.push_str(");");
                str
            })
        }
    }

    impl<Q: QueryBuilder + Default> CreateTableSt<Q> {
        pub fn init(header: &str, table: &str) -> Self {
            Self {
                header: header.to_string(),
                ident: (None, table.to_string()),
                columns: Default::default(),
                constraints: Default::default(),
                verbatim: Default::default(),
                ctx: Default::default(),
                _sqlx: PhantomData,
            }
        }
        pub fn column_def<C>(&mut self, name: &str, constraint: C)
        where
            Q: ExpressionToFragment<'static, C> + ColumPositionConstraint,
        {
            let item = Q::expression_to_fragment(&mut self.ctx, constraint);
            self.columns.push((name.to_string(), item));
        }
    }
}
