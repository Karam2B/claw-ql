#![allow(unused)]
#![warn(unused_must_use)]
#![allow(non_camel_case_types)]

use axum::response;
use claw_ql::{
    expressions::col_eq,
    query_builder::{Expression, ManyExpressions, OpExpression, PossibleExpression, StatementBuilder},
};
use sqlx::{Executor, Sqlite};
use std::{marker::PhantomData, ops::Not};

pub struct Statement<Base, Wheres, Order> {
    pub base: Base,
    pub wheres: Wheres,
    pub order: Order,
}

impl<B, W, O> OpExpression for Statement<B, W, O> {}

// #[auto_dynamic_stmt]
impl<Base, Where, Order> Expression<'static, Sqlite> for Statement<Base, Where, Order>
where
    Base: Expression<'static, Sqlite>,          // -> String,
    Where: ManyExpressions<'static, Sqlite>,    // -> Vec<String>
    Order: PossibleExpression<'static, Sqlite>, // -> Option<String>
{
    fn expression(self, ctx: &mut StatementBuilder<'static, Sqlite>) {
        ctx.syntax(&"FROM");
        self.base.expression(ctx);
        self.wheres.expression(&"WHERE", &"AND", ctx);
        if self.order.is_op() {
            ctx.syntax(&"ORDER");
            self.order.expression(ctx);
        }
    }
}

#[derive(Default)]
pub struct SqliteDynamicBuilder {
    pub count: usize,
}

impl SqliteDynamicBuilder {
    pub fn expr_to_fragment<E>(&mut self, expr: E) -> String {
        self.count += 1;
        format!("${}", self.count)
    }
}

pub struct DynamicStatement {
    pub(crate) stmt: Statement<String, Vec<String>, Option<String>>,
    ctx: SqliteDynamicBuilder,
}

pub struct DynamicStatmentInput {
    pub base: String,
}

impl From<DynamicStatmentInput> for DynamicStatement {
    fn from(input: DynamicStatmentInput) -> Self {
        Self {
            stmt: Statement {
                base: input.base,
                wheres: vec![],
                order: None,
            },
            ctx: SqliteDynamicBuilder::default(),
        }
    }
}

impl From<String> for DynamicStatement {
    fn from(base: String) -> Self {
        Self {
            stmt: Statement {
                base: base,
                wheres: vec![],
                order: None,
            },
            ctx: SqliteDynamicBuilder::default(),
        }
    }
}

impl<Base, Where, Order> From<Statement<Base, Where, Order>> for DynamicStatement {
    fn from(stmt: Statement<Base, Where, Order>) -> Self {
        todo!()
    }
}

impl DynamicStatement {
    pub fn desc_order(&mut self, col: &str) {
        self.stmt.order = Some(format!("DESC {}", col));
    }
    pub fn where_<W>(&mut self, where_item: W) {
        let where_item = self.ctx.expr_to_fragment(where_item);
        self.stmt.wheres.push(where_item);
    }
}

impl OpExpression for DynamicStatement {}
impl Expression<'static, Sqlite> for DynamicStatement {
    fn expression(self, ctx: &mut StatementBuilder<'static, Sqlite>) {
        todo!()
    }
}

fn select() {
    let mut st = DynamicStatement {
        stmt: Statement {
            base: String::from("Todo"),
            wheres: vec![],
            order: None,
        },
        ctx: SqliteDynamicBuilder { count: 0 },
    };

    st.desc_order("title");
    st.where_(col_eq {
        col: "description".to_string(),
        eq: 5,
    });

    let sql = StatementBuilder::new(st);

    pretty_assertions::assert_eq!(
        sql.stmt(),
        "FROM 'Todo' WHERE 'description' = $1 ORDER BY DESC 'title';",
    );
}
