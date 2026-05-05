use std::marker::PhantomData;

use crate::{
    collections::{Collection, CollectionId},
    database_extention::DatabaseExt,
    execute::Executable,
    extentions::common_expressions::{Identifier, OnInsert},
    fix_executor::ExecutorTrait,
    from_row::{FromRowAlias, FromRowData},
    operations::{LinkedOutput, Operation, OperationOutput},
    query_builder::{Expression, ManyExpressions, StatementBuilder, functional_expr::ManyFlat},
    statements::insert_statement::{InsertStatement, One},
};

pub trait LinkDataSplit {
    type Link: InsertLink;
    fn consume_data_only_once(
        self,
    ) -> (
        Self::Link,
        InsertLinkData<
            <Self::Link as InsertLink>::PreOpInput,
            <Self::Link as InsertLink>::OnInsertInput,
            <Self::Link as InsertLink>::PostOpInput,
        >,
    );
}

pub struct InsertId<Base, Id> {
    base: Base,
    id: Id,
}

impl<Base, Id> LinkDataSplit for InsertId<Base, Id> {
    type Link = InsertId<Base, PhantomData<Id>>;
    fn consume_data_only_once(self) -> (Self::Link, InsertLinkData<(), Id, ()>) {
        (
            InsertId {
                base: self.base,
                id: PhantomData,
            },
            InsertLinkData {
                pre_op_input: (),
                insert_items_input: self.id,
                post_op_input: (),
            },
        )
    }
}

impl<Base, Id> InsertLink for InsertId<Base, PhantomData<Id>> {
    type PreOp = ();

    type PreOpInput = ();

    fn pre_operation(&self, _: Self::PreOpInput) -> Self::PreOp {}

    type OnInsertIdent = ();

    type OnInsertReturning = ();

    fn on_insert_ident(&self) -> Self::OnInsertIdent {}

    fn on_insert_returning(&self) -> Self::OnInsertReturning {}

    type OnInsertExpr = ();

    type OnInsertInput = Id;

    fn on_insert(
        &self,
        input: Id,
        _: <Self::PreOp as OperationOutput>::Output,
    ) -> Self::OnInsertExpr
    where
        Self::PreOp: OperationOutput,
    {
        let _consume_id_only_once = input;
        todo!()
    }

    type FromRow = ();

    fn from_row(&self) -> Self::FromRow {}

    type FromRowPostOpInput = ();

    type FromRowTakeInput = ();

    fn insert_once(
        &self,
        _: <Self::FromRow as FromRowData>::RData,
    ) -> (Self::FromRowPostOpInput, Self::FromRowTakeInput)
    where
        Self::FromRow: FromRowData,
    {
        ((), ())
    }

    fn insert_many(
        &self,
        _: <Self::FromRow as FromRowData>::RData,
        _: &mut Self::InsertManyPostOpOutput,
        _: &mut Self::InsertManyTakeOutput,
    ) -> Self::FromRowTakeInput
    where
        Self::FromRow: FromRowData,
    {
        todo!()
    }

    type PostOpMany: OperationOutput<Output = <Self::PostOp as OperationOutput>::Output>;
    fn post_op_many(&self, _: Self::PostOpInput, _: Self::FromRowPostOpInput) -> Self::PostOpMany
    where
        Self::FromRow: FromRowData,
    {
        todo!()
    }

    type PostOp = ();

    type PostOpInput = ();

    fn post_operation(&self, _: Self::PostOpInput, _: Self::FromRowPostOpInput) -> Self::PostOp {}

    type Output = ();

    fn take(
        self,
        _: <Self::PostOp as OperationOutput>::Output,
        _: Self::FromRowTakeInput,
    ) -> Self::Output {
    }

    fn take_many(
        self,
        _: &mut <Self::PostOp as OperationOutput>::Output,
        _: Self::FromRowTakeInput,
    ) -> Self::Output {
    }
}

pub struct InsertLinkData<PreOpInput, InsertItemsInput, PostOpInput> {
    pub pre_op_input: PreOpInput,
    pub insert_items_input: InsertItemsInput,
    pub post_op_input: PostOpInput,
}

pub trait InsertLink {
    type PreOp: OperationOutput;
    type PreOpInput;
    fn pre_operation(&self, input: Self::PreOpInput) -> Self::PreOp;

    type OnInsertIdent;
    type OnInsertReturning;
    fn on_insert_ident(&self) -> Self::OnInsertIdent;
    fn on_insert_returning(&self) -> Self::OnInsertReturning;

    type OnInsertExpr;
    type OnInsertInput;
    fn on_insert(
        &self,
        input: Self::OnInsertInput,
        pre_op: <Self::PreOp as OperationOutput>::Output,
    ) -> Self::OnInsertExpr
    where
        Self::PreOp: OperationOutput;

    type FromRow: FromRowData;
    fn from_row(&self) -> Self::FromRow;

    type FromRowPostOpInput;
    type FromRowTakeInput;
    fn insert_once(
        &self,
        from_row: <Self::FromRow as FromRowData>::RData,
    ) -> (Self::FromRowPostOpInput, Self::FromRowTakeInput)
    where
        Self::FromRow: FromRowData;

    type PostOp: OperationOutput;
    type PostOpInput;
    fn post_operation(
        &self,
        link_input: Self::PostOpInput,
        from_row_input: Self::FromRowPostOpInput,
    ) -> Self::PostOp;

    type Output;
    fn take(
        self,
        pre_op: <Self::PostOp as OperationOutput>::Output,
        insert_items: Self::FromRowTakeInput,
    ) -> Self::Output;
}

impl LinkDataSplit for () {
    type Link = ();
    fn consume_data_only_once(self) -> (Self::Link, InsertLinkData<(), (), ()>) {
        (
            (),
            InsertLinkData {
                pre_op_input: (),
                insert_items_input: (),
                post_op_input: (),
            },
        )
    }
}

impl InsertLink for () {
    type PreOp = ();
    type PreOpInput = ();
    fn pre_operation(&self, _: Self::PreOpInput) -> Self::PreOp {}

    type OnInsertExpr = ();
    type OnInsertInput = ();
    fn on_insert(&self, _: (), _: ()) -> Self::OnInsertExpr
    where
        Self::PreOp: OperationOutput,
    {
    }

    type OnInsertIdent = ();
    type OnInsertReturning = ();
    fn on_insert_ident(&self) -> Self::OnInsertIdent {}
    fn on_insert_returning(&self) -> Self::OnInsertReturning {}

    type PostOp = ();
    type PostOpInput = ();
    fn post_operation(&self, _: (), _: ()) -> Self::PostOp
    where
        Self::OnInsertExpr: FromRowData,
    {
    }

    type Output = ();

    fn take(
        self,
        _: <Self::PostOp as OperationOutput>::Output,
        _: <Self::OnInsertExpr as FromRowData>::RData,
    ) -> Self::Output
    where
        Self::PostOp: OperationOutput,
        Self::OnInsertExpr: FromRowData,
    {
    }

    type FromRow = ();

    fn from_row(&self) -> Self::FromRow {}

    type FromRowPostOpInput = ();

    type FromRowTakeInput = ();

    fn insert_once(&self, _: ()) -> (Self::FromRowPostOpInput, Self::FromRowTakeInput)
    where
        Self::FromRow: FromRowData,
    {
        ((), ())
    }
}

pub struct InsertOne<Handler, Data, Links> {
    pub base: Handler,
    pub data: Data,
    pub links: Links,
}

impl<H, L, D> OperationOutput for InsertOne<H, H::Data, (L, D)>
where
    L: InsertLink,
    H: Collection,
{
    type Output = LinkedOutput<<H::Id as CollectionId>::IdData, H::Data, L::Output>;
}

impl<S, Base, Link, PreOpInput, InsertItemsInput, PostOpInput> Operation<S>
    for InsertOne<
        Base,
        Base::Data,
        (
            Link,
            InsertLinkData<PreOpInput, InsertItemsInput, PostOpInput>,
        ),
    >
where
    S: DatabaseExt,
    S: ExecutorTrait,
    Link: Send
        + InsertLink<
            PreOpInput = PreOpInput,
            PreOpInput: Send,
            PostOpInput = PostOpInput,
            PostOpInput: Send,
            OnInsertInput = InsertItemsInput,
            OnInsertInput: Send,
        >,
    Base: Collection<Data: Send, Id: Send + CollectionId<IdData: Send>>,
    Base: Send,
    Base: Clone,
    Base: Identifier<Identifier: for<'q> ManyExpressions<'q, S>>,
    Base: OnInsert<InsertInput = Base::Data, InsertExpression: for<'q> ManyExpressions<'q, S>>,
    Base: for<'r> FromRowAlias<'r, S::Row, RData = Base::Data>,
    Base::Id: Identifier<Identifier: for<'q> Expression<'q, S>>,
    Base::Id: for<'r> FromRowAlias<'r, S::Row, RData = <Base::Id as CollectionId>::IdData>,
    Link::PreOp: Operation<S>,
    Link::PostOp: Operation<S>,
    Link::OnInsertExpr: Send + for<'q> ManyExpressions<'q, S>,
    Link::OnInsertIdent: Send + for<'q> ManyExpressions<'q, S>,
    Link::OnInsertReturning: Send + for<'q> ManyExpressions<'q, S>,
    Link::FromRow: for<'r> FromRowAlias<'r, S::Row, RData: Send>,
    Link::FromRowTakeInput: Send,
    Link::Output: Send,
{
    fn exec_operation(self, pool: &mut S::Connection) -> impl Future<Output = Self::Output> + Send
    where
        S: sqlx::Database,
        Self: Sized,
    {
        async move {
            let pre_op = self
                .links
                .0
                .pre_operation(self.links.1.pre_op_input)
                .exec_operation(&mut *pool)
                .await;

            let (stmt, arg) = StatementBuilder::<'_, S>::new(InsertStatement {
                table_name: self.base.table_name(),
                identifiers: ManyFlat((self.base.identifier(), self.links.0.on_insert_ident())),
                returning: ManyFlat((
                    self.base.id().identifier(),
                    self.base.identifier(),
                    self.links.0.on_insert_returning(),
                )),
                values: One(ManyFlat((
                    self.base.clone().on_insert(self.data),
                    self.links
                        .0
                        .on_insert(self.links.1.insert_items_input, pre_op),
                ))),
            })
            .unwrap();

            let row = S::fetch_optional(
                &mut *pool,
                Executable {
                    string: &stmt,
                    arguments: arg,
                },
            )
            .await
            .unwrap()
            .unwrap();

            let id = self.base.id().no_alias(&row).unwrap();
            let attributes = self.base.no_alias(&row).unwrap();

            let links = {
                let ii = self.links.0.from_row().no_alias(&row).unwrap();
                let (post_op_input_2, from_row_take_input) = self.links.0.insert_once(ii);
                let po = self
                    .links
                    .0
                    .post_operation(self.links.1.post_op_input, post_op_input_2)
                    .exec_operation(&mut *pool)
                    .await;
                self.links.0.take(po, from_row_take_input)
            };

            LinkedOutput {
                id,
                attributes,
                links,
            }
        }
    }
}

#[cfg(test)]
mod test {
    use crate::{
        connect_in_memory::ConnectInMemory,
        operations::{LinkedOutput, Operation, insert_one::InsertOne},
        test_module::{self, Todo},
    };
    use sqlx::{Sqlite, query};

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

    ",
        )
        .execute(&mut conn)
        .await
        .unwrap();

        let output = Operation::<Sqlite>::exec_operation(
            InsertOne {
                data: Todo {
                    title: String::from("todo"),
                    done: false,
                    description: None,
                },
                base: test_module::todo,
                links: (),
            },
            &mut conn,
        )
        .await;

        pretty_assertions::assert_eq!(
            output,
            LinkedOutput {
                id: 1,
                attributes: Todo {
                    title: String::from("todo"),
                    done: false,
                    description: None,
                },
                links: ()
            }
        );
    }
}
