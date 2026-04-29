use crate::{
    collections::{Collection, CollectionId},
    database_extention::DatabaseExt,
    execute::Executable,
    expressions::larger_than_or_equal::LargerThanOrEqual,
    extentions::common_expressions::{Aliased, OnInsert, Scoped},
    fix_executor::ExecutorTrait,
    from_row::{FromRowAlias, FromRowData, RowPreAliased},
    operations::{LinkedOutput, Operation, OperationOutput},
    query_builder::{
        Bind, Expression, ManyExpressions, StatementBuilder, functional_expr::ManyFlat,
    },
    statements::select_statement::SelectStatement,
};
use sqlx::{Encode, Type};

pub trait LinkFetch {
    type SelectItems;
    fn non_aggregating_select_items(&self) -> Self::SelectItems;

    type Join;
    fn non_duplicating_join_expressions(&self) -> Self::Join;

    type Wheres;
    fn where_expressions(&self) -> Self::Wheres;

    type Op;
    type OpInput;

    fn operation_construct_once(&self, item: &<Self::SelectItems as FromRowData>::RData) -> Self::Op
    where
        Self::SelectItems: FromRowData,
    {
        let mut ret = self.operation_initialize_input();
        self.operation_fix_on_many(item, &mut ret);
        self.operation_construct(ret)
    }

    fn operation_initialize_input(&self) -> Self::OpInput;

    fn operation_fix_on_many(
        &self,
        item: &<Self::SelectItems as FromRowData>::RData,
        poi: &mut Self::OpInput,
    ) where
        Self::SelectItems: FromRowData;

    fn operation_construct(&self, input: Self::OpInput) -> Self::Op
    where
        Self::SelectItems: FromRowData;

    type Output;

    fn take_once(
        self,
        item: <Self::SelectItems as FromRowData>::RData,
        mut op: <Self::Op as OperationOutput>::Output,
    ) -> Self::Output
    where
        Self: Sized,
        Self::SelectItems: FromRowData,
        Self::Op: OperationOutput,
    {
        self.take_many(item, &mut op)
    }

    fn take_many(
        &self,
        item: <Self::SelectItems as FromRowData>::RData,
        op: &mut <Self::Op as OperationOutput>::Output,
    ) -> Self::Output
    where
        Self::SelectItems: FromRowData,
        Self::Op: OperationOutput;
}

mod functional_impls {
    use super::LinkFetch;
    use crate::from_row::FromRowData;

    impl LinkFetch for () {
        type Output = ();
        type SelectItems = ();
        fn non_aggregating_select_items(&self) -> Self::SelectItems {}

        type Join = ();
        fn non_duplicating_join_expressions(&self) -> Self::Join {}

        type Wheres = ();

        fn where_expressions(&self) -> Self::Wheres {}

        type Op = ();
        type OpInput = ();
        fn operation_initialize_input(&self) -> Self::OpInput {}
        fn operation_construct(&self, _: Self::OpInput) -> Self::Op
        where
            Self::SelectItems: FromRowData,
        {
        }

        fn operation_fix_on_many(
            &self,
            _: &<Self::SelectItems as FromRowData>::RData,
            _: &mut Self::Op,
        ) where
            Self::SelectItems: FromRowData,
        {
        }

        fn take_many(
            &self,
            item: <Self::SelectItems as FromRowData>::RData,
            _: &mut <Self::Op as crate::operations::OperationOutput>::Output,
        ) -> Self::Output {
            item
        }
    }
}

pub struct FetchMany<From, Links, Wheres, Order, FirstItem> {
    pub base: From,
    pub wheres: Wheres,
    pub links: Links,
    pub cursor_order_by: Order,
    pub cursor_first_item: Option<FirstItem>,
    pub limit: i64,
}

#[derive(Debug, PartialEq, Eq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize))]
pub struct ManyOutput<T, Next> {
    pub items: Vec<T>,
    pub next_item: Option<Next>,
}

impl<B, L, W, O, F> OperationOutput for FetchMany<B, L, W, O, (<B::Id as CollectionId>::IdData, F)>
where
    B: Collection,
    L: LinkFetch,
    O: FromRowData,
{
    type Output = ManyOutput<
        LinkedOutput<<B::Id as CollectionId>::IdData, B::Data, L::Output>,
        (<B::Id as CollectionId>::IdData, O::RData),
    >;
}

impl<S, Base, Links, Wheres, OrderBy, First> Operation<S>
    for FetchMany<Base, Links, Wheres, OrderBy, (<Base::Id as CollectionId>::IdData, First)>
where
    S: DatabaseExt,
    S: ExecutorTrait,
    Base: Send,
    OrderBy: Send,
    First: Send,
    Wheres: Send,
    Wheres: for<'q> ManyExpressions<'q, S>,
    Links: Send + LinkFetch<Output: Send>,
    Links::Wheres: for<'q> ManyExpressions<'q, S>,
    Links::SelectItems: Send + Aliased<Aliased: for<'q> ManyExpressions<'q, S>>,
    Links::SelectItems: for<'r> FromRowAlias<'r, S::Row, RData: Send>,
    Links::Join: for<'q> ManyExpressions<'q, S>,
    Links::Op: Operation<S>,
    Links::OpInput: Send,
    Base: Collection<Data: Send, Id: Send>,
    Base: Aliased<Aliased: for<'q> ManyExpressions<'q, S>>,
    Base: FromRowData<RData = Base::Data>,
    Base: for<'r> FromRowAlias<'r, S::Row>,
    Base::Id: FromRowData<RData = <Base::Id as CollectionId>::IdData>,
    Base::Id: for<'r> FromRowAlias<'r, S::Row>,
    Base::Id: CollectionId<IdData: Send + for<'q> Encode<'q, S> + Type<S>>,
    Base::Id: Scoped<Scoped: for<'q> Expression<'q, S>>,
    Base::Id: Aliased<Aliased: for<'q> Expression<'q, S>>,
    Links: LinkFetch<Output: Send>,
    i64: for<'q> Encode<'q, S> + Type<S>,
    OrderBy: Send + Clone,
    OrderBy: Scoped<Scoped: for<'q> ManyExpressions<'q, S>>,
    OrderBy: for<'r> FromRowAlias<'r, S::Row, RData: Send>,
    First: Send,
    First: Scoped<Scoped: for<'q> ManyExpressions<'q, S>>,
    First: OnInsert<InsertInput = (), InsertExpression: for<'q> ManyExpressions<'q, S>>,
{
    async fn exec_operation(self, pool: &mut S::Connection) -> Self::Output {
        // let db = S::singleton();
        let id = self.base.id();
        let link_items = self.links.non_aggregating_select_items();
        let query_builder = StatementBuilder::<'_, S>::new(SelectStatement {
            select_items: ManyFlat((
                id.aliased("i"),
                self.base.aliased("b"),
                link_items.aliased("l"),
            )),
            from: self.base.table_name().to_string(),
            joins: self.links.non_duplicating_join_expressions(),
            group_by: (),
            order: self.cursor_order_by.scoped(),
            wheres: ManyFlat((
                self.wheres,
                self.links.where_expressions(),
                self.cursor_first_item.map(|(id, first)| {
                    let idents = first.scoped();
                    let first = first.on_insert(());
                    LargerThanOrEqual {
                        id: ManyFlat((idents, self.base.id().scoped())),
                        values: ManyFlat((first, Bind(id))),
                    }
                }),
            )),
            limit: Bind(self.limit + 1),
        });

        let (stmt, arg) = query_builder.unwrap();

        tracing::info!(sql_stmt = %stmt, "fetch many");

        let mut s = S::fetch_all(
            &mut *pool,
            Executable {
                string: &stmt,
                arguments: arg,
            },
        )
        .await
        .unwrap();

        let has_more = if s.len() == (self.limit + 1) as usize {
            let last = s
                .pop()
                .expect("bug: len is usize + 1, should have last item to pop");
            let next = self
                .cursor_order_by
                .pre_alias(RowPreAliased::new(&last, "b"))
                .unwrap();
            let id = id.pre_alias(RowPreAliased::new(&last, "i")).unwrap();
            Some((id, next))
        } else {
            None
        };

        let mut input = self.links.operation_initialize_input();

        let all = s
            .into_iter()
            .map(|e| {
                let id = id.pre_alias(RowPreAliased::new(&e, "i")).unwrap();
                let link = link_items.pre_alias(RowPreAliased::new(&e, "l")).unwrap();
                self.links.operation_fix_on_many(&link, &mut input);
                return LinkedOutput {
                    id,
                    attributes: self.base.pre_alias(RowPreAliased::new(&e, "b")).unwrap(),
                    links: link,
                };
            })
            .collect::<Vec<_>>();

        let mut po = self
            .links
            .operation_construct(input)
            .exec_operation(&mut *pool)
            .await;

        let all = all
            .into_iter()
            .map(|e| LinkedOutput {
                id: e.id,
                attributes: e.attributes,
                links: self.links.take_many(e.links, &mut po),
            })
            .collect::<Vec<_>>();

        ManyOutput {
            items: all,
            next_item: has_more,
        }
    }
}

#[cfg(test)]
mod test {
    use sqlx::{Sqlite, query};

    use crate::{
        connect_in_memory::ConnectInMemory,
        extentions::common_expressions::OnInsert,
        operations::{
            LinkedOutput, Operation, SafeOperation,
            fetch_many::{FetchMany, ManyOutput},
        },
        test_module::{self, Todo, todo_members},
    };

    #[tokio::test]
    async fn main() {
        let mut conn = Sqlite::connect_in_memory_2().await;

        query(
            "
        CREATE TABLE Todo (
            id INTEGER PRIMARY KEY AUTOINCREMENT,
            title TEXT NOT NULL,
            done BOOLEAN NOT NULL,
            description TEXT
        );

        INSERT INTO Todo (title, done, description) VALUES
            ('non_unique', true, 'description_1'),
            ('second_todo', false, 'description_2'),
            ('third_todo', true, 'description_3'),
            ('non_unique', false, 'description_4'),
            ('fifth_todo', true, 'description_5'),
            ('sixth_todo', false, 'description_6');
    ",
        )
        .execute(&mut conn)
        .await
        .unwrap();

        let output = Operation::<Sqlite>::exec_operation(
            FetchMany {
                base: test_module::todo,
                wheres: (),
                links: (),
                cursor_order_by: todo_members::title,
                cursor_first_item: Some((
                    4,
                    todo_members::title.on_insert(String::from("non_unique")),
                )),
                limit: 2,
            },
            &mut conn,
        )
        .await;

        pretty_assertions::assert_eq!(
            output,
            ManyOutput {
                items: vec![
                    LinkedOutput {
                        id: 4,
                        attributes: Todo {
                            title: "non_unique".to_string(),
                            done: false,
                            description: Some("description_4".to_string()),
                        },
                        links: (),
                    },
                    LinkedOutput {
                        id: 2,
                        attributes: Todo {
                            title: "second_todo".to_string(),
                            done: false,
                            description: Some("description_2".to_string()),
                        },
                        links: (),
                    },
                ],
                next_item: Some((6, String::from("sixth_todo"))),
            }
        );
    }
}

#[claw_ql_macros::skip]
mod deprecate_vec_impls {

    use super::LinkFetchMany;
    use crate::collections::CollectionId;
    use crate::from_row::FromRowData;
    use crate::operations::OperationOutput;
    use crate::operations::fetch_many_cursor_multi_col::LinkFetchManyTakeId;
    use crate::operations::fetch_many_cursor_multi_col::functional_impls::select_items_supported_in_vec::VecCollection;

    mod select_items_supported_in_vec {
        use std::ops::Not;

        use sqlx::Row;

        use crate::{
            expressions::multi_col_expressions_stack_heavy::AliasedCols,
            from_row::{FromRowAlias, FromRowData, RowTwoAliased},
            query_builder::{
                IsOpExpression, ManyExpressions, StatementBuilder, functional_expr::ManyFlat,
            },
        };

        impl IsOpExpression for (usize, AliasedCols<'static>) {
            fn is_op(&self) -> bool {
                self.1.cols.is_empty().not()
            }
        }

        impl<'q, S> ManyExpressions<'q, S> for (usize, AliasedCols<'static>) {
            fn expression(
                self,
                start: &'static str,
                join: &'static str,
                ctx: &mut StatementBuilder<'q, S>,
            ) where
                S: crate::database_extention::DatabaseExt,
            {
                let len = self.1.cols.len();
                if len == 0 {
                    return;
                }
                ctx.syntax(start);
                for (i, each) in self.1.cols.into_iter().enumerate() {
                    ctx.sanitize(self.1.table);
                    ctx.syntax(&".");
                    ctx.sanitize(each);
                    ctx.syntax(&" AS ");
                    ctx.sanitize_strings((self.1.alias, self.0, *each));
                    if i < len - 1 {
                        ctx.syntax(join);
                    }
                }
            }
        }

        pub struct DynamicAliasedCols {
            pub table: String,
            pub cols: Vec<String>,
            pub alias: String,
        }

        impl IsOpExpression for (usize, DynamicAliasedCols) {
            fn is_op(&self) -> bool {
                self.1.cols.is_empty().not()
            }
        }

        impl<'q, S> ManyExpressions<'q, S> for (usize, DynamicAliasedCols) {
            fn expression(
                self,
                start: &'static str,
                join: &'static str,
                ctx: &mut StatementBuilder<'q, S>,
            ) where
                S: crate::database_extention::DatabaseExt,
            {
                let len = self.1.cols.len();
                if len == 0 {
                    return;
                }
                ctx.syntax(start);
                for (i, each) in self.1.cols.into_iter().enumerate() {
                    ctx.sanitize(self.1.table.as_str());
                    ctx.syntax(&".");
                    ctx.sanitize(each.as_str());
                    ctx.syntax(&" AS ");
                    ctx.sanitize_strings((self.1.alias.as_str(), self.0, each.as_str()));
                    if i < len - 1 {
                        ctx.syntax(join);
                    }
                }
            }
        }

        pub struct VecCollection<T> {
            pub vec: Vec<T>,
        }
        // T: Into<DynamicAliasedCols> + Clone,

        impl<T> IsOpExpression for VecCollection<T>
        where
            T: Into<DynamicAliasedCols> + Clone,
        {
            fn is_op(&self) -> bool {
                self.vec.is_empty().not()
            }
        }

        impl<'q, S, T> ManyExpressions<'q, S> for VecCollection<T>
        where
            T: Into<DynamicAliasedCols> + 'q + Clone,
            T: ManyExpressions<'q, S>,
        {
            fn expression(
                self,
                start: &'static str,
                join: &'static str,
                ctx: &mut StatementBuilder<'q, S>,
            ) where
                S: crate::database_extention::DatabaseExt,
            {
                let ac = self
                    .vec
                    .into_iter()
                    .enumerate()
                    .map(|(i, e)| (i, e.into()))
                    .collect::<Vec<_>>();

                ManyExpressions::expression(ManyFlat(ac), start, join, ctx);
            }
        }

        impl IsOpExpression for VecCollection<AliasedCols<'static>> {
            fn is_op(&self) -> bool {
                self.vec.is_empty().not()
            }
        }

        impl<'q, S> ManyExpressions<'q, S> for VecCollection<AliasedCols<'static>> {
            fn expression(
                self,
                start: &'static str,
                join: &'static str,
                ctx: &mut StatementBuilder<'q, S>,
            ) where
                S: crate::database_extention::DatabaseExt,
            {
                ManyExpressions::expression(
                    ManyFlat(self.vec.into_iter().enumerate().collect::<Vec<_>>()),
                    start,
                    join,
                    ctx,
                );
            }
        }

        impl<T: FromRowData> FromRowData for VecCollection<T> {
            type RData = Vec<T::RData>;
        }

        impl<'r, R, T> FromRowAlias<'r, R> for VecCollection<T>
        where
            R: Row,
            T: FromRowAlias<'r, R>,
        {
            fn no_alias(&self, _: &'r R) -> Result<Self::RData, crate::from_row::FromRowError> {
                todo!()
            }

            fn pre_alias(
                &self,
                row: crate::from_row::RowPreAliased<'r, R>,
            ) -> Result<Self::RData, crate::from_row::FromRowError>
            where
                R: Row,
            {
                let mut ret = vec![];
                for (i, each) in self.vec.iter().enumerate() {
                    let s = each.two_alias(two_alias {
                        row: row.row,
                        str_alias: row.alias,
                        num_alias: Some(i),
                    });
                    if s.is_err() {
                        return Err(s.err().expect("bug: expected is_err"));
                    }
                    ret.push(s.unwrap());
                }
                Ok(ret)
            }

            fn post_alias(
                &self,
                _: crate::from_row::RowPostAliased<'r, R>,
            ) -> Result<Self::RData, crate::from_row::FromRowError>
            where
                R: Row,
            {
                todo!()
            }

            fn two_alias(
                &self,
                _: crate::from_row::RowTwoAliased<'r, R>,
            ) -> Result<Self::RData, crate::from_row::FromRowError>
            where
                R: Row,
            {
                panic!(
                    "bug: two_alias should never be called output of VecCollection associated imps"
                );
            }
        }

        // another impl for DynamicLink::SelectItems whatever that is
    }

    impl<T, I> LinkFetchManyTakeId<I> for Vec<T>
    where
        T: LinkFetchManyTakeId<I>,
    {
        fn take(
            &self,
            _: &I::IdData,
            _: &mut Vec<<Self::SelectItems as FromRowData>::RData>,
            _: &mut <Self::PostOperation as OperationOutput>::Output,
        ) -> Self::Output
        where
            I: CollectionId,
            Self::PostOperation: OperationOutput,
            Self::SelectItems: FromRowData,
        {
            todo!()
        }
    }

    impl<T> LinkFetch for Vec<T>
    where
        T: LinkFetch,
    {
        type Output = Vec<T::Output>;
        type SelectItems = VecCollection<T::SelectItems>;
        fn non_aggregating_select_items(&self) -> Self::SelectItems {
            let s: Vec<_> = self
                .iter()
                .map(|e| e.non_aggregating_select_items())
                .collect();

            VecCollection { vec: s }
        }
        type Join = ();
        fn non_duplicating_join(&self) -> Self::Join {
            todo!()
        }
        type Wheres = ();
        fn wheres(&self) -> Self::Wheres {
            todo!()
        }
        type PostOperation = ();

        fn post_select(&self) -> Self::PostOperation
        where
            Self::SelectItems: FromRowData,
        {
            todo!()
        }
    }
}
