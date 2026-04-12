#![allow(unused)]
use crate::collections::{Collection, Id, SingleIncremintalInt};
use crate::database_extention::DatabaseExt;
use crate::execute::Executable;
use crate::expressions::{col, scoped_column, table};
use crate::extentions::Members;
use crate::from_row::{FromRowAlias, pre_alias};
use crate::links::{self, Link};
use crate::operations::{LinkedOutput, Operation};
use crate::query_builder::functional_expr::{ManyFlat, ManyImplPossible};
use crate::query_builder::{Expression, ManyExpressions, QueryBuilder};
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

impl<S: Database, Base, Links, Wheres> Operation<S> for FetchOne<Base, Links, Wheres>
where
    S: DatabaseExt,
    Base: Collection<Data: Send, Id = SingleIncremintalInt> + 'static,
    <Base::Id as Id>::Data: for<'q> Decode<'q, S> + Type<S>,
    for<'q> &'q str: ColumnIndex<S::Row>,
    Base: Members<S>,
    Wheres: ManyExpressions<'static, S>,
    Base: for<'r> FromRowAlias<'r, S::Row, FromRowData = Base::Data>,
    // fetch_optional
    for<'c> &'c mut <S as sqlx::Database>::Connection: Executor<'c, Database = S>,
    Links: Link<Base>,
    Links::Spec: Send
        + LinkFetchOne<
            S,
            Output: Send,
            Inner: Send,
            Joins: Send + ManyExpressions<'static, S>,
            Wheres: Send + ManyExpressions<'static, S>,
        >,
    Wheres: Send,
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

    async fn exec_operation(self, pool: Pool<S>) -> Self::Output
    where
        S: Database,
    {
        // let mut query_builder = QueryBuilder::<'_, S>::default();

        let link_spec = self.links.spec(&self.base);
        let link_extend_stmt = link_spec.non_aggregating_select_items();
        let link_extend_joins = link_spec.non_duplicating_joins();
        let link_extend_wheres = link_spec.wheres();

        let query_builder = QueryBuilder::<'_, S>::new(SelectStatement {
            select_items: ManyFlat((
                table(self.base.table_name().to_string())
                    .col("id")
                    .pre_alias("local_"),
                self.base
                    .members_names()
                    .into_iter()
                    .map(|e| {
                        table(self.base.table_name().to_string())
                            .col(e)
                            .pre_alias("base_")
                    })
                    .collect::<Vec<_>>(),
                link_extend_stmt
                    .into_iter()
                    .map(|e| e.pre_alias("link_"))
                    .collect::<Vec<_>>(),
            )),
            from: self.base.table_name().to_string(),
            joins: link_extend_joins,
            wheres: ManyFlat((link_extend_wheres, self.wheres)),
            order: (),
            limit: (),
        });
        /*

        let main_statement = SelectStatement {
            select_items: ManyFlat::<((), ())>(
                (table(self.base.table_name().to_string())
                    .col("id")
                    .pre_alias("local_"),
            ),
            /*

            ,
            ManyImplPossible {
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
            ManyImplPossible {
                expressions: link_extend_stmt
                    .into_iter()
                    .map(|e| e.pre_alias("link_"))
                    .collect::<Vec<_>>(),
                start: "",
                join: ", ",
            },
             */
            from: table(self.base.table_name().to_string()),
            joins: link_extend_joins,
            wheres: ManyFlat((link_extend_wheres, self.wheres)),
            order: (),
            limit: (),
        };

        Expression::expression(main_statement, &mut query_builder);
         */

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

        let sub_op = sub_op.exec_operation(pool.clone()).await;

        Some(LinkedOutput {
            id: Row::get(&s, "local_id"),
            attributes: self
                .base
                .pre_alias(pre_alias(&s, "base_"))
                .expect("bug: sqlx errors should ruled out by claw_ql"),
            links: link_spec.take(sub_op, inner),
        })
    }
}

pub trait LinkFetchOne<S> {
    type Joins;
    type Wheres;

    fn non_aggregating_select_items(&self) -> Vec<scoped_column<String, String>>;
    /// joins has to be non duplicating in order to be extendable
    /// otherwise I have to rewrite the code that uses this struct
    ///
    /// example of duplicating joins is optional_to_many RIGHT JOIN
    fn non_duplicating_joins(&self) -> Self::Joins;
    fn wheres(&self) -> Self::Wheres;

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

#[cfg(test)]
mod test {
    use sqlx::{Executor, Sqlite};

    use crate::{
        connect_in_memory::ConnectInMemory,
        execute::Executable,
        expressions::{col_eq, scoped_column},
        operations::Operation,
        operations::{CollectionOutput, LinkedOutput},
        prelude::sql::FetchOne,
        test_module::*,
    };

    #[tokio::test]
    async fn test_fetch_one() {
        let pool = Sqlite::connect_in_memory().await;

        Executor::execute(
            &pool,
            Executable {
                string: "
                CREATE TABLE IF NOT EXISTS Category (
                    id INTEGER PRIMARY KEY AUTOINCREMENT,
                    title TEXT NOT NULL
                );

                CREATE TABLE IF NOT EXISTS Todo (
                    id INTEGER PRIMARY KEY AUTOINCREMENT,
                    title TEXT NOT NULL,
                    done BOOLEAN NOT NULL,
                    description TEXT,
                    category_id INTEGER,
                    FOREIGN KEY (category_id) REFERENCES Category(id)
                );
                
                INSERT INTO Category (id, title)
                    VALUES
                        (1, 'category_1');
                        
                INSERT INTO Todo 
                    (id, title, done, description, category_id)
                    VALUES
                        (1, 'first_todo', true, 'description_1', NULL), 
                        (2, 'second_todo', false, NULL, 1);
                        ",
                arguments: Default::default(),
            },
        )
        .await
        .unwrap();

        let fetch_one = FetchOne {
            base: todo,
            wheres: col_eq {
                col: scoped_column {
                    table: "Todo".to_string(),
                    column: "title".to_string(),
                },
                eq: "second_todo",
            },
            links: category,
        }
        .exec_operation(pool)
        .await;

        pretty_assertions::assert_eq!(
            fetch_one,
            Some(LinkedOutput {
                id: 2,
                attributes: Todo {
                    title: "second_todo".to_string(),
                    done: false,
                    description: None
                },
                links: Some(CollectionOutput {
                    id: 1,
                    attributes: Category {
                        title: "category_1".to_string()
                    }
                }),
            })
        );
    }
}
