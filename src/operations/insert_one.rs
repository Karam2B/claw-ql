use crate::{
    collections::{Collection, CollectionId, CreateIdFor},
    database_extention::DatabaseExt,
    execute::Executable,
    extentions::common_expressions::{Identifier, OnInsert, TableNameExpression},
    fix_executor::ExecutorTrait,
    from_row::{FromRowAlias, FromRowData},
    operations::{LinkedOutput, Operation, OperationOutput},
    query_builder::{Expression, ManyExpressions, StatementBuilder, functional_expr::ManyFlat},
    statements::insert_statement::{InsertStatement, One},
};

pub trait InsertLinkConsumeData {
    type Link: InsertOneLink;
    fn consume_data(
        self,
    ) -> (
        Self::Link,
        InsertLinkData<
            <Self::Link as InsertOneLink>::PreOpData,
            <Self::Link as InsertOneLink>::InsertValuesData,
            <Self::Link as InsertOneLink>::PostOpData,
        >,
    );
}

pub struct InsertLinkData<PreOpData, InsertValueData, PostOpData> {
    pub insert_value_data: InsertValueData,
    pub pre_op_data: PreOpData,
    pub post_op_data: PostOpData,
}

pub trait InsertOneLink {
    type PreOp: OperationOutput;
    type PreOpData;
    fn pre_operation_init(&self, input: Self::PreOpData) -> Self::PreOp;

    fn pre_op_split(
        &self,
        pre_op_output: <Self::PreOp as OperationOutput>::Output,
    ) -> Result<
        (
            Self::PreOpToInsertValue,
            Self::PreOpToTake,
            Self::PreOpToPostOp,
        ),
        ConstraintViolation,
    >;
    type PreOpToInsertValue;
    type PreOpToTake;
    type PreOpToPostOp;

    type InsertNames;
    fn insert_names(&self) -> Self::InsertNames;

    type InsertReturning;
    fn insert_returning(&self) -> Self::InsertReturning;

    type InsertValuesData;
    type InsertValues;
    fn insert_value(
        &self,
        from_data: Self::InsertValuesData,
        pre_op_output: Self::PreOpToInsertValue,
    ) -> Self::InsertValues;

    type FromRow: FromRowData;
    fn from_row(&self) -> Self::FromRow;

    type TakeInput;
    type PostOp: OperationOutput;
    type PostOpData;
    fn from_row_result(
        &self,
        from_data: Self::PostOpData,
        from_row: <Self::FromRow as FromRowData>::RData,
        pre_op_to_post_op: Self::PreOpToPostOp,
    ) -> (Self::PostOp, Self::TakeInput);

    type PostOpOutput;
    fn post_op_output(&self,
        poo: <Self::PostOp as OperationOutput>::Output,
    ) -> Result<Self::PostOpOutput, ConstraintViolation> ;

    type Output;
    fn take(
        self,
        post_op_output: Self::PostOpOutput,
        insert_items: Self::TakeInput,
        pre_op_to_post_op: Self::PreOpToTake,
    ) -> Self::Output;
}

impl InsertLinkConsumeData for () {
    type Link = ();
    fn consume_data(
        self,
    ) -> (
        Self::Link,
        InsertLinkData<
            <Self::Link as InsertOneLink>::PreOpData,
            <Self::Link as InsertOneLink>::InsertValuesData,
            <Self::Link as InsertOneLink>::PostOpData,
        >,
    ) {
        (
            (),
            InsertLinkData {
                insert_value_data: (),
                pre_op_data: (),
                post_op_data: (),
            },
        )
    }
}

impl InsertOneLink for () {
    type PreOp = ();

    type PreOpData = ();

    fn pre_operation_init(&self, _: Self::PreOpData) -> Self::PreOp {}

    fn pre_op_split(
        &self,
        _: <Self::PreOp as OperationOutput>::Output,
    ) -> Result<
        (
            Self::PreOpToInsertValue,
            Self::PreOpToTake,
            Self::PreOpToPostOp,
        ),
        ConstraintViolation,
    > {
        Ok(((), (), ()))
    }

    type PreOpToInsertValue = ();
    type PreOpToTake = ();
    type PreOpToPostOp = ();

    type InsertNames = ();

    fn insert_names(&self) -> Self::InsertNames {}

    type InsertReturning = ();

    fn insert_returning(&self) -> Self::InsertReturning {}

    type InsertValuesData = ();

    type InsertValues = ();

    fn insert_value(
        &self,
        _: Self::InsertValuesData,
        _: <Self::PreOp as OperationOutput>::Output,
    ) -> Self::InsertValues {
    }

    type FromRow = ();

    fn from_row(&self) -> Self::FromRow {}

    type TakeInput = ();

    type PostOp = ();

    type PostOpData = ();

    fn from_row_result(
        &self,
        _: Self::PostOpData,
        _: <Self::FromRow as FromRowData>::RData,
        _: Self::PreOpToTake,
    ) -> (Self::PostOp, Self::TakeInput) {
        ((), ())
    }

    type PostOpOutput = ();
    fn post_op_output(&self
    ,_: <Self::PostOp as OperationOutput>::Output,
    ) -> Result<Self::PostOpOutput, ConstraintViolation> {
        Ok(())
    }

    type Output = ();

    fn take(
        self,
        _: Self::PostOpOutput,
        _: Self::TakeInput,
        _: Self::PreOpToTake,
    ) -> Self::Output {
    }
}

pub struct InsertOne<Id, Handler, Data, Links> {
    pub base: Handler,
    pub id: Id,
    pub data: Data,
    pub links: Links,
}

#[derive(Debug, PartialEq, Eq, Clone)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct ConstraintViolation(pub Option<String>);

impl From<()> for ConstraintViolation {
    fn from(_: ()) -> Self {
        ConstraintViolation(None)
    }
}

impl<I, H, PreL, L> OperationOutput for InsertOne<I, H, H::InputData, PreL>
where
    PreL: InsertLinkConsumeData<Link = L>,
    L: InsertOneLink,
    H: Collection,
{
    type Output = Result<
        LinkedOutput<<H::Id as CollectionId>::IdData, H::OutputData, L::Output>,
        ConstraintViolation,
    >;
}

impl<Id, S, Base, LinkPreSplit, Link> Operation<S> for InsertOne<Id, Base, Base::InputData, LinkPreSplit>
where
    S: DatabaseExt,
    S: ExecutorTrait,
    Id: CreateIdFor<Base::Id, Result: Send>,
    Id::Result: for<'q> ManyExpressions<'q, S>,
    Id: Send,
    LinkPreSplit: Send + InsertLinkConsumeData<Link = Link>,
    Link: Send,
    Link: InsertOneLink,
    Base: TableNameExpression<TableNameExpression: for<'q> Expression<'q, S>>,
    Base: Collection<InputData: Send, OutputData: Send, Id: Send + CollectionId<IdData: Send>>,
    Base: Send,
    Base: Identifier<Identifier: for<'q> ManyExpressions<'q, S>>,
    Base: OnInsert<InsertInput = Base::InputData, InsertExpression: for<'q> ManyExpressions<'q, S>>,
    Base: for<'r> FromRowAlias<'r, S::Row, RData = Base::OutputData>,
    Base::Id: Identifier<Identifier: for<'q> ManyExpressions<'q, S>>,
    Base::Id: for<'r> FromRowAlias<'r, S::Row, RData = <Base::Id as CollectionId>::IdData>,
    Link::PreOp: Operation<S, Output: Send>,
    Link::PreOpData: Send,
    Link::InsertNames: for<'q> ManyExpressions<'q, S>,
    Link::InsertValues: for<'q> ManyExpressions<'q, S>,
    Link::InsertValuesData: Send,
    Link::PostOpData: Send,
    Link::InsertReturning: for<'q> ManyExpressions<'q, S>,
    Link::FromRow: for<'r> FromRowAlias<'r, S::Row, RData: Send>,
    Link::TakeInput: Send,
    Link::PostOp: Operation<S, Output: Send>,
    Link::Output: Send,
    Link::PreOpToInsertValue: Send,
    Link::PreOpToTake: Send,
    Link::PreOpToPostOp: Send,
{
    fn exec_operation(self, pool: &mut S::Connection) -> impl Future<Output = Self::Output> + Send
    where
        S: sqlx::Database,
        Self: Sized,
    {
        async move {
            let (link, link_data) = self.links.consume_data();
            let pre_op = link
                .pre_operation_init(link_data.pre_op_data)
                .exec_operation(&mut *pool)
                .await;

            let (pre_op_to_insert_value, pre_op_to_take, pre_op_to_post_op) =
                link.pre_op_split(pre_op)?;

            let base_id = self.base.id();

            let id_insert = self.id.create_id(&base_id);

            let (stmt, arg) = StatementBuilder::<'_, S>::new(InsertStatement {
                table_name: self.base.table_name_expression(),
                identifiers: ManyFlat((
                    if id_insert.is_some() {
                        Some(base_id.identifier())
                    } else {
                        None
                    },
                    self.base.identifier(),
                    link.insert_names(),
                )),
                values: One(ManyFlat((
                    id_insert,
                    self.base.on_insert(self.data),
                    link.insert_value(link_data.insert_value_data, pre_op_to_insert_value),
                ))),
                returning: ManyFlat((
                    base_id.identifier(),
                    self.base.identifier(),
                    link.insert_returning(),
                )),
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
            .map_err(|e| {
                if let Some(e) = e.as_database_error() {
                    if e.is_check_violation() || e.is_unique_violation() || e.is_foreign_key_violation() {
                        return ConstraintViolation(e.constraint().map(|c| c.to_string()));
                    }
                    
                } 
                    tracing::error!(sqlx_error = ?e, "bug: must clear all sqlx errors, hard to know where this error was originated!");
                    panic!()
            })?
            .unwrap();

            let id = base_id.no_alias(&row).unwrap();
            let attributes = self.base.no_alias(&row).unwrap();

            let links = {
                let ii = link.from_row().no_alias(&row).unwrap();
                let (post_op_input_2, from_row_take_input) =
                    link.from_row_result(link_data.post_op_data, ii, pre_op_to_post_op);
                let po = post_op_input_2.exec_operation(&mut *pool).await;
                let po = link.post_op_output(po)?;
                link.take(po, from_row_take_input, pre_op_to_take)
            };

            Ok(LinkedOutput {
                id,
                attributes,
                links,
            })
        }
    }
}

#[cfg(test)]
mod test {
    use crate::{
        collections::AutoGenerate,
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
                id: AutoGenerate,
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
            Ok(LinkedOutput {
                id: 1,
                attributes: Todo {
                    title: String::from("todo"),
                    done: false,
                    description: None,
                },
                links: ()
            })
        );
    }
}
