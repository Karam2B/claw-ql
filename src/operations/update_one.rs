use std::ops::Not;

use crate::{
    collections::{Collection, SingleIncremintalInt},
    database_extention::DatabaseExt,
    expressions::table,
    extentions::Members,
    extentions::common_expressions::OnUpdate,
    from_row::FromRowAlias,
    operations::{IsUniqueFilter, NeedCheck, SafeOperation},
    query_builder::{IsOpExpression, ManyExpressions, StatementBuilder, functional_expr::ManyFlat},
    sqlx_error_handling::HandleSqlxResult,
    statements::update_statement::UpdateStatement,
    use_executor,
};
use sqlx::{ColumnIndex, Database, Decode, Executor, Row, Type};

use crate::{
    links::Link,
    operations::{LinkedOutput, Operation},
};

pub struct UpdateOne<Handler, Partial, Wheres, Links> {
    pub handler: Handler,
    pub partial: Partial,
    pub wheres: Wheres,
    pub links: Links,
}

pub trait LinkUpdateOne<S> {
    type PreOp;
    fn pre_op(&self) -> Self::PreOp;
    type UpdateSets;
    type UpdateWheres;
    type UpdateReturning;
    fn update_extention(
        &self,
        pre_op: <Self::PreOp as Operation<S>>::Output,
    ) -> (Self::UpdateSets, Self::UpdateWheres, Self::UpdateReturning)
    where
        Self::PreOp: Operation<S>;
    type PostOp;
    fn post_op(&self, id: i64, row: &S::Row) -> Self::PostOp
    where
        S: Database;
    type Output;
    fn output(&self, post_op: <Self::PostOp as Operation<S>>::Output) -> Self::Output
    where
        Self::PostOp: Operation<S>;
}

impl<S> LinkUpdateOne<S> for () {
    type PreOp = ();
    type Output = ();
    type PostOp = ();
    type UpdateSets = ();
    type UpdateWheres = ();
    type UpdateReturning = ();
    fn output(&self, _: <Self::PostOp as Operation<S>>::Output) -> Self::Output
    where
        Self::PostOp: Operation<S>,
    {
    }

    fn pre_op(&self) -> Self::PreOp {}

    fn update_extention(
        &self,
        _: <Self::PreOp as Operation<S>>::Output,
    ) -> (Self::UpdateSets, Self::UpdateWheres, Self::UpdateReturning)
    where
        Self::PreOp: Operation<S>,
    {
        ((), (), ())
    }

    fn post_op(&self, _: i64, _: &S::Row) -> Self::PostOp
    where
        S: Database,
    {
    }
}

#[derive(Debug)]
pub enum UpdateOneError<T> {
    ValidationError(T),
    NonUniqueOperation,
    NonOperational,
}

impl<Handler, Wheres, Links> SafeOperation
    for UpdateOne<Handler, Handler::UpdateInput, Wheres, Links>
where
    Handler: Collection,
    Handler: OnUpdate,
    Links: Link<Handler, Spec: IsUniqueFilter<Handler> + IsOpExpression>,
    Wheres: IsUniqueFilter<Handler>,
    Handler::UpdateExpression: IsOpExpression,
{
    type Error = UpdateOneError<Handler::UpdateError>;
    type Ok = NeedCheck<UpdateOne<Handler, Handler::UpdateExpression, Wheres, Links::Spec>>;
    fn safety_check(self) -> Result<Self::Ok, Self::Error> {
        let partial = match self.handler.validate_on_update(self.partial) {
            Ok(partial) => partial,
            Err(e) => return Err(UpdateOneError::ValidationError(e)),
        };

        // if self.wheres or links::wheres do
        // not specify any unique filters,
        // return an error
        // but what makes this update_one
        // different from update_many?
        // in update_many, you might want to
        // have some conditional update based on
        // each record
        let links = self.links.spec(&self.handler);
        if (self.wheres.is_unique(&self.handler) || links.is_unique(&self.handler)).not() {
            return Err(UpdateOneError::NonUniqueOperation);
        }
        if partial.is_op().not() && links.is_op().not() {
            return Err(UpdateOneError::NonOperational);
        }

        Ok(NeedCheck(UpdateOne {
            handler: self.handler,
            partial,
            wheres: self.wheres,
            links,
        }))
    }
}

impl<S, Handler, Wheres, Links> Operation<S>
    for NeedCheck<UpdateOne<Handler, Handler::UpdateExpression, Wheres, Links>>
where
    S: DatabaseExt,
    Handler: Send,
    Links: Send,
    Wheres: Send,
    S: Database,
    // from row
    Handler: for<'r> FromRowAlias<'r, S::Row, FromRowData = Handler::Data>,
    Handler: Collection<Data: Send, Id = SingleIncremintalInt> + 'static,
    Handler: OnUpdate<UpdateExpression: Send + ManyExpressions<'static, S>>,
    Links: Send
        + IsOpExpression
        + LinkUpdateOne<
            S,
            PreOp: Operation<S, Output: Send> + Send,
            UpdateSets: Send + ManyExpressions<'static, S>,
            UpdateWheres: ManyExpressions<'static, S>,
            UpdateReturning: ManyExpressions<'static, S>,
            PostOp: Operation<S>,
            Output: Send,
        >,
    for<'c> &'c mut <S as sqlx::Database>::Connection: Executor<'c, Database = S>,
    usize: ColumnIndex<S::Row>,
    i64: for<'q> Decode<'q, S> + Type<S>,
    Wheres: ManyExpressions<'static, S>,
    Wheres: IsUniqueFilter<Handler>,
    Handler: Members,
{
    type Output = Option<
        LinkedOutput<i64, <Handler as Collection>::Data, <Links as LinkUpdateOne<S>>::Output>,
    >;
    async fn exec_operation(self, pool: &mut S::Connection) -> Self::Output {
        let pre_op = self.0.links.pre_op().exec_operation(&mut *pool).await;
        let (sets, wheres, returning) = self.0.links.update_extention(pre_op);

        let qb = StatementBuilder::new(UpdateStatement {
            table_name: table(self.0.handler.table_name().to_string()),
            values: ManyFlat((self.0.partial, sets)),
            wheres: ManyFlat((self.0.wheres, wheres)),
            returning: ManyFlat(("id", returning, self.0.handler.members_names())),
        });

        let mut rest: Vec<_> = use_executor!(fetch_all(&mut *pool, qb)).unwrap_sqlx_error::<S>();

        if rest.len() == 0 {
            return None;
        }

        if rest.len() != 1 {
            panic!(
                "bug: make an update_one of multiple records! the bug is in the implementation of RuntimeCheck"
            )
        }

        let first_row = rest.pop().expect("bug: should have only one row");
        let id: i64 = first_row.try_get(0).expect("bug: should have first as id");

        let post_op = self
            .0
            .links
            .post_op(id, &first_row)
            .exec_operation(&mut *pool)
            .await;

        Some(LinkedOutput {
            id,
            attributes: self.0.handler.no_alias(&first_row).unwrap(),
            links: self.0.links.output(post_op),
        })
    }
}

#[cfg(test)]
mod test {
    use crate::connect_in_memory::ConnectInMemory;
    use crate::expressions::col_eq;
    use crate::operations::update_one::UpdateOne;
    use crate::operations::{Operation, SafeOperation};
    use crate::test_module::{self, *};
    use crate::update_mod::Update;
    use sqlx::{FromRow, Sqlite};

    #[tokio::test]
    async fn test_update_one() {
        let mut conn = Sqlite::connect_in_memory_2().await;

        sqlx::query(
            "
            CREATE TABLE IF NOT EXISTS Category (
                id INTEGER PRIMARY KEY AUTOINCREMENT,
                title TEXT NOT NULL
            );

            CREATE TABLE IF NOT EXISTS Todo (
                id INTEGER PRIMARY KEY AUTOINCREMENT,
                title TEXT NOT NULL,
                done BOOLEAN NOT NULL,
                description TEXT,
                category_id INTEGER REFERENCES Category(id) ON DELETE SET NULL
            );

            INSERT INTO Todo (id, title, done, description) VALUES 
                (1, 'first_todo', true, 'description_1'),
                (2, 'second_todo', false, NULL);

            ",
        )
        .execute(&mut conn)
        .await
        .unwrap();

        let s = UpdateOne {
            handler: test_module::todo,
            partial: TodoPartial {
                title: Update::Set("new_title".to_string()),
                done: Update::Keep,
                description: Update::Keep,
            },
            wheres: col_eq { col: "id", eq: 2 },
            links: (),
        }
        .safety_check()
        .unwrap();

        Operation::<Sqlite>::exec_operation(s, &mut conn)
            .await
            .unwrap();

        let row = sqlx::query("SELECT title FROM Todo WHERE id = 2;")
            .fetch_one(&mut conn)
            .await
            .unwrap();

        let s = <(String,) as FromRow<_>>::from_row(&row).unwrap().0;

        assert_eq!(s, "new_title".to_string());
    }
}
