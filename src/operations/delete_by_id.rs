use std::ops::Not;

use crate::{
    collections::Collection,
    database_extention::DatabaseExt,
    expressions::col_eq,
    extentions::Members,
    from_row::FromRowAlias,
    operations::{IsUniqueFilter, NeedCheck, SafeOperation},
    query_builder::{ManyExpressions, StatementBuilder, functional_expr::ManyFlat},
    statements::delete_statement::DeleteStatement,
    use_executor,
};
use sqlx::{ColumnIndex, Database, Decode, Encode, Executor, Row, Type};

use crate::{
    links::Link,
    operations::{LinkedOutput, Operation},
};

pub struct DeleteById<Handler, Links> {
    pub handler: Handler,
    pub id: i64,
    pub links: Links,
}

pub trait LinkDeleteById<S> {
    type PreOp;
    fn pre_op(&self, id: i64) -> Self::PreOp;
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

impl<S: Database> LinkDeleteById<S> for () {
    type PreOp = ();
    fn pre_op(&self, _: i64) -> Self::PreOp {}
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
impl<S, Handler, Links> Operation<S> for DeleteById<Handler, Links>
where
    S: DatabaseExt,
    Links: Send,
    S: Database,
    Handler: Send,
    Handler: Collection<Data: Send>
        + for<'q> FromRowAlias<'q, S::Row, FromRowData = <Handler as Collection>::Data>,
    Handler: Members,
    Links: Link<Handler>,
    Links::Spec: Send
        + LinkDeleteById<
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
    type Output = Option<
        LinkedOutput<
            i64,
            <Handler as Collection>::Data,
            <Links::Spec as LinkDeleteById<S>>::DeleteReturn,
        >,
    >;
    async fn exec_operation(self, pool: &mut S::Connection) -> Self::Output {
        let links = self.links.spec(&self.handler);
        let pre_op = links.pre_op(self.id).exec_operation(&mut *pool).await;
        let (link_row, link_expr) = links.delete_return(pre_op);

        let qb = StatementBuilder::<'_, S>::new(DeleteStatement {
            table_name: self.handler.table_name().to_string(),
            wheres: col_eq {
                col: "id",
                eq: self.id,
            },
            returning: ManyFlat(("id", self.handler.members_names(), link_expr)),
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
        let attributes = self.handler.no_alias(&row).unwrap();
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
    use crate::operations::Operation;
    use crate::test_module::*;
    use crate::{connect_in_memory::ConnectInMemory, operations::delete_by_id::DeleteById};
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
            DeleteById {
                handler: todo,
                id: 2,
                links: (),
            },
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
