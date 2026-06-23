use crate::{
    collections::{Collection, CollectionId},
    database_extention::DatabaseExt,
    execute::Executable,
    fix_executor::ExecutorTrait,
    from_row::{FromRowAlias, FromRowData, RowPreAliased},
    operations::{
        LinkedOutput, Operation, OperationOutput,
        operations_expressions_crossover::{
            ExpressionsForOperation, OnInsert, SelfPrescribedInsert,
        },
    },
    sqlx_query_builder::{
        ManyExpressions, StatementBuilder,
        basic_expressions::{Bind, ManyColumnsLargerOrEqual, ManyFlat},
        statements::select_statement::SelectStatement,
    },
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

mod std_impls {
    use super::LinkFetch;
    use crate::from_row::{RowPostAliased, RowPreAliased, RowTwoAliased};
    use crate::{
        from_row::{FromRowAlias, FromRowData, FromRowError},
        operations::operations_expressions_crossover::ExpressionsForOperation,
    };
    use sqlx::Row;

    pub struct Empty;

    impl LinkFetch for () {
        type Output = ();
        // maybe should be replaced by ()
        type SelectItems = Empty;
        fn non_aggregating_select_items(&self) -> Self::SelectItems {
            Empty
        }

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

    impl ExpressionsForOperation for Empty {
        type Identifier = ();
        fn identifier(&self) -> Self::Identifier {
            ()
        }
        type Scoped = ();
        fn scoped(&self) -> Self::Scoped {
            ()
        }
        type ScopedAliased = ();
        fn scoped_aliased(&self, _: &'static str) -> Self::ScopedAliased {
            ()
        }
        type NumScopedAliased = ();
        fn num_scoped_aliased(&self, _: usize, _: &'static str) -> Self::NumScopedAliased {
            ()
        }
    }

    impl FromRowData for Empty {
        type RData = ();
    }

    impl<'r, R> FromRowAlias<'r, R> for Empty
    where
        R: Row,
    {
        fn no_alias(&self, _: &'r R) -> Result<Self::RData, FromRowError> {
            Ok(())
        }
        fn pre_alias(&self, _: RowPreAliased<'r, R>) -> Result<Self::RData, FromRowError> {
            Ok(())
        }
        fn post_alias(&self, _: RowPostAliased<'r, R>) -> Result<Self::RData, FromRowError> {
            Ok(())
        }
        fn two_alias(&self, _: RowTwoAliased<'r, R>) -> Result<Self::RData, FromRowError> {
            Ok(())
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
        LinkedOutput<<B::Id as CollectionId>::IdData, B::OutputData, L::Output>,
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
    Links::SelectItems:
        Send + ExpressionsForOperation<ScopedAliased: for<'q> ManyExpressions<'q, S>>,
    // Links::SelectItems: Send + Aliased<Aliased: for<'q> ManyExpressions<'q, S>>,
    Links::SelectItems: for<'r> FromRowAlias<'r, S::Row, RData: Send>,
    Links::Join: for<'q> ManyExpressions<'q, S>,
    Links::Op: Operation<S>,
    Links::OpInput: Send,
    Base: Collection<OutputData: Send, Id: Send>,
    Base: ExpressionsForOperation<ScopedAliased: for<'q> ManyExpressions<'q, S>>,
    // Base: Aliased<Aliased: for<'q> ManyExpressions<'q, S>>,
    Base: FromRowData<RData = Base::OutputData>,
    Base: for<'r> FromRowAlias<'r, S::Row>,
    Base::Id: FromRowData<RData = <Base::Id as CollectionId>::IdData>,
    Base::Id: for<'r> FromRowAlias<'r, S::Row>,
    Base::Id: CollectionId<IdData: Send + for<'q> Encode<'q, S> + Type<S>>,
    Base::Id: ExpressionsForOperation<
            ScopedAliased: for<'q> ManyExpressions<'q, S>,
            Scoped: for<'q> ManyExpressions<'q, S>,
        >,
    // Base::Id: Scoped<Scoped: for<'q> Expression<'q, S>>,
    // Base::Id: Aliased<Aliased: for<'q> Expression<'q, S>>,
    Links: LinkFetch<Output: Send>,
    i64: for<'q> Encode<'q, S> + Type<S>,
    OrderBy: Send + Clone,
    // OrderBy: Scoped<Scoped: for<'q> ManyExpressions<'q, S>>,
    OrderBy: ExpressionsForOperation<Scoped: for<'q> ManyExpressions<'q, S>>,
    OrderBy: for<'r> FromRowAlias<'r, S::Row, RData: Send>,
    First: Send,
    First: SelfPrescribedInsert<
            InsertValue: Send + for<'q> ManyExpressions<'q, S>,
            InsertId: Send + for<'q> ManyExpressions<'q, S>,
        >,
{
    async fn exec_operation(self, pool: &mut S::Connection) -> Self::Output {
        // let db = S::singleton();
        let id = self.base.id();
        let link_items = self.links.non_aggregating_select_items();
        let query_builder = StatementBuilder::<S>::new(SelectStatement {
            select_items: ManyFlat((
                id.scoped_aliased("i"),
                self.base.scoped_aliased("b"),
                link_items.scoped_aliased("l"),
            )),
            from: self.base.table_name().to_string(),
            joins: self.links.non_duplicating_join_expressions(),
            group_by: (),
            order: self.cursor_order_by.scoped(),
            wheres: ManyFlat((
                self.wheres,
                self.links.where_expressions(),
                self.cursor_first_item
                    .map(|(id, first)| {
                        let (idents, values) = first.on_insert();
                        // let idents = first.scoped();
                        // let first = first.on_insert(());
                        ManyColumnsLargerOrEqual {
                            ids: ManyFlat((idents, self.base.id().scoped())),
                            values: ManyFlat((values, Bind(id))),
                        }
                    })
                    .unwrap(),
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
        operations::{
            LinkedOutput, Operation,
            fetch_many::{FetchMany, ManyOutput},
            operations_expressions_crossover::NamedBind,
        },
        test_module::{Todo, TodoHandler, todo_members},
    };

    #[tokio::test]
    async fn main() {
        let mut conn = Sqlite::in_memory_connection().await;

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
                base: TodoHandler,
                wheres: (),
                links: (),
                cursor_order_by: todo_members::title,
                // cursor_first_item: None::<(i64, ())>,
                cursor_first_item: Some((4, todo_members::title::bind(String::from("non_unique")))),
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
