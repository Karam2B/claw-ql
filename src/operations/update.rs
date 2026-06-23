use std::ops::Not;

use crate::{
    collections::{Collection, CollectionId},
    database_extention::DatabaseExt,
    execute::Executable,
    fix_executor::ExecutorTrait,
    from_row::{FromRowAlias, FromRowData},
    operations::{LinkedOutput, Operation, OperationOutput, insert::ConstraintViolation, operations_expressions_crossover::{ExpressionsForOperation, OnUpdate, TableExpressions}},
    sqlx_query_builder::{
        Expression, IsOpExpression, ManyExpressions, StatementBuilder, basic_expressions::ManyFlat, statements::update_statement::UpdateStatement,
    },
};

pub struct Update<Base, Partial, Wheres, Links> {
    pub base: Base,
    pub partial: Partial,
    pub wheres: Wheres,
    pub links: Links,
}

pub trait UpdateLinkSplit {
    type Link: UpdateLink;
    fn init_split(
        self,
    ) -> (
        Self::Link,
        UpdateLinkData<
            <Self::Link as UpdateLink>::InitSplitForWheres,
            <Self::Link as UpdateLink>::InitSplitForUpdateValues,
            <Self::Link as UpdateLink>::InitSplitForPreOp,
            <Self::Link as UpdateLink>::InitSplitPostOp,
        >,
    );
}

pub struct UpdateLinkData<Wheres, UpdateValues, PreOp, PostOp> {
    pub wheres: Wheres,
    pub update_values: UpdateValues,
    pub pre_op: PreOp,
    pub post_op: PostOp,
}

pub trait UpdateLink {
    type InitSplitForPreOp;

    type PreOpSplitWheres;
    type PreOpSplitValues;
    type PreOpSplitPostOp;
    type PreOpSplitTake;
    type PreOp: OperationOutput;
    fn pre_op(&self, init_split_for_pre_op: Self::InitSplitForPreOp) -> Self::PreOp;
    fn split_pre_op(
        &self,
        pre_op: <Self::PreOp as OperationOutput>::Output,
    ) -> Result<(Self::PreOpSplitWheres, Self::PreOpSplitValues, Self::PreOpSplitPostOp, Self::PreOpSplitTake), ConstraintViolation>;

    type InitSplitForWheres;

    type UpdateWhere;
    fn wheres(&self, wheres: Self::InitSplitForWheres) -> Self::UpdateWhere;

    type UpdateNames;
    fn update_names(&self) -> Self::UpdateNames;

    type InitSplitForUpdateValues;
    type UpdateValues;
    fn update_values(&self, values: Self::InitSplitForUpdateValues,
    pre_op_output: Self::PreOpSplitValues,
    ) -> Self::UpdateValues;

    type FromRow: FromRowData;
    fn from_row(&self) -> Self::FromRow;

    type PostOp: OperationOutput;
    type InitSplitPostOp;
    fn post_op(
        &self,
        from_init_split: Self::InitSplitPostOp,
        from_pre_op: Self::PreOpSplitPostOp,
    ) -> Self::PostOp;

    fn from_row_result(
        &self,
        row_data: &<Self::FromRow as FromRowData>::RData,
        post_op: &mut Self::PostOp,
    );

    type Output;
    type PostOpOutput;
    fn post_op_output(&self,
        poo: <Self::PostOp as OperationOutput>::Output,
    ) -> Result<Self::PostOpOutput, ConstraintViolation> ;

    fn take(
        &self,
        from_row: <Self::FromRow as FromRowData>::RData,
        post_op: &mut Self::PostOpOutput,
        pre_op_split_take: &mut Self::PreOpSplitTake,
    ) -> Self::Output;
}

impl UpdateLinkSplit for () {
    type Link = ();
    fn init_split(self) -> (Self::Link, UpdateLinkData<(), (), (), ()>) {
        (
            (),
            UpdateLinkData {
                wheres: (),
                update_values: (),
                pre_op: (),
                post_op: (),
            },
        )
    }
}

impl UpdateLink for () {
    type InitSplitForWheres = ();
    type UpdateWhere = ();
    type PreOp = ();
    fn pre_op(&self, _: Self::InitSplitForPreOp) -> Self::PreOp {}
    fn wheres(&self, _: Self::InitSplitForWheres) -> Self::UpdateWhere {}

    type InitSplitForPreOp = ();
    type PreOpSplitWheres = ();
    type PreOpSplitValues = ();
    type PreOpSplitPostOp = ();
    type PreOpSplitTake = ();
    fn split_pre_op(
        &self,
        _: (),
    ) -> Result<(Self::PreOpSplitWheres, Self::PreOpSplitValues, Self::PreOpSplitPostOp, Self::PreOpSplitTake), ConstraintViolation> {
        Ok(((), (), (), ()))
    }

    type UpdateNames = ();
    fn update_names(&self) -> Self::UpdateNames {}

    type InitSplitForUpdateValues = ();
    type UpdateValues = ();
    fn update_values(&self, _: Self::InitSplitForUpdateValues, _: Self::PreOpSplitValues) -> Self::UpdateValues {}

    type FromRow = ();
    fn from_row(&self) -> Self::FromRow {}

    type PostOp = ();
    type InitSplitPostOp = ();
    fn post_op(&self, _: Self::InitSplitPostOp, _: Self::PreOpSplitPostOp) -> Self::PostOp {}
    fn from_row_result(&self, _: &<Self::FromRow as FromRowData>::RData, _: &mut Self::PostOp) {}

    type PostOpOutput = ();
    fn post_op_output(&self,
        _: <Self::PostOp as OperationOutput>::Output,
    ) -> Result<Self::PostOpOutput, ConstraintViolation> {
        Ok(())
    }

    type Output = ();
    fn take(
        &self,
        _: <Self::FromRow as FromRowData>::RData,
        _: &mut <Self::PostOp as OperationOutput>::Output,
        _: &mut Self::PreOpSplitTake,
    ) -> Self::Output {
    }
}

impl<Handler, Partial, Wheres, PL, Links> OperationOutput for Update<Handler, Partial, Wheres, PL>
where
    Handler: Collection,
    PL: UpdateLinkSplit<Link = Links>,
    Links: UpdateLink,
{
    type Output = Result<
        Vec<LinkedOutput<<Handler::Id as CollectionId>::IdData, Handler::OutputData, Links::Output>>,
        ConstraintViolation,
    >;
}

impl<S, Base, Partial, Wheres, PreSplitLink, Links> Operation<S>
    for Update<Base, Partial, Wheres, PreSplitLink>
where
    S: DatabaseExt,
    S: ExecutorTrait,
    Base: Clone,
    Base: Send,
    Base: TableExpressions<
        Identifier: Send + for<'q> ManyExpressions<'q, S>,
        PascalCase: for<'q> Expression<'q, S>,
    >,
    Base: OnUpdate<
        Partial,
        UpdateExpression: Send + for<'q> ManyExpressions<'q, S>,
    >,
    Base::Id: ExpressionsForOperation<
    Identifier: Send + for<'q> ManyExpressions<'q, S>,
    >,
    // Base: Identifier<Identifier: Send + for<'q> ManyExpressions<'q, S>>,
    // Base: TableNameExpression<TableNameExpression: for<'q> Expression<'q, S>>,
    Base: Collection<OutputData: Send>,
    // Base:
    //     V0OnUpdate<UpdateInput = Partial, UpdateExpression: Send + for<'q> ManyExpressions<'q, S>>,
    Base: for<'r> FromRowAlias<'r, S::Row, RData = Base::OutputData>,
    Base::Id: Send + CollectionId<IdData: Send>,
    // Base::Id: Identifier<Identifier: Send + for<'q> ManyExpressions<'q, S>>,
    Base::Id: for<'r> FromRowAlias<'r, S::Row, RData = <Base::Id as CollectionId>::IdData>,
    Partial: Send,
    Wheres: Send,
    Wheres: for<'q> ManyExpressions<'q, S>,
    PreSplitLink: Send + UpdateLinkSplit<Link = Links>,
    Links: Send + UpdateLink,
    Links::InitSplitForWheres: Send,
    Links::UpdateWhere: for<'q> ManyExpressions<'q, S>,
    Links::InitSplitForUpdateValues: Send,
    Links::UpdateValues: Send + for<'q> ManyExpressions<'q, S>,
    Links::UpdateNames: Send + for<'q> ManyExpressions<'q, S>,
    Links::FromRow: Send + for<'r> FromRowAlias<'r, S::Row, RData: Send>,
    Links::InitSplitPostOp: Send,
    Links::InitSplitForPreOp: Send,
    Links::PreOp: Send + Operation<S>,
    Links::PreOpSplitWheres: Send + for<'q> ManyExpressions<'q, S>,
    // Links::PreOpSplitValues: Send + for<'q> ManyExpressions<'q, S>,
    Links::PostOp: Send + Operation<S>,
    Links::Output: Send,
    Links::PreOpSplitTake: Send,
{
    fn exec_operation(self, pool: &mut <S>::Connection) -> impl Future<Output = Self::Output> + Send
    where
        S: sqlx::Database,
        Self: Sized,
    {
        async move {
            let (self_link, self_link_data) = self.links.init_split();
            let id = self.base.id();

            let pre_op = self_link.pre_op(self_link_data.pre_op).exec_operation(&mut *pool).await;

            let (pre_op_wheres, pre_op_values, pre_op_split_for_post_op, mut pre_op_split_take) =
                self_link.split_pre_op(pre_op)?;

            let values = ManyFlat((
                self.base.clone().on_update(self.partial),
                self_link.update_values(self_link_data.update_values, pre_op_values),
            ));

            if values.is_op().not() {
                panic!(
                    "bug: update operation is not operational, the bug should be catched before using Update"
                );
            }

            let (stmt, args) = StatementBuilder::<S>::new(UpdateStatement {
                table_name: self.base.table_name_pascal_case(),
                wheres: ManyFlat((
                    self.wheres,
                    self_link.wheres(self_link_data.wheres),
                    pre_op_wheres,
                )),
                returning: ManyFlat((
                    id.identifier(),
                    self.base.identifier(),
                    self_link.update_names(),
                )),
                values,
            })
            .unwrap();

            let link_from_row = self_link.from_row();
            let mut from_row_data = vec![];
            let mut post_op = self_link.post_op(self_link_data.post_op, pre_op_split_for_post_op);

            let res = S::fetch_all(
                &mut *pool,
                Executable {
                    string: &stmt,
                    arguments: args,
                },
            )
            .await
            .map_err(|e| {
                if let Some(db) = e.as_database_error() {
                    if db.is_check_violation() || db.is_unique_violation() || db.is_foreign_key_violation() {
                        return ConstraintViolation(db.constraint().map(|c| c.to_string()));
                    }
                }
            
                tracing::error!(sqlx_error = ?e, "bug: must clear all sqlx errors, hard to know where this error was originated!");
                panic!()
            })?
            .into_iter()
            .map(|e| {
                let id = self.base.id().no_alias(&e).unwrap();
                let attributes = self.base.no_alias(&e).unwrap();
                let link_r = link_from_row.no_alias(&e).unwrap();
                from_row_data.push(link_r);

                LinkedOutput {
                    id,
                    attributes,
                    links: (),
                }
            })
            .collect::<Vec<_>>();

            from_row_data.iter().for_each(|e| {
                self_link.from_row_result(e, &mut post_op);
            });

            let  poo = post_op.exec_operation(pool).await;
            let mut poo = self_link.post_op_output(poo)?;

            Ok(res
                .into_iter()
                .zip(from_row_data.into_iter())
                .map(|(e, f)| LinkedOutput {
                    id: e.id,
                    attributes: e.attributes,
                    links: self_link.take(f, &mut poo, &mut pre_op_split_take),
                })
                .collect())
        }
    }
}

#[cfg(test)]
mod test {
    use crate::collections::Collection;
    use crate::from_row::FromRowAlias;
    use crate::operations::{CollectionOutput, LinkedOutput, Operation};
    use crate::sqlx_query_builder::basic_expressions::{ColumnEqual, ScopedColumn};
    use crate::test_module::{Todo, TodoHandler, TodoPartial};
    use crate::update_mod::Update;
    use crate::{connect_in_memory::ConnectInMemory, operations::update::Update as UpdateOp};
    use sqlx::Sqlite;

    #[tokio::test]
    async fn main() {
        let mut pool = Sqlite::in_memory_connection().await;

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
                base: TodoHandler,
                wheres: ColumnEqual {
                    col: ScopedColumn {
                        table: ("Todo",),
                        col: ("id",),
                    } ,
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
        .await
        .unwrap();

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
                id: TodoHandler.id().no_alias(&row).unwrap(),
                attributes: TodoHandler.no_alias(&row).unwrap(),
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
