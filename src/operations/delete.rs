use crate::{
    collections::{Collection, CollectionId},
    database_extention::DatabaseExt,
    execute::Executable,
    extentions::common_expressions::{Identifier, TableNameExpression},
    fix_executor::ExecutorTrait,
    from_row::FromRowAlias,
    operations::{LinkedOutput, Operation, OperationOutput},
    query_builder::{Expression, ManyExpressions, StatementBuilder, functional_expr::ManyFlat},
    statements::delete_statement::DeleteStatement,
};

pub struct Delete<Base, Wheres, Links> {
    pub base: Base,
    pub wheres: Wheres,
    pub links: Links,
}

pub trait LinkDelete {
    type Output;
}

impl LinkDelete for () {
    type Output = ();
}

impl<Base, Wheres, Links> OperationOutput for Delete<Base, Wheres, Links>
where
    Base: Collection,
    Links: LinkDelete,
{
    type Output = Vec<
        LinkedOutput<<Base::Id as CollectionId>::IdData, <Base as Collection>::Data, Links::Output>,
    >;
}

impl<S, Base, Wheres, Links> Operation<S> for Delete<Base, Wheres, Links>
where
    S: DatabaseExt + ExecutorTrait,
    Base: Send,
    Base: Collection,
    Base: Identifier<Identifier: for<'q> ManyExpressions<'q, S>>,
    Base: TableNameExpression<TableNameExpression: for<'q> Expression<'q, S>>,
    Base: for<'r> FromRowAlias<'r, S::Row, RData = Base::Data>,
    Base::Data: Send,
    Base::Id: Send + CollectionId<IdData: Send>,
    Base::Id: Identifier<Identifier: for<'q> ManyExpressions<'q, S>>,
    Base::Id: for<'r> FromRowAlias<'r, S::Row, RData = <Base::Id as CollectionId>::IdData>,
    Wheres: Send,
    Wheres: for<'q> ManyExpressions<'q, S>,
    Links: Send,
    Links: LinkDelete,
    Links::Output: Send,
    // for closure
    Base::Id: Sync,
    Base: Sync,
    // todo
    Links: LinkDelete<Output = ()>,
{
    fn exec_operation(self, pool: &mut <S>::Connection) -> impl Future<Output = Self::Output> + Send
    where
        S: sqlx::Database,
        Self: Sized,
    {
        async move {
            let id = self.base.id();

            let (stmt, args) = StatementBuilder::<S>::new(DeleteStatement {
                table_name: self.base.table_name_expression(),
                wheres: ManyFlat((self.wheres,)),
                returning: ManyFlat((id.identifier(), self.base.identifier())),
            })
            .unwrap();

            let res = S::fetch_all_mapped(
                &mut *pool,
                Executable {
                    string: &stmt,
                    arguments: args,
                },
                |row| {
                    let id = id.no_alias(&row).unwrap();
                    let attributes = self.base.no_alias(&row).unwrap();
                    LinkedOutput {
                        id,
                        attributes,
                        links: (),
                    }
                },
            )
            .await
            .unwrap();

            res
        }
    }
}

#[cfg(test)]
mod test {
    use crate::expressions::ColumnEqual;
    use crate::operations::{LinkedOutput, Operation};
    use crate::test_module::{self, *};
    use crate::{connect_in_memory::ConnectInMemory, operations::delete::Delete};
    use sqlx::{Row, Sqlite};

    #[tokio::test]
    async fn main() {
        let mut pool = Sqlite::connect_in_memory_2().await;

        sqlx::query(
            "
            CREATE TABLE Todo (
                id INTEGER PRIMARY KEY AUTOINCREMENT,
                title TEXT NOT NULL,
                done BOOLEAN NOT NULL,
                description TEXT
            );
            INSERT INTO Todo (title, done, description) VALUES ('todo_1', false, 'description_1'), ('todo_2', true, 'description_2'), ('todo_3', false, 'description_3');
        ",
        )
        .execute(&mut pool)
        .await
        .unwrap();

        let output = Operation::<Sqlite>::exec_operation(
            Delete {
                base: test_module::todo,
                wheres: ColumnEqual {
                    col: todo_members::id,
                    eq: 2,
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
                    title: String::from("todo_2"),
                    done: true,
                    description: Some(String::from("description_2")),
                },
                links: ()
            }]
        );

        let check = sqlx::query("SELECT * FROM Todo;")
            .fetch_all(&mut pool)
            .await
            .unwrap()
            .into_iter()
            .map(|row| row.get::<i64, _>("id"))
            .collect::<Vec<_>>();

        pretty_assertions::assert_eq!(check, vec![1, 3]);
    }
}
