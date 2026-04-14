use std::ops::Not;

use crate::{
    collections::Collection,
    database_extention::DatabaseExt,
    expressions::col_eq,
    extentions::Members,
    from_row::FromRowAlias,
    operations::{IsUniqueFilter, NeedCheck, SafeOperation},
    query_builder::{Expression, ManyExpressions, StatementBuilder, functional_expr::ManyFlat},
    statements::delete_statement::DeleteStatement,
    use_executor,
};
use sqlx::{ColumnIndex, Database, Decode, Encode, Executor, Row, Type};

use crate::{
    links::Link,
    operations::{LinkedOutput, Operation},
};

pub struct DeleteOne<Handler, Wheres, Links> {
    pub handler: Handler,
    pub wheres: Wheres,
    pub links: Links,
}

#[derive(Debug)]
pub enum DeleteOneError {
    NonUniqueFilters,
}

impl<H, W, L> SafeOperation for DeleteOne<H, W, L>
where
    W: IsUniqueFilter<H>,
    L: Link<H, Spec: IsUniqueFilter<H>>,
{
    type Error = DeleteOneError;

    type Ok = NeedCheck<DeleteOne<H, W, L::Spec>>;

    fn safety_check(self) -> Result<Self::Ok, Self::Error> {
        let links = self.links.spec(&self.handler);

        if (self.wheres.is_unique(&self.handler) || links.is_unique(&self.handler)).not() {
            return Err(DeleteOneError::NonUniqueFilters);
        }

        Ok(NeedCheck(DeleteOne {
            handler: self.handler,
            wheres: self.wheres,
            links,
        }))
    }
}

pub trait LinkDeleteOne<S> {
    type PreOp;
    fn pre_op<Wheres: ManyExpressions<'static, S>>(&self, wheres: Wheres) -> Self::PreOp;
    type DeleteReturnExpression;
    type DeleteReturnFromRow: for<'q> FromRowAlias<'q, S::Row, FromRowData = Self::DeleteReturn>
    where
        S: Database;
    type DeleteReturn;
    fn delete_return(
        &self,
        pre_op: <Self::PreOp as Operation<S>>::Output,
    ) -> (Self::DeleteReturnFromRow, Self::DeleteReturnExpression)
    where
        S: Database,
        Self::PreOp: Operation<S>;
}

impl<S: Database> LinkDeleteOne<S> for () {
    type PreOp = ();
    fn pre_op<W>(&self, _: W) -> Self::PreOp {}
    type DeleteReturnExpression = ();
    type DeleteReturnFromRow = ();
    type DeleteReturn = ();

    fn delete_return(
        &self,
        _: <Self::PreOp as Operation<S>>::Output,
    ) -> (Self::DeleteReturnFromRow, Self::DeleteReturnExpression)
    where
        S: Database,
        Self::PreOp: Operation<S>,
    {
        ((), ())
    }
}

impl<S, Handler, Wheres, Links> Operation<S> for NeedCheck<DeleteOne<Handler, Wheres, Links>>
where
    S: DatabaseExt,
    Links: Send,
    S: Database,
    Handler: Send,
    Handler: Collection<Data: Send>
        + for<'q> FromRowAlias<'q, S::Row, FromRowData = <Handler as Collection>::Data>,
    Handler: Members,
    Wheres: Clone + Send + ManyExpressions<'static, S>,
    Links: Send
        + LinkDeleteOne<
            S,
            PreOp: Operation<S, Output: Send> + Send,
            DeleteReturnExpression: ManyExpressions<'static, S>,
            DeleteReturnFromRow: Send + for<'r> FromRowAlias<'r, S::Row>,
            DeleteReturn: Send,
        >,
    for<'c> &'c mut <S as sqlx::Database>::Connection: Executor<'c, Database = S>,
    usize: ColumnIndex<S::Row>,
    i64: for<'q> Encode<'q, S> + for<'q> Decode<'q, S> + Type<S>,
{
    type Output = Option<LinkedOutput<i64, <Handler as Collection>::Data, Links::DeleteReturn>>;
    async fn exec_operation(self, pool: &mut S::Connection) -> Self::Output {
        let pre_op = self
            .0
            .links
            .pre_op(self.0.wheres.clone())
            .exec_operation(&mut *pool)
            .await;

        let (link_row, link_expr) = self.0.links.delete_return(pre_op);

        let qb = StatementBuilder::<'_, S>::new(DeleteStatement {
            table_name: self.0.handler.table_name().to_string(),
            wheres: self.0.wheres,
            returning: ManyFlat(("id", self.0.handler.members_names(), link_expr)),
        });

        let mut rest: Vec<_> = use_executor!(fetch_all(&mut *pool, qb))
            .expect("internal bug: claw_ql should have cleared all sqlx error at this point");

        if rest.len() == 0 {
            return None;
        }

        if rest.len() != 1 {
            panic!("make an update_one of multiple records!")
        }

        let row = rest.pop().expect("bug: should have only one row");

        let id: i64 = row.try_get(0).expect("bug: should have first as id");
        let attributes = self.0.handler.no_alias(&row).unwrap();
        let links = link_row.no_alias(&row).unwrap();

        Some(LinkedOutput {
            id,
            attributes,
            links,
        })
    }
}

#[cfg(test)]
mod test {
    use crate::operations::delete_one::DeleteOne;
    use crate::operations::{Operation, SafeOperation};
    use crate::test_module::*;
    use crate::{connect_in_memory::ConnectInMemory, expressions::col_eq};
    use sqlx::{Row, Sqlite};

    #[tokio::test]
    async fn main() {
        let mut pool = Sqlite::connect_in_memory_2().await;

        sqlx::query(
            "
            CREATE TABLE IF NOT EXISTS Todo (
                id INTEGER PRIMARY KEY AUTOINCREMENT,
                title TEXT NOT NULL,
                done BOOLEAN NOT NULL,
                description TEXT,
            );

            INSERT INTO Todo (id, title, done, description) VALUES 
                (1, 'first_todo', true, 'description_1'),
                (2, 'second_todo', false, NULL),
                (3, 'third_todo', true, 'description_3');

            ",
        )
        .execute(&mut pool)
        .await
        .unwrap();

        Operation::<Sqlite>::exec_operation(
            DeleteOne {
                handler: todo,
                wheres: col_eq {
                    col: todo_members::id,
                    eq: 3,
                },
                links: (),
            }
            .safety_check()
            .unwrap(),
            &mut pool,
        )
        .await
        .unwrap();

        let rows = sqlx::query("SELECT id, title FROM Todo; ")
            .fetch_all(&mut pool)
            .await
            .unwrap();

        if rows.len() != 2 {
            panic!("did not delete one row");
        };

        let first: String = rows[0].get("title");
        let third: String = rows[1].get("title");

        pretty_assertions::assert_eq!(first, "first_todo".to_string());
        pretty_assertions::assert_eq!(third, "third_todo".to_string());
    }
}
