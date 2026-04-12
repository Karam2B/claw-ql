use crate::{
    collections::{Collection, SingleIncremintalInt},
    database_extention::DatabaseExt,
    expressions::table,
    operations::insert_one::returning_clause::returning,
    query_builder::QueryBuilder,
    query_builder::{ManyExpressions, functional_expr::ManyImplPossible},
    statements::update_statement::UpdateStatement,
    use_executor,
};
use sqlx::{ColumnIndex, Database, Decode, Executor, Pool, Type};

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
    fn pre_op(self) -> Self::PreOp;
}

impl<S, Handler, Partial, Wheres, Links> Operation<S> for UpdateOne<Handler, Partial, Wheres, Links>
where
    S: DatabaseExt,
    Handler: Send,
    Partial: Send,
    Links: Send,
    Wheres: Send,
    S: Database,
    Handler: Collection<Data: Send, Id = SingleIncremintalInt> + 'static,
    Partial: ManyExpressions<'static, S>,
    Links: Link<Handler>,
    Links::Spec: Send + LinkUpdateOne<S, PreOp: Operation<S, Output: Send> + Send>,
    for<'c> &'c mut <S as sqlx::Database>::Connection: Executor<'c, Database = S>,
    usize: ColumnIndex<S::Row>,
    i64: for<'q> Decode<'q, S> + Type<S>,
    Wheres: ManyExpressions<'static, S>,
    // to delete
    String: for<'q> Decode<'q, S> + Type<S>,
    for<'s> &'s str: ColumnIndex<S::Row>,
{
    type Output = Option<LinkedOutput<(), (), ()>>;
    async fn exec_operation(self, pool: Pool<S>) -> Self::Output {
        let mut tx = pool.begin().await.unwrap();

        let query_builder = QueryBuilder::new(UpdateStatement {
            table_name: table(self.handler.table_name().to_string()),
            values: self.partial,
            wheres: (),
            // wheres: (ManyImplPossible {
            //     start: "",
            //     join: " AND ",
            //     expressions: self.wheres,
            // },),
            returning: returning(vec!["title".to_string(), "id".to_string()]),
        });

        todo!("fix the where");

        let rest: Vec<_> = use_executor!(fetch_all(tx.as_mut(), query_builder))
            .expect("internal bug: claw_ql should have cleared all sqlx error at this point");

        if rest.len() == 0 {
            return None;
        }

        if rest.len() != 1 {
            tx.rollback().await.unwrap();
            panic!("make an update_one of multiple records!")
        }

        tx.commit().await.unwrap();

        Some(LinkedOutput {
            id: (),
            attributes: (),
            links: (),
        })
    }
}

#[cfg(test)]
mod test {
    use super::UpdateOne;
    use crate::connect_in_memory::ConnectInMemory;
    use crate::expressions::{col_eq, scoped_column};
    use crate::from_row::{FromRowAlias, pre_alias};
    use crate::links::set_new_mod::set_new;
    use crate::operations::Operation;
    use crate::test_module::*;
    use crate::update_mod::update;
    use sqlx::{Row, Sqlite};

    #[tokio::test]
    async fn test_update_one() {
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
                category_id INTEGER REFERENCES Category(id) ON DELETE SET NULL
            );

            INSERT INTO Todo (id, title, done, description) VALUES 
                (1, 'first_todo', true, 'description_1'),
                (2, 'second_todo', false, NULL);

            ",
        )
        .execute(&pool)
        .await
        .unwrap();

        // trait Op2<S: Database> {
        //     async fn exec_operation<S: Database>(pool: Pool<S>) {}

        // }

        // let mut s = pool.begin().await.unwrap();

        // Executor::fetch_one(
        //     &pool,
        //     Executable {
        //         string: "sdf",
        //         arguments: Default::default(),
        //     },
        // )
        // .await
        // .unwrap();

        // exec_operation()

        UpdateOne {
            handler: todo,
            partial: TodoPartial {
                title: update::set("new_title".to_string()),
                done: update::keep,
                description: update::keep,
            },
            wheres: col_eq {
                col: scoped_column {
                    table: "Todo",
                    column: "title",
                },
                eq: "second_todo",
            },
            links: set_new(Category {
                title: "category_1".to_string(),
            }),
        }
        .exec_operation(pool.clone())
        .await
        .unwrap();

        let row = sqlx::query(
            "
            SELECT 
                t.title as todo_title, 
                t.done as todo_done, 
                t.description as todo_description, 
                t.category_id, 
                c.title as category_title
            FROM Todo t LEFT JOIN Category c ON t.category_id = c.id
            WHERE t.id = 2 
            ;
            ",
        )
        .fetch_one(&pool)
        .await
        .unwrap();

        let todo_data = todo.pre_alias(pre_alias(&row, "todo_")).unwrap();

        pretty_assertions::assert_eq!(
            todo_data,
            Todo {
                title: "new_title".to_string(),
                done: false,
                description: None,
            }
        );

        let category_id: i64 = row.get("category_id");

        pretty_assertions::assert_eq!(category_id, 0);

        let category_data = category.pre_alias(pre_alias(&row, "category_")).unwrap();

        pretty_assertions::assert_eq!(
            category_data,
            Category {
                title: "category_1".to_string()
            }
        );
    }
}
