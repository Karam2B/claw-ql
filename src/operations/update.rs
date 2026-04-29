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
    Base: OnUpdate<UpdateInput = Partial, UpdateExpression: for<'q> ManyExpressions<'q, S>>,
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

            let returning = ManyFlat((id.identifier(), self.base.identifier()));

            if returning.is_op().not() {
                panic!(
                    "bug: update operation is not operational, the bug should be catched before using Update"
                );
            }

            let (stmt, args) = StatementBuilder::<'_, S>::new(UpdateStatement {
                table_name: self.base.table_name_expression(),
                wheres: ManyFlat((self.wheres,)),
                returning,
                values: self.base.clone().on_update(self.partial),
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

pub struct OnOneRecord<Operation> {
    pub operation: Operation,
}

impl<V, Op: OperationOutput<Output = Vec<V>>> OperationOutput for OnOneRecord<Op> {
    type Output = Option<V>;
}

impl<V, S, Op> Operation<S> for OnOneRecord<Op>
where
    V: Send,
    Op: Operation<S, Output = Vec<V>>,
{
    fn exec_operation(self, pool: &mut <S>::Connection) -> impl Future<Output = Self::Output> + Send
    where
        S: sqlx::Database,
        Self: Sized,
    {
        async move {
            let mut res = self.operation.exec_operation(pool).await;

            let last = res.pop();

            if res.len() != 0 {
                panic!("made an operation on multiple records!")
            }

            return last;
        }
    }
}
