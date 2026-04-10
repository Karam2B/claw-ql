#![allow(non_camel_case_types)]
#![allow(unexpected_cfgs)]
pub mod expressions;
pub mod functional_expr;
pub mod sanitize;
pub mod statements;
pub mod syntax;

use crate::{
    DatabaseExt,
    functional_expr::{BoxedExpression, StaticExpression},
};
use sqlx::{Encode, Type};

pub struct QueryBuilder<'q, S>
where
    S: DatabaseExt,
{
    stmt: String,
    count: usize,
    arg: S::Arguments<'q>,
}

impl<'q, S: DatabaseExt> Default for QueryBuilder<'q, S> {
    fn default() -> Self {
        QueryBuilder {
            stmt: String::new(),
            count: 0,
            arg: S::Arguments::default(),
        }
    }
}

pub trait SqlSanitize<S> {
    fn to_sql(&self) -> &str;
    fn safe_to_sql(&self) -> bool {
        false
    }
}

pub trait SqlSyntax<S> {
    fn to_sql(self, str: &mut String);
}

/// assert is not noop
/// prevent downstream crate from `impl Expression<LocalType> for ForiegnTypes`
pub trait OpExpression {}

pub trait Expression<'q, S>: OpExpression + 'q {
    fn expression(self, ctx: &mut QueryBuilder<'q, S>)
    where
        S: DatabaseExt;
}

pub trait OneOrMoreExpressions<'q, S>: OpExpression + 'q {
    fn to_expr(self) -> Vec<Box<dyn BoxedExpression<'q, S> + 'q>>
    where
        Self: Sized,
    {
        todo!()
    }
    fn expression(self, start: &'static str, join: &'static str, ctx: &mut QueryBuilder<'q, S>)
    where
        S: DatabaseExt;
}

pub trait IsOpExpression {
    fn is_op(&self) -> bool;
}

pub trait PossibleExpression<'q, S>: IsOpExpression + 'q {
    fn expression_starting(self, start: &'static str, ctx: &mut QueryBuilder<'q, S>)
    where
        S: DatabaseExt;
    fn expression(self, ctx: &mut QueryBuilder<'q, S>)
    where
        S: DatabaseExt;
}

pub trait ToStaticExpressions<S> {
    fn to_static_expr(self) -> Vec<Box<dyn StaticExpression<S> + Send>>
    where
        Self: Sized;
}

pub trait ZeroOrMoreExpressions<'q, S>: IsOpExpression + 'q {
    fn to_expr(self) -> Vec<Box<dyn BoxedExpression<'q, S> + 'q>>
    where
        Self: Sized;
    fn expression(self, start: &'static str, join: &'static str, ctx: &mut QueryBuilder<'q, S>)
    where
        S: DatabaseExt;
}

pub fn run_expression<'q, S, E>(expr: E) -> (String, <S as sqlx::Database>::Arguments<'q>)
where
    S: DatabaseExt,
    E: Expression<'q, S>,
{
    let mut b = QueryBuilder::<'q, S>::default();
    expr.expression(&mut b);
    (b.stmt, b.arg)
}

impl<'q, S> QueryBuilder<'q, S>
where
    S: DatabaseExt,
{
    pub fn bind<V>(&mut self, value: V)
    where
        V: Encode<'q, S> + 'q + Type<S>,
    {
        use sqlx::Arguments;
        self.arg.add(value).expect("when does this ever fail?");
        self.count += 1;
        self.stmt.push_str(format!("${}", self.count).as_str());
    }

    pub fn sanitize<D: SqlSanitize<S>>(&mut self, display: D) {
        if S::NAME != "SQLite" {
            panic!(
                "I only know how to sanitize for sqlite syntax for now! todo: specify behaviour in DatabaseExt trait"
            )
        }
        self.stmt.push('\'');

        let s = display.to_sql();
        let safe = display.safe_to_sql();

        if safe {
            self.stmt.push_str(s);
        } else {
            let mut s = s.chars();
            while let Some(next) = s.next() {
                match next {
                    '\'' => {
                        self.stmt.push(next);
                        self.stmt.push('\'');
                    }
                    '\\' => {
                        self.stmt.push(next);
                        self.stmt.push('\\');
                    }
                    n => self.stmt.push(n),
                }
            }
        }

        self.stmt.push('\'');
    }

    /// push str that is known to not cause sql injection,
    pub fn syntax<D: SqlSyntax<S>>(&mut self, str: D) {
        str.to_sql(&mut self.stmt);
    }

    pub fn unwrap(self) -> (String, S::Arguments<'q>) {
        (self.stmt, self.arg)
    }
}

pub mod syntax_trait {
    pub trait ValidSyntax<S> {
        fn runtime_check(&self) -> bool {
            true
        }
    }

    pub trait CreateTableHeader<S> {
        fn runtime_check(&self) -> bool {
            true
        }
    }

    pub trait TableIdent {}
    pub trait ColIdent<Table> {}
}

pub use syntax_trait::ValidSyntax;

#[allow(unused)]
mod mut_query_builder {
    use std::hash::BuildHasher;

    use crate::{
        OpExpression,
        expressions::{col, col_eq},
    };

    pub trait MutBuilder<'q> {
        type Bound: 'static;
        type Ctx1: BuilderCtx1<'q, Builder = Self>;
        type Ctx2: BuilderCtx2<'q, Builder = Self>;
    }

    pub trait BuilderCtx1<'q> {
        type Builder: MutBuilder<'q, Ctx1 = Self>;
        fn bind<V>(&mut self, value: V) -> <Self::Builder as MutBuilder<'q>>::Bound;
    }
    pub trait BuilderCtx2<'q> {
        type Builder: MutBuilder<'q, Ctx2 = Self>;
        fn sanitize<S>(&mut self, san: S);
        fn bind(&mut self, san: <Self::Builder as MutBuilder<'q>>::Bound);
        fn syntax<S>(&mut self, san: S);
    }

    pub trait MutExpression<'q, S: MutBuilder<'q>>: 'q {
        type Intermid: 'static;
        fn stage_1(self, ctx: &mut S::Ctx1) -> Self::Intermid;
        fn stage_2(this: Self::Intermid, ctx: &mut S::Ctx2);
    }

    impl<'q, S, Col, Eq> MutExpression<'q, S> for col_eq<Col, Eq>
    where
        Col: 'static,
        Eq: 'q,
        S: MutBuilder<'q>,
    {
        type Intermid = col_eq<Col, S::Bound>;
        fn stage_1(self, ctx: &mut <S as MutBuilder<'q>>::Ctx1) -> Self::Intermid {
            let eq = ctx.bind(self.eq);
            col_eq { col: self.col, eq }
        }
        fn stage_2(this: Self::Intermid, ctx: &mut <S as MutBuilder<'q>>::Ctx2) {
            ctx.sanitize(this.col);
            ctx.syntax(" = ");
            ctx.bind(this.eq);
        }
    }
}
