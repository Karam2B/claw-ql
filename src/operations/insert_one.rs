use sqlx::{ColumnIndex, Database, Decode, Executor, Row, Type};

use crate::{
    collections::{Collection, HasHandler, Id, SingleIncremintalInt},
    database_extention::DatabaseExt,
    expressions::table,
    extentions::Members,
    links::Link,
    operations::{LinkedOutput, Operation},
    query_builder::{ManyExpressions, QueryBuilder},
    row_utils::RowToJson,
    singlton_default::SingltonDefault,
    statements::insert_one_statement::{InsertStatement, values_for_insert},
    use_executor,
};

pub struct InsertOne<Handler, Entry, Links> {
    pub handler: Handler,
    pub entry: Entry,
    pub links: Links,
}

impl<Entry, Links> InsertOne<Entry::Handler, Entry, Links>
where
    Entry: HasHandler,
    Entry::Handler: SingltonDefault + Clone,
{
    pub fn new(entry: Entry, links: Links) -> Self {
        let handler = Entry::Handler::singlton_default().clone();
        InsertOne {
            handler,
            entry,
            links,
        }
    }
}

pub trait LinkInsertOne<S> {
    type PreOp;
    type Resume;
    fn pre_op(self) -> (Self::PreOp, Self::Resume);

    fn current_table(o: <Self::PreOp as Operation<S>>::Output) -> Vec<String>
    where
        Self::PreOp: Operation<S>;
    type FromRow;
    type SubOp;
    fn from_row(&self, row: &S::Row) -> (Self::FromRow, Self::SubOp)
    where
        S: Database;
    type Output;
    fn take(
        self,
        from_row: Self::FromRow,
        sub_op: <Self::SubOp as Operation<S>>::Output,
    ) -> Self::Output
    where
        Self::SubOp: Operation<S>,
        S: Database;
}

pub mod returning_clause {
    use crate::{
        database_extention::DatabaseExt,
        query_builder::{Expression, OpExpression, QueryBuilder, syntax::comma_join},
    };

    #[allow(non_camel_case_types)]
    pub struct returning(pub Vec<String>);

    impl OpExpression for returning {}
    impl<'q, S: DatabaseExt> Expression<'q, S> for returning {
        fn expression(mut self, ctx: &mut QueryBuilder<'q, S>) {
            let pop = self.0.pop();
            for each in self.0 {
                ctx.sanitize(&each);
                ctx.syntax(&comma_join);
            }
            if let Some(last) = pop {
                ctx.sanitize(&last);
            }
        }
    }
}

impl<S, Handler, Entry, Links> Operation<S> for InsertOne<Handler, Entry, Links>
where
    S: DatabaseExt,
    Handler: Send,
    Handler: Members<S>,
    Entry: HasHandler<Handler = Handler> + Send,
    Entry: for<'q> ManyExpressions<'q, S>,
    Handler: Collection<Data = Entry, Id = SingleIncremintalInt>,
    for<'c> &'c mut <S as sqlx::Database>::Connection: Executor<'c, Database = S>,
    Links: Sync + Send,
    Links: Link<Handler, Spec: Send> + Send,
    <Links as Link<Handler>>::Spec:
        LinkInsertOne<S, Output: Send, FromRow: Send, SubOp: Send + Operation<S, Output: Send>>,
    i64: for<'q> Decode<'q, S> + Type<S>,
    usize: ColumnIndex<S::Row>,
    // remove later
    S::Row: RowToJson,
    String: for<'q> Decode<'q, S> + Type<S>,
{
    type Output = LinkedOutput<
        <<Entry::Handler as Collection>::Id as Id>::Data,
        (),
        <<Links as Link<Handler>>::Spec as LinkInsertOne<S>>::Output,
    >;
    async fn exec_operation(self, pool: sqlx::Pool<S>) -> Self::Output
    where
        S: sqlx::Database,
    {
        let query_builder = QueryBuilder::new(InsertStatement {
            table_name: table(self.handler.table_name()),
            identifiers: self.handler.members_names(),
            values: crate::statements::insert_one_statement::OneDefault(values_for_insert(
                self.entry,
            )),
            returning: returning_clause::returning(vec!["id".to_string()]),
        });

        let spec = self.links.spec(&self.handler);

        println!("{}", query_builder.stmt());

        let row = use_executor!(fetch_one(&pool, query_builder)).expect("bug: claw_ql must clear all sqlx's error, it is hard to know where this error was originated!");

        println!("{}", row.to_json());

        let (from_row, sub_op) = spec.from_row(&row);

        let sub_op = sub_op.exec_operation(pool.clone()).await;

        let links = spec.take(from_row, sub_op);

        let id: i64 = row.get(0);

        return LinkedOutput {
            id,
            attributes: (),
            links,
        };
    }
}

#[cfg(test)]
mod test {
    use super::InsertOne;
    use super::Operation;
    use crate::from_row::FromRowAlias;
    use crate::from_row::pre_alias;
    use crate::links::set_new_mod::set_new;
    use crate::test_module::*;
    use crate::{connect_in_memory::ConnectInMemory, test_module::Todo};
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
            
            ",
        )
        .execute(&pool)
        .await
        .unwrap();

        let _ = InsertOne::new(
            Todo {
                title: "first_todo".to_string(),
                done: true,
                description: Some("description_1".to_string()),
            },
            set_new(Category {
                title: "category_1".to_string(),
            }),
        )
        .exec_operation(pool.clone())
        .await;

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

        let result = todo.pre_alias(pre_alias(&row, "todo_")).unwrap();

        pretty_assertions::assert_eq!(
            result,
            Todo {
                title: "first_todo".to_string(),
                done: true,
                description: Some("description_1".to_string()),
            }
        );

        let category_id: i64 = row.get("category_id");

        pretty_assertions::assert_eq!(category_id, 0,);

        let result = category.pre_alias(pre_alias(&row, "category_")).unwrap();

        pretty_assertions::assert_eq!(
            result,
            Category {
                title: "category_1".to_string(),
            }
        );
    }
}
