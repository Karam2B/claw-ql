#![allow(unused)]
use crate::collections::{Id, SingleIncremintalInt};
use crate::execute::Executable;
use crate::expressions::{col, scoped_column, table};
use crate::extentions::Members;
use crate::from_row::{FromRowAlias, pre_alias};
use crate::functional_expr::ZeroOrMoreImplPossible;
use crate::links::{self, Link};
use crate::operations::{LinkedOutput, Operation};
use crate::{DatabaseExt, Expression, query_builder::QueryBuilder};
use crate::{PossibleExpression, ZeroOrMoreExpressions, use_executor};
use crate::{
    collections::Collection, functional_expr::BoxedExpression, functional_expr::boxed_expr,
    statements::SelectStatement,
};
use axum::serve::Listener;
use sqlx::{ColumnIndex, Database, Decode, Pool, Type};
use sqlx::{Executor, Row};

pub struct FetchOne<From, Links, Wheres> {
    pub base: From,
    // extendable
    pub wheres: Wheres,
    // extendable and generate data
    pub link: Links,
}

impl<S: Database, Base, Links, W> Operation<S> for FetchOne<Base, Links, W>
where
    S: DatabaseExt,
    Base: Collection<Data: Send, Id = SingleIncremintalInt> + 'static,
    <Base::Id as Id>::Data: for<'q> Decode<'q, S> + Type<S>,
    for<'q> &'q str: ColumnIndex<S::Row>,
    Base: Members<S>,
    Base: for<'r> FromRowAlias<'r, S::Row>,
    // fetch_optional
    for<'c> &'c mut <S as sqlx::Database>::Connection: Executor<'c, Database = S>,
    Links: Link<Base>,
    Links::Spec: Send
        + LinkFetchOne<
            S,
            Output: Send,
            Inner: Send,
            Joins: Send + ZeroOrMoreExpressions<'static, S>,
            Wheres: Send + ZeroOrMoreExpressions<'static, S>,
        >,
    W: Send,
    Base: Send,
    Links: Send,
{
    type Output = Option<
        LinkedOutput<
            <Base::Id as Id>::Data,
            Base::Data,
            <<Links as Link<Base>>::Spec as LinkFetchOne<S>>::Output,
        >,
    >;

    async fn exec(self, pool: Pool<S>) -> Self::Output
    where
        S: Database,
    {
        let mut query_builder = QueryBuilder::<'_, S>::default();

        let link_spec = self.link.spec(&self.base);
        let extend_stmt = link_spec.extend_select();

        let ss = SelectStatement {
            select_items: (
                table(self.base.table_name().to_string())
                    .col("id")
                    .pre_alias("local_"),
                ZeroOrMoreImplPossible {
                    expressions: self
                        .base
                        .members_names()
                        .into_iter()
                        .map(|e| {
                            table(self.base.table_name().to_string())
                                .col(e)
                                .pre_alias("base_")
                        })
                        .collect::<Vec<_>>(),
                    start: "",
                    join: ", ",
                },
                ZeroOrMoreImplPossible {
                    expressions: extend_stmt
                        .non_aggregating_select_items
                        .into_iter()
                        .map(|e| e.pre_alias("link_"))
                        .collect::<Vec<_>>(),
                    start: "",
                    join: ", ",
                },
            ),
            from: table(self.base.table_name().to_string()),
            joins: extend_stmt.non_duplicating_joins,
            wheres: extend_stmt.wheres,
            order: (),
            limit: (),
        };

        Expression::expression(ss, &mut query_builder);
        // let (sql, arg) = query_builder.unwrap();

        let s = use_executor!(fetch_optional(&pool, query_builder))
            .expect("bug: claw_ql must clear all sqlx's error, but I really don't know where this error has originated!")?;
        // let s = Executor::fetch_optional(
        // &pool,
        // Executable {
        //     string: &sql,
        //     arguments: arg,
        //     db: std::marker::PhantomData,
        // },
        //     )
        //     .await
        //     .expect("bug: claw_ql must clear all sqlx's error, but I really don't know where this error has originated!")?;

        let (sub_op, inner) = link_spec.sub_op(pre_alias(&s, "link_"));

        let sub_op = sub_op.exec(pool.clone()).await;

        Some(LinkedOutput {
            id: Row::get(&s, "local_id"),
            attributes: self
                .base
                .pre_alias(pre_alias(&s, "base_"))
                .expect("bug: sqlx errors should ruled out by claw_ql"),
            link: link_spec.take(sub_op, inner),
        })
    }
}

pub trait LinkFetchOne<S> {
    type Joins;
    type Wheres;
    fn extend_select(
        &self,
    ) -> SelectStatementExtendableParts<
        //
        Vec<scoped_column<String, String>>,
        Self::Joins,
        Self::Wheres,
    >;

    type Inner;
    type SubOp: Operation<S>;
    fn sub_op(&self, row: pre_alias<'_, <S as Database>::Row>) -> (Self::SubOp, Self::Inner)
    where
        S: Database;

    type Output;
    fn take(
        self,
        extend: <Self::SubOp as Operation<S>>::Output,
        inner: Self::Inner,
    ) -> Self::Output;
}

pub struct SelectStatementExtendableParts<S, J, W> {
    pub non_aggregating_select_items: S,
    /// joins has to be non duplicating in order to be extendable
    /// otherwise I have to rewrite the code that uses this struct
    ///
    /// example of duplicating joins is optional_to_many RIGHT JOIN
    pub non_duplicating_joins: J,
    pub wheres: W,
}
