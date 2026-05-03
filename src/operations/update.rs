use std::ops::Not;

use crate::{
    collections::{Collection, CollectionId},
    database_extention::DatabaseExt,
    execute::Executable,
    extentions::common_expressions::{Identifier, OnUpdate, TableNameExpression},
    fix_executor::ExecutorTrait,
    from_row::FromRowAlias,
    operations::{LinkedOutput, Operation, OperationOutput},
    query_builder::{
        Expression, IsOpExpression, ManyExpressions, StatementBuilder, functional_expr::ManyFlat,
    },
    statements::update_statement::UpdateStatement,
};

pub struct Update<Base, Partial, Wheres, Links> {
    pub base: Base,
    pub partial: Partial,
    pub wheres: Wheres,
    pub links: Links,
}

pub trait UpdateLink {
    type Output;
}

impl UpdateLink for () {
    type Output = ();
}

impl<Handler, Partial, Wheres, Links> OperationOutput for Update<Handler, Partial, Wheres, Links>
where
    Handler: Collection,
    Links: UpdateLink,
{
    type Output =
        Vec<LinkedOutput<<Handler::Id as CollectionId>::IdData, Handler::Data, Links::Output>>;
}

impl<S, Base, Partial, Wheres, Links> Operation<S> for Update<Base, Partial, Wheres, Links>
where
    S: DatabaseExt,
    S: ExecutorTrait,
    Base: Clone,
    Base: Send,
    Base: Identifier<Identifier: Send + for<'q> ManyExpressions<'q, S>>,
    Base: TableNameExpression<TableNameExpression: for<'q> Expression<'q, S>>,
    Base: Collection<Data: Send>,
    Base: OnUpdate<UpdateInput = Partial, UpdateExpression: Send + for<'q> ManyExpressions<'q, S>>,
    Base: for<'r> FromRowAlias<'r, S::Row, RData = Base::Data>,
    Base::Id: Send + CollectionId<IdData: Send>,
    Base::Id: Identifier<Identifier: Send + for<'q> ManyExpressions<'q, S>>,
    Base::Id: for<'r> FromRowAlias<'r, S::Row, RData = <Base::Id as CollectionId>::IdData>,
    Partial: Send,
    Wheres: Send,
    Wheres: for<'q> ManyExpressions<'q, S>,
    Links: Send,
    Links: UpdateLink<Output: Send>,
    // continue here
    Links: UpdateLink<Output = ()>,
{
    fn exec_operation(self, pool: &mut <S>::Connection) -> impl Future<Output = Self::Output> + Send
    where
        S: sqlx::Database,
        Self: Sized,
    {
        async move {
            let id = self.base.id();

            let values = ManyFlat((self.base.clone().on_update(self.partial),));

            if values.is_op().not() {
                panic!(
                    "bug: update operation is not operational, the bug should be catched before using Update"
                );
            }

            let (stmt, args) = StatementBuilder::<'_, S>::new(UpdateStatement {
                table_name: self.base.table_name_expression(),
                wheres: ManyFlat((self.wheres,)),
                returning: ManyFlat((id.identifier(), self.base.identifier())),
                values,
            })
            .unwrap();

            let res = S::fetch_all(
                &mut *pool,
                Executable {
                    string: &stmt,
                    arguments: args,
                },
            )
            .await
            .unwrap()
            .into_iter()
            .map(|e| {
                let id = self.base.id().no_alias(&e).unwrap();
                let attributes = self.base.no_alias(&e).unwrap();
                LinkedOutput {
                    id,
                    attributes,
                    links: (),
                }
            });

            res.collect()
        }
    }
}

#[cfg(test)]
mod test {
    use crate::expressions::ColumnEqual;
    use crate::from_row::FromRowAlias;
    use crate::operations::{CollectionOutput, LinkedOutput, Operation};
    use crate::test_module::{self, *};
    use crate::update_mod::Update;
    use crate::{connect_in_memory::ConnectInMemory, operations::update::Update as UpdateOp};
    use sqlx::Sqlite;

    #[tokio::test]
    async fn main() {
        let mut pool = Sqlite::connect_in_memory_2().await;

        sqlx::query("
            CREATE TABLE Todo (
                id INTEGER PRIMARY KEY AUTOINCREMENT,
                title TEXT NOT NULL,
                done BOOLEAN NOT NULL,
                description TEXT
            );
            INSERT INTO Todo (title, done, description) VALUES ('todo_1', false, 'description_1'), ('todo_2', true, 'description_2'), ('todo_3', false, 'description_3');
        ").execute(&mut pool).await.unwrap();

        let output = Operation::<Sqlite>::exec_operation(
            UpdateOp {
                base: test_module::todo,
                wheres: ColumnEqual {
                    col: todo_members::id,
                    eq: 2,
                },
                partial: TodoPartial {
                    title: Update::Set("new_title".to_string()),
                    done: Update::Keep,
                    description: Update::Keep,
                },
                links: (),
            },
            &mut pool,
        )
        .await;

        pretty_assertions::assert_eq!(
            output,
            vec![LinkedOutput {
                id: 2,
                attributes: Todo {
                    title: "new_title".to_string(),
                    done: true,
                    description: Some("description_2".to_string()),
                },
                links: ()
            }]
        );

        let check = sqlx::query("SELECT * FROM Todo;")
            .fetch_all(&mut pool)
            .await
            .unwrap()
            .into_iter()
            .map(|row| CollectionOutput {
                id: todo_members::id.no_alias(&row).unwrap(),
                attributes: test_module::todo.no_alias(&row).unwrap(),
            })
            .collect::<Vec<_>>();

        pretty_assertions::assert_eq!(
            check,
            vec![
                CollectionOutput {
                    id: 1,
                    attributes: Todo {
                        title: "todo_1".to_string(),
                        done: false,
                        description: Some("description_1".to_string()),
                    },
                },
                CollectionOutput {
                    id: 2,
                    attributes: Todo {
                        title: "new_title".to_string(),
                        done: true,
                        description: Some("description_2".to_string()),
                    },
                },
                CollectionOutput {
                    id: 3,
                    attributes: Todo {
                        title: "todo_3".to_string(),
                        done: false,
                        description: Some("description_3".to_string()),
                    },
                },
            ]
        );
    }
}
