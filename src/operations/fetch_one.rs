use crate::{
    collections::{Collection, CollectionId},
    database_extention::DatabaseExt,
    execute::Executable,
    extentions::common_expressions::{Aliased, TableNameExpression},
    fix_executor::ExecutorTrait,
    from_row::{FromRowAlias, RowPreAliased},
    operations::{LinkedOutput, Operation, OperationOutput, fetch_many::LinkFetch},
    query_builder::{Expression, ManyExpressions, StatementBuilder, functional_expr::ManyFlat},
    statements::select_statement::SelectStatement,
};

pub struct FetchOne<Base, Links, Wheres> {
    pub base: Base,
    pub links: Links,
    pub wheres: Wheres,
}

impl<B, L, W> OperationOutput for FetchOne<B, L, W>
where
    B: Collection,
    L: LinkFetch,
{
    type Output = Option<LinkedOutput<<B::Id as CollectionId>::IdData, B::Data, L::Output>>;
}

impl<S, Base, Links, Wheres> Operation<S> for FetchOne<Base, Links, Wheres>
where
    S: DatabaseExt,
    S: ExecutorTrait,
    Base: Send,
    Base: Collection,
    Base: for<'r> FromRowAlias<'r, S::Row, RData = Base::Data>,
    Base::Id: Send + CollectionId<IdData: Send>,
    Base::Id: Aliased<Aliased: for<'q> ManyExpressions<'q, S>>,
    Base::Id: for<'r> FromRowAlias<'r, S::Row, RData = <Base::Id as CollectionId>::IdData>,
    Base::Data: Send,
    Base: Aliased<Aliased: for<'q> ManyExpressions<'q, S>>,
    Base: TableNameExpression<TableNameExpression: for<'q> Expression<'q, S>>,
    Links: Send,
    Links: LinkFetch,
    Links::SelectItems: Send,
    Links::SelectItems: Aliased<Aliased: for<'q> ManyExpressions<'q, S>>,
    Links::SelectItems: for<'r> FromRowAlias<'r, S::Row, RData: Send>,
    Links::Output: Send,
    Links::Join: for<'q> ManyExpressions<'q, S>,
    Links::Wheres: for<'q> ManyExpressions<'q, S>,
    Links::Op: Operation<S>,
    Wheres: Send,
    Wheres: for<'q> ManyExpressions<'q, S>,
{
    fn exec_operation(self, pool: &mut <S>::Connection) -> impl Future<Output = Self::Output> + Send
    where
        S: sqlx::Database,
        Self: Sized,
    {
        async move {
            let lsi = self.links.non_aggregating_select_items();
            let id = self.base.id();

            let (stmt, args) = StatementBuilder::<'_, S>::new(SelectStatement {
                select_items: ManyFlat((
                    //
                    id.aliased("i"),
                    self.base.aliased("b"),
                    lsi.aliased("l"),
                )),
                from: self.base.table_name_expression(),
                joins: self.links.non_duplicating_join_expressions(),
                wheres: ManyFlat((self.wheres, self.links.where_expressions())),
                group_by: (),
                order: (),
                limit: (),
            })
            .unwrap();

            let row = S::fetch_optional(
                &mut *pool,
                Executable {
                    string: &stmt,
                    arguments: args,
                },
            )
            .await
            .unwrap()?;

            let id = id.pre_alias(RowPreAliased::new(&row, "i")).unwrap();
            let attributes = self.base.pre_alias(RowPreAliased::new(&row, "b")).unwrap();
            let link_items = lsi.pre_alias(RowPreAliased::new(&row, "l")).unwrap();

            let op = self
                .links
                .operation_construct_once(&link_items)
                .exec_operation(&mut *pool)
                .await;

            Some(LinkedOutput {
                id,
                attributes,
                links: self.links.take_once(link_items, op),
            })
        }
    }
}
