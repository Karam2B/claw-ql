use crate::{
    collections::{Collection, SingleIncremintalInt},
    database_extention::DatabaseExt,
    extentions::common_expressions::OnInsert,
    from_row::row_helpers::OneRowHelper,
    links::{Link, relation_optional_to_many::OptionalToMany, set_id_mod::SetIdSpec},
    operations::{LinkedOutput, Operation, SafeOperation},
    query_builder::{Bind, ManyExpressions, StatementBuilder, functional_expr::ManyFlat},
    sqlx_error_handling::HandleSqlxResult,
    statements::insert_statement::{InsertStatement, One},
    use_executor,
};
use sqlx::{ColumnIndex, Database, Type};

pub trait LinkInsertOne<S> {
    type PreOp;
    fn pre_op(&self) -> Self::PreOp;

    type InsertExtentionIdents;
    type InsertExtentionValues;
    fn insert_extention(
        &self,
        output: <Self::PreOp as Operation<S>>::Output,
    ) -> (Self::InsertExtentionIdents, Self::InsertExtentionValues)
    where
        Self::PreOp: Operation<S>;

    type PostOp;
    fn post_op(&self, id: i64) -> Self::PostOp;
    type Output;
    fn output(&self, output: <Self::PostOp as Operation<S>>::Output) -> Self::Output
    where
        Self::PostOp: Operation<S>;
}

impl<S> LinkInsertOne<S> for () {
    type PreOp = ();

    fn pre_op(&self) -> Self::PreOp {}

    type InsertExtentionIdents = ();
    type InsertExtentionValues = ();

    fn insert_extention(
        &self,
        _: <Self::PreOp as Operation<S>>::Output,
    ) -> (Self::InsertExtentionIdents, Self::InsertExtentionValues)
    where
        Self::PreOp: Operation<S>,
    {
        ((), ())
    }

    type PostOp = ();

    fn post_op(&self, _: i64) -> Self::PostOp {}

    type Output = ();

    fn output(&self, _: <Self::PostOp as Operation<S>>::Output) -> Self::Output
    where
        Self::PostOp: Operation<S>,
    {
    }
}

impl<From, To, S> LinkInsertOne<S> for SetIdSpec<OptionalToMany<String, From, To>, i64> {
    type PreOp = ();

    fn pre_op(&self) -> Self::PreOp {}

    type InsertExtentionIdents = Vec<String>;
    type InsertExtentionValues = Bind<i64>;

    fn insert_extention(
        &self,
        _: <Self::PreOp as Operation<S>>::Output,
    ) -> (Vec<String>, Self::InsertExtentionValues)
    where
        Self::PreOp: Operation<S>,
    {
        (vec![self.og_spec.foriegn_key.clone()], Bind(self.input))
    }

    type PostOp = ();

    fn post_op(&self, _: i64) -> Self::PostOp {}

    type Output = ();

    fn output(&self, _: <Self::PostOp as Operation<S>>::Output) -> Self::Output
    where
        Self::PostOp: Operation<S>,
    {
    }
}

pub struct InsertOne<Handler, Data, Links> {
    pub handler: Handler,
    pub data: Data,
    pub links: Links,
}

#[derive(Debug)]
pub enum InsertOneError<T> {
    ValidationError(T),
}

impl<H, L> SafeOperation for InsertOne<H, H::DataInput, L>
where
    H: OnInsert,
    L: Link<H>,
{
    type Error = InsertOneError<H::DataError>;
    type Ok = InsertOne<H, H::InsertExpression, L::Spec>;
    fn safety_check(self) -> Result<Self::Ok, Self::Error> {
        let data = match self.handler.validate_on_insert(self.data) {
            Ok(data) => data,
            Err(e) => return Err(InsertOneError::ValidationError(e)),
        };
        let links = self.links.spec(&self.handler);
        Ok(InsertOne {
            handler: self.handler,
            data,
            links,
        })
    }
}

impl<S, H, L> Operation<S> for InsertOne<H, H::InsertExpression, L>
where
    H: Collection<Id = SingleIncremintalInt> + Send,
    H: OnInsert<InsertExpression: Send + ManyExpressions<'static, S>>,
    S: DatabaseExt,
    usize: ColumnIndex<S::Row>,
    i64: Type<S> + for<'q> sqlx::Decode<'q, S>,
    for<'c> &'c mut S::Connection: sqlx::Executor<'c, Database = S>,
    S::Connection: sqlx::Connection<Database = S>,
    L: LinkInsertOne<S>
        + Send
        + LinkInsertOne<
            S,
            PreOp: Operation<S>,
            InsertExtentionIdents: Send + ManyExpressions<'static, S>,
            InsertExtentionValues: Send + ManyExpressions<'static, S>,
            PostOp: Operation<S>,
            Output: Send,
        >,
    H: ManyExpressions<'static, S>,
    H: Send,
{
    type Output = LinkedOutput<i64, (), L::Output>;
    async fn exec_operation(self, conn: &mut <S>::Connection) -> Self::Output
    where
        S: Database,
    {
        let pre_op = self.links.pre_op().exec_operation(&mut *conn).await;
        let (idents, values) = self.links.insert_extention(pre_op);

        let qb = StatementBuilder::<'_, S>::new(InsertStatement {
            table_name: self.handler.table_name().to_string(),
            identifiers: ManyFlat((idents, self.handler)),
            values: One(ManyFlat((values, self.data))),
            returning: vec!["id"],
        });

        let id = use_executor!(fetch_one(&mut *conn, qb))
            .unwrap_sqlx_error::<S>()
            .from_row::<(i64,)>()
            .unwrap()
            .0;

        let s = self.links.post_op(id).exec_operation(&mut *conn).await;

        LinkedOutput {
            id,
            attributes: (),
            links: self.links.output(s),
        }
    }
}

#[cfg(test)]
mod test {
    use super::InsertOne;
    use crate::connect_in_memory::ConnectInMemory;
    use crate::from_row::row_helpers::AliasRowHelper;
    use crate::links::set_id_mod::set_id;
    use crate::operations::Operation;
    use crate::operations::SafeOperation;
    use crate::test_module;
    use crate::test_module::Category;
    use crate::test_module::Todo;
    use crate::test_module::category;
    use sqlx::Row;
    use sqlx::Sqlite;

    #[tokio::test]
    async fn test_insert_one() {
        let pool = Sqlite::connect_in_memory().await;

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
                category_id INTEGER,
                FOREIGN KEY (category_id) REFERENCES Category(id)
            );
            INSERT INTO Category (title) VALUES ('category_1');
            ",
        )
        .execute(&pool)
        .await
        .unwrap();

        let mut tx = pool.begin().await.unwrap();

        Operation::<Sqlite>::exec_operation(
            InsertOne {
                handler: test_module::todo,
                data: Todo {
                    title: "first_todo".to_string(),
                    done: true,
                    description: Some("description_1".to_string()),
                },
                links: set_id {
                    to: category,
                    id: 1,
                },
            }
            .safety_check()
            .unwrap(),
            tx.as_mut(),
        )
        .await;

        tx.commit().await.unwrap();

        // move to query
        let row = sqlx::query(
            "SELECT 
                    t.title as todo_title, 
                    t.done as todo_done, 
                    t.description as todo_description, 
                    t.category_id, 
                    c.title as category_title
                FROM Todo t LEFT JOIN Category c ON t.category_id = c.id;",
        )
        .fetch_one(&pool)
        .await
        .unwrap();

        let result = row.row_pre_alias(&test_module::todo, "todo_").unwrap();

        pretty_assertions::assert_eq!(
            result,
            Todo {
                title: "first_todo".to_string(),
                done: true,
                description: Some("description_1".to_string()),
            }
        );

        let category_id: i64 = row.get("category_id");

        pretty_assertions::assert_eq!(category_id, 1);

        let result = row.row_pre_alias(&category, "category_").unwrap();

        pretty_assertions::assert_eq!(
            result,
            Category {
                title: "category_1".to_string(),
            }
        );
    }
}
