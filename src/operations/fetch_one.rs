#![allow(unused)]
use crate::collections::{Collection, Id, SingleIncremintalInt};
use crate::database_extention::DatabaseExt;
use crate::execute::Executable;
use crate::expressions::{col, scoped_column, table};
use crate::extentions::Members;
use crate::from_row::{FromRowAlias, pre_alias};
use crate::links::{self, Link};
use crate::operations::{LinkedOutput, Operation};
use crate::query_builder::functional_expr::ZeroOrMoreImplPossible;
use crate::query_builder::{Expression, QueryBuilder, ZeroOrMoreExpressions};
use crate::statements::select_statement::SelectStatement;
use crate::use_executor;
use axum::serve::Listener;
use sqlx::{ColumnIndex, Database, Decode, Pool, Type};
use sqlx::{Executor, Row};

pub struct FetchOne<From, Links, Wheres> {
    pub base: From,
    // extendable
    pub wheres: Wheres,
    // extendable and generate data
    pub links: Links,
}

impl<S: Database, Base, Links, W> Operation<S> for FetchOne<Base, Links, W>
where
    S: DatabaseExt,
    Base: Collection<Data: Send, Id = SingleIncremintalInt> + 'static,
    <Base::Id as Id>::Data: for<'q> Decode<'q, S> + Type<S>,
    for<'q> &'q str: ColumnIndex<S::Row>,
    Base: Members<S>,
    W: ZeroOrMoreExpressions<'static, S>,
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

        let link_spec = self.links.spec(&self.base);
        let link_extend_stmt = link_spec.extend_select();

        let main_statement = SelectStatement {
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
                    expressions: link_extend_stmt
                        .non_aggregating_select_items
                        .into_iter()
                        .map(|e| e.pre_alias("link_"))
                        .collect::<Vec<_>>(),
                    start: "",
                    join: ", ",
                },
            ),
            from: table(self.base.table_name().to_string()),
            joins: link_extend_stmt.non_duplicating_joins,
            wheres: (
                ZeroOrMoreImplPossible {
                    expressions: link_extend_stmt.wheres,
                    start: "",
                    join: " AND ",
                },
                ZeroOrMoreImplPossible {
                    expressions: self.wheres,
                    start: "",
                    join: " AND ",
                },
            ),
            order: (),
            limit: (),
        };

        Expression::expression(main_statement, &mut query_builder);

        let s = use_executor!(fetch_optional(&pool, query_builder));

        let s = match s {
            Err(sqlx::Error::RowNotFound) | Ok(None) => return None,
            Err(err) => {
                panic!(
                    "bug: claw_ql must clear all sqlx's error, 
it is hard to know where this error was originated!
error: {:?}",
                    err
                )
            }
            Ok(Some(ok)) => ok,
        };

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
