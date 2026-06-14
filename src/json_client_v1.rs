// pub use crate::json_client_v0::dynamic_collection;
pub use crate::json_client_v0::sqlx_type_ident;
pub use crate::json_client_v0::to_bind_trait;

pub mod http_client_error {
    use serde::Serialize;

    #[derive(Clone, Debug)]
    pub struct HttpClientError {
        pub status: hyper::StatusCode,
        pub payload: serde_json::Value,
    }

    impl<T: Serialize> From<T> for HttpClientError {
        fn from(value: T) -> Self {
            Self {
                status: hyper::StatusCode::BAD_REQUEST,
                payload: serde_json::to_value(value).expect("bug: when serialization ever fail?"),
            }
        }
    }

    struct _TodoOverNetworkError {
        // derived from the error's type
        // need to have a logic to prevent different errors to have same id
        pub globally_unique_code: &'static str,
        pub status_code: hyper::StatusCode,

        // derived from error's type
        // information to build frontend_client
        pub problematic_span: (),
        pub non_problematic_but_causing_span: Vec<()>,

        // derived from error's instance
        // to be sent to client,
        // but shouldn't be displayed,
        // it's the fronted end responsiblity to build a UI that never allows the user to send possible 4xx requests
        // can be used for debuging purposes
        pub public_debug_info: (),

        // private debug info
        // all errors should be handled in two ways
        //      1. in backend (5xx, panics, fix bugs in the backend)
        //      2. in frontend (4xx, use better clients in order to never send a request that will result in 4xx)
        // but if either fail this would be the fall back
        pub private_info: (),    // to be saved in backend
        pub private_info_id: (), // to be sent and displayed on the frontend, it's a way for users to share their issue with developers
    }
}

pub mod dynamic_collection {
    use core::fmt;

    use crate::{
        database_extention::DatabaseExt,
        json_client_v0::{dynamic_collection::DynamicField, sqlx_type_ident::SqlxTypeHandler},
    };

    pub struct DynamicCollection<S> {
        pub name: String,
        pub name_lower_case: String,
        pub fields: Vec<DynamicField<Box<dyn SqlxTypeHandler<S> + Send + Sync>>>,
    }

    impl<S> fmt::Debug for DynamicCollection<S>
    where
        S: DatabaseExt,
    {
        fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
            write!(
                f,
                "DynamicCollection {{ name: {}, fields: {:?}, db: {} }}",
                self.name,
                self.fields,
                S::NAME
            )
        }
    }
    impl<S> Clone for DynamicCollection<S>
    where
        S: DatabaseExt,
    {
        fn clone(&self) -> Self {
            Self {
                name: self.name.clone(),
                name_lower_case: self.name_lower_case.clone(),
                fields: self
                    .fields
                    .iter()
                    .map(|e| DynamicField {
                        name: e.name.clone(),
                        is_optional: e.is_optional,
                        type_info: e.type_info.clone_self(),
                    })
                    .collect(),
            }
        }
    }

    mod impl_on_migrate {
        use super::*;
        use crate::{
            json_client_v0::dynamic_collection as og_dynamic_collection, on_migrate::OnMigrate,
        };

        impl<S> OnMigrate for DynamicCollection<S>
        where
            S: DatabaseExt,
        {
            type Statements = og_dynamic_collection::impl_on_migrate::MigrateDynamicCollection<S>;
            fn statments(&self) -> Self::Statements {
                og_dynamic_collection::impl_on_migrate::MigrateDynamicCollection {
                    name: self.name.clone(),
                    fields: self
                        .fields
                        .iter()
                        .map(|e| DynamicField {
                            name: e.name.clone(),
                            is_optional: e.is_optional,
                            type_info: e.type_info.type_expression(),
                        })
                        .collect(),
                }
            }
        }
    }

    mod collection_impls {
        use crate::{
            collections::{Collection, SingleIncremintalInt},
            json_client_v1::partial_serde::PartialSerializeV1,
        };

        use super::*;

        impl<S> Collection for DynamicCollection<S>
        where
            S: DatabaseExt,
        {
            fn table_name(&self) -> &str {
                &self.name
            }

            fn table_name_lower_case(&self) -> &str {
                &self.name_lower_case
            }

            type InputData = PartialSerializeV1;
            type UpdateData = PartialSerializeV1;
            type OutputData = PartialSerializeV1;

            type Id = SingleIncremintalInt<String>;

            fn id(&self) -> Self::Id {
                SingleIncremintalInt(self.name.clone())
            }
        }
    }

    mod impl_aliased {
        use crate::{
            database_extention::DatabaseExt, extentions::common_expressions::Aliased,
            json_client_v0::dynamic_collection::str_aliased_impls::DynamicAliasedCols,
            json_client_v1::dynamic_collection::DynamicCollection,
        };

        impl<S> Aliased for DynamicCollection<S>
        where
            S: DatabaseExt,
        {
            type Aliased = DynamicAliasedCols;
            fn aliased(&self, alias: &'static str) -> Self::Aliased {
                DynamicAliasedCols {
                    table: self.name.clone(),
                    cols: self.fields.iter().map(|e| e.name.clone()).collect(),
                    num: None,
                    alias,
                }
            }

            type NumAliased = DynamicAliasedCols;

            fn num_aliased(&self, num: usize, alias: &'static str) -> Self::NumAliased {
                DynamicAliasedCols {
                    table: self.name.clone(),
                    cols: self.fields.iter().map(|e| e.name.clone()).collect(),
                    num: Some(num),
                    alias,
                }
            }
        }
    }

    mod expression_table_name {
        use crate::{
            extentions::common_expressions::TableNameExpression,
            json_client_v1::dynamic_collection::DynamicCollection,
        };

        impl<S> TableNameExpression for DynamicCollection<S> {
            type TableNameExpression = String;
            fn table_name_expression(&self) -> Self::TableNameExpression {
                self.name.clone()
            }
            type LowerCaseTableNameExpression = String;
            fn lower_case_table_name_expression(&self) -> Self::LowerCaseTableNameExpression {
                self.name_lower_case.clone()
            }
        }
    }

    mod from_row_impls {
        use std::collections::BTreeMap;

        use crate::{
            from_row::{FromRowAlias, FromRowData},
            json_client_v1::partial_serde::PartialSerializeV1,
        };

        use super::*;

        impl<S> FromRowData for DynamicCollection<S> {
            type RData = PartialSerializeV1;
        }

        impl<'r, S> FromRowAlias<'r, S::Row> for DynamicCollection<S>
        where
            S: DatabaseExt,
        {
            fn no_alias(
                &self,
                row: &'r S::Row,
            ) -> Result<Self::RData, crate::from_row::FromRowError> {
                let mut oj = BTreeMap::new();
                for field in self.fields.iter() {
                    let s =
                        field
                            .type_info
                            .from_row_no_alias(field.is_optional, &field.name, row)?;
                    oj.insert(field.name.to_string(), s);
                }
                Ok(PartialSerializeV1::new(oj))
            }

            fn pre_alias(
                &self,
                row: crate::from_row::RowPreAliased<'r, S::Row>,
            ) -> Result<Self::RData, crate::from_row::FromRowError>
            where
                S::Row: sqlx::Row,
            {
                let mut oj = BTreeMap::new();
                for field in self.fields.iter() {
                    let s = field.type_info.from_row_pre_alias(
                        field.is_optional,
                        &field.name,
                        row.clone(),
                    )?;
                    oj.insert(field.name.to_string(), s);
                }
                Ok(PartialSerializeV1::new(oj))
            }

            fn post_alias(
                &self,
                _: crate::from_row::RowPostAliased<'r, S::Row>,
            ) -> Result<Self::RData, crate::from_row::FromRowError>
            where
                S::Row: sqlx::Row,
            {
                panic!("in the process of deprecating this method");
            }

            fn two_alias(
                &self,
                row: crate::from_row::RowTwoAliased<'r, S::Row>,
            ) -> Result<Self::RData, crate::from_row::FromRowError>
            where
                S::Row: sqlx::Row,
            {
                let mut oj = serde_json::Map::new();
                for field in self.fields.iter() {
                    let s = field.type_info.from_row_two_alias(
                        field.is_optional,
                        &field.name,
                        row.clone(),
                    )?;
                    oj.insert(field.name.to_string(), s);
                }
                Ok(PartialSerializeV1::new(oj))
            }
        }
    }
}

pub mod sqlx_executor {
    use super::json_client::JsonClientSetting;
    use super::json_client::Operation;
    use super::json_client::OperationOutput;
    use crate::database_extention::DatabaseExt;
    use crate::fix_executor::ExecutorTrait;
    use crate::json_client_v1::dynamic_collection::DynamicCollection;

    use crate::json_client_v0::fetch_many::extending_link_trait::JsonLinkFetchMany;
    use crate::json_client_v1::add_collection::add_collection;
    use crate::json_client_v1::add_link::add_link;
    use crate::json_client_v1::fetch_many::fetch_many;
    use crate::json_client_v1::http_client_error::HttpClientError;
    use crate::json_client_v1::insert_one::insert_one;
    use crate::json_client_v1::json_client::JsonClient;
    use crate::links::DefaultRelationKey;
    use crate::links::relation_optional_to_many::OptionalToMany;
    use crate::on_migrate::OnMigrate;
    use crate::query_builder::Expression;
    use futures::future::Future;
    use oneshot;
    use sqlx::Database;
    use sqlx::IntoArguments;
    use sqlx::Pool;
    use std::collections::HashMap;
    use std::convert::Infallible;
    use std::sync::Arc;
    use tokio::sync::RwLock as Trw;
    use tokio::sync::mpsc as tokio_mpsc;

    pub type LinkInformations = crate::json_client_v0::json_client::LinkInformations;

    pub struct ClientData<S: Database> {
        pub(crate) collections: Trw<HashMap<String, Trw<DynamicCollection<S>>>>,
        pub(crate) migration: Trw<Vec<String>>,
        pub(crate) link_info: Trw<LinkInformations>,
        #[allow(dead_code)]
        pub(crate) setting: JsonClientSetting,
        pub(crate) pool: Pool<S>,
    }

    pub struct SqlxExecutor<S: Database> {
        reciever: tokio_mpsc::UnboundedReceiver<(
            Operation,
            oneshot::Sender<Result<OperationOutput, HttpClientError>>,
        )>,
        data: Arc<ClientData<S>>,
    }

    impl JsonClient {
        pub fn new_sqlx_db<S: Database>(
            pool: Pool<S>,
            setting: JsonClientSetting,
        ) -> (Self, SqlxExecutor<S>) {
            let (sender, reciever) = tokio_mpsc::unbounded_channel::<(
                Operation,
                oneshot::Sender<Result<OperationOutput, HttpClientError>>,
            )>();

            let data = Arc::new({
                ClientData {
                    collections: Trw::new(Default::default()),
                    link_info: Trw::new(Default::default()),
                    migration: Trw::new(Default::default()),
                    setting: setting,
                    pool,
                }
            });

            (JsonClient { sender }, SqlxExecutor { reciever, data })
        }
    }

    fn drop_collection<S: Database>(
        _: Arc<ClientData<S>>,
        _: &'static str,
    ) -> impl Future<Output = Result<&'static str, HttpClientError>> + 'static + Send + use<S> {
        async move { Ok("test success") }
    }

    impl<S> SqlxExecutor<S>
    where
        S: DatabaseExt,
        bool: for<'d> sqlx::Decode<'d, S> + sqlx::Type<S> + for<'q> sqlx::Encode<'q, S>,
        std::string::String:
            sqlx::Type<S> + for<'q> sqlx::Encode<'q, S> + for<'d> sqlx::Decode<'d, S>,
        for<'a> &'a str: sqlx::ColumnIndex<<S as sqlx::Database>::Row>,
        DynamicCollection<S>: OnMigrate<Statements: Expression<'static, S>>,
        for<'a> S::Arguments<'a>: IntoArguments<'a, S>,
        S: ExecutorTrait,
        OptionalToMany<DefaultRelationKey, DynamicCollection<S>, DynamicCollection<S>>:
            JsonLinkFetchMany<S>,
        for<'q> &'q str: sqlx::ColumnIndex<<S as sqlx::Database>::Row>,
        i64: for<'q> sqlx::Encode<'q, S> + for<'q> sqlx::Decode<'q, S> + sqlx::Type<S>,
    {
        pub fn run(mut self) -> impl Future<Output = Infallible> {
            async move {
                loop {
                    let operation = self.reciever.recv().await.unwrap();

                    macro_rules! macr {
                        ($([$operation:ident, $fn:ident])*) => {
                            match operation.0 {
                                $(
                                    Operation::$operation(input) => {
                                        let s = $fn(self.data.clone(), input);
                                        tokio::spawn(async move {
                                            let s = tokio::spawn(s).await;
                                            match s {
                                                Ok(output) => operation
                                                    .1
                                                    .send(output.map(|e| OperationOutput::$operation(e)))
                                                    .unwrap(),
                                                Err(e) => match e.try_into_panic() {
                                                    Ok(s) => {
                                                        let _msg = match s
                                                            .downcast::<String>()
                                                            .map_err(|e| e.downcast::<&str>())
                                                        {
                                                            Ok(s) => *s,
                                                            Err(Ok(s)) => s.to_string(),
                                                            _ => String::from("unknown panic"),
                                                        };
                                                        operation
                                                            .1
                                                            .send(Err(HttpClientError {
                                                                status:
                                                                    hyper::StatusCode::INTERNAL_SERVER_ERROR,
                                                                payload: serde_json::Value::Null,
                                                            }))
                                                            .unwrap();
                                                    },
                                                    Err(_) => {panic!("uncoverd task join error case")},
                                                }
                                            };
                                        });
                                    }
                                )*,
                                #[allow(unused)]
                                _ => {
                                    todo!("unsupported operation")
                                }
                            }
                        };
                    }

                    macr!(
                        [AddCollection, add_collection]
                        [AddLink, add_link]
                        [FetchMany, fetch_many]
                        [InsertOne, insert_one]
                        [DropCollection, drop_collection]
                    );
                }
            }
        }
    }
}

pub mod json_client {
    use serde::{Deserialize, Serialize};
    use std::collections::BTreeMap;
    use std::convert::Infallible;
    use tokio::sync::mpsc as tokio_mpsc;

    use crate::json_client_v0 as old_mod;
    use crate::json_client_v0::fetch_many::SupportedLinkFetchMany;
    use crate::json_client_v1::partial_serde::{PartialDeserializeV1, PartialSerializeV1};
    use crate::json_client_v1::{
        add_collection::{AddCollectionInput, AddCollectionOutput},
        http_client_error::HttpClientError,
    };
    use crate::operations::fetch_many::ManyOutput;
    use crate::operations::{CollectionOutput, LinkedOutput};

    pub type AddLinkInput = old_mod::add_link::AddLinkInput;
    pub type AddLinkOutput = old_mod::add_link::AddLinkOutput;

    #[derive(Debug, Deserialize)]
    #[serde(deny_unknown_fields)]
    pub struct FetchManyInput {
        pub base: String,
        pub filters: Vec<crate::json_client_v1::supported_filter::SupportedFilter>,
        pub links: Vec<SupportedLinkFetchMany>,
        pub pagination: Pagination,
    }

    #[derive(Debug, Deserialize)]
    #[serde(deny_unknown_fields)]
    pub struct InsertOneInput {
        pub base: String,
        pub data: PartialDeserializeV1,
        pub links: Vec<PartialDeserializeV1>,
    }

    pub type InsertOneOutput = LinkedOutput<i64, PartialSerializeV1, Vec<PartialSerializeV1>>;

    #[derive(Debug, Deserialize)]
    pub struct Pagination {
        pub limit: i64,
        pub first_item: Option<CollectionOutput<i64, BTreeMap<String, PartialDeserializeV1>>>,
        pub order_by: Vec<PagniationOrderBy>,
    }

    #[derive(Debug, Deserialize, Clone)]
    pub struct PagniationOrderBy {
        pub col: String,
        pub direction: OrderDirection,
    }

    mod pagination_order_by_impls {
        use super::OrderDirection;
        use crate::database_extention::DatabaseExt;
        use crate::query_builder::Expression;
        use crate::query_builder::OpExpression;
        use crate::query_builder::StatementBuilder;

        impl OpExpression for super::PagniationOrderBy {}

        impl<'q, S> Expression<'q, S> for super::PagniationOrderBy
        where
            S: DatabaseExt,
        {
            fn expression(self, ctx: &mut StatementBuilder<'q, S>) {
                ctx.sanitize(self.col.as_str());
                ctx.syntax(" ");
                match self.direction {
                    OrderDirection::Asc => ctx.syntax("ASC"),
                    OrderDirection::Desc => ctx.syntax("DESC"),
                }
            }
        }
    }

    #[derive(Clone, Debug, Deserialize)]
    pub enum OrderDirection {
        #[serde(rename = "asc")]
        Asc,
        #[serde(rename = "desc")]
        Desc,
    }

    pub type FetchManyOutput = ManyOutput<
        LinkedOutput<i64, PartialSerializeV1, Vec<PartialSerializeV1>>,
        CollectionOutput<i64, BTreeMap<String, PartialSerializeV1>>,
    >;

    #[derive(Debug, Deserialize)]
    #[non_exhaustive]
    pub enum Operation {
        AddCollection(AddCollectionInput),
        AddLink(AddLinkInput),
        FetchMany(FetchManyInput),
        InsertOne(InsertOneInput),
        DropCollection(&'static str),
    }

    #[derive(Debug, Serialize)]
    #[non_exhaustive]
    pub enum OperationOutput {
        AddCollection(AddCollectionOutput),
        AddLink(AddLinkOutput),
        FetchMany(FetchManyOutput),
        InsertOne(InsertOneOutput),
        DropCollection(&'static str),
    }

    #[claw_ql_macros::skip]
    macro_rules! ops {
        ($([$method_name:ident, $op_name:ident, $input:ident, $output:ident])*) => {
            impl JsonClient {
                $(
                    pub fn $method_name(
                        &self,
                        input: $input,
                    ) -> impl Future<Output = Result<$output, HttpClientError>> + 'static + Send + use<>
                    {
                        let (sender, reciever) =
                            oneshot::async_channel::<Result<OperationOutput, HttpClientError>>();
                        self.sender
                            .send((Operation::$op_name(input), sender))
                            .unwrap();

                        async move {
                            return reciever
                                .await
                                .map_err(|_| HttpClientError {
                                    status: hyper::StatusCode::INTERNAL_SERVER_ERROR,
                                    payload: serde_json::to_value(format!("internal server error")).unwrap(),
                                })?
                                .map(|e| match e {
                                    OperationOutput::$op_name(e) => Ok(e),
                                    _ => Err(HttpClientError {
                                        status: hyper::StatusCode::INTERNAL_SERVER_ERROR,
                                        payload: serde_json::to_value(format!("internal server error")).unwrap(),
                                    }),
                                })?;
                        }
                    }
                )*
            }
        };
    }

    impl JsonClient {
        pub fn insert_one(
            &self,
            input: InsertOneInput,
        ) -> impl Future<Output = Result<InsertOneOutput, HttpClientError>> + 'static + Send + use<>
        {
            let (sender, reciever) =
                oneshot::async_channel::<Result<OperationOutput, HttpClientError>>();
            self.sender
                .send((Operation::InsertOne(input), sender))
                .unwrap();
            async move {
                return reciever
                    .await
                    .map_err(|_| HttpClientError {
                        status: hyper::StatusCode::INTERNAL_SERVER_ERROR,
                        payload: serde_json::to_value(format!("internal server error")).unwrap(),
                    })?
                    .map(|e| match e {
                        OperationOutput::InsertOne(e) => Ok(e),
                        _ => Err(HttpClientError {
                            status: hyper::StatusCode::INTERNAL_SERVER_ERROR,
                            payload: serde_json::to_value(format!("internal server error"))
                                .unwrap(),
                        }),
                    })?;
            }
        }
        pub fn add_collection(
            &self,
            input: AddCollectionInput,
        ) -> impl Future<Output = Result<AddCollectionOutput, HttpClientError>> + 'static + Send + use<>
        {
            let (sender, reciever) =
                oneshot::async_channel::<Result<OperationOutput, HttpClientError>>();
            self.sender
                .send((Operation::AddCollection(input), sender))
                .unwrap();
            async move {
                return reciever
                    .await
                    .map_err(|_| HttpClientError {
                        status: hyper::StatusCode::INTERNAL_SERVER_ERROR,
                        payload: serde_json::to_value(format!("internal server error")).unwrap(),
                    })?
                    .map(|e| match e {
                        OperationOutput::AddCollection(e) => Ok(e),
                        _ => Err(HttpClientError {
                            status: hyper::StatusCode::INTERNAL_SERVER_ERROR,
                            payload: serde_json::to_value(format!("internal server error"))
                                .unwrap(),
                        }),
                    })?;
            }
        }
        pub fn add_link(
            &self,
            input: AddLinkInput,
        ) -> impl Future<Output = Result<AddLinkOutput, HttpClientError>> + 'static + Send + use<>
        {
            let (sender, reciever) =
                oneshot::async_channel::<Result<OperationOutput, HttpClientError>>();
            self.sender
                .send((Operation::AddLink(input), sender))
                .unwrap();
            async move {
                return reciever
                    .await
                    .map_err(|_| HttpClientError {
                        status: hyper::StatusCode::INTERNAL_SERVER_ERROR,
                        payload: serde_json::to_value(format!("internal server error")).unwrap(),
                    })?
                    .map(|e| match e {
                        OperationOutput::AddLink(e) => Ok(e),
                        _ => Err(HttpClientError {
                            status: hyper::StatusCode::INTERNAL_SERVER_ERROR,
                            payload: serde_json::to_value(format!("internal server error"))
                                .unwrap(),
                        }),
                    })?;
            }
        }
        pub fn fetch_many(
            &self,
            input: FetchManyInput,
        ) -> impl Future<Output = Result<FetchManyOutput, HttpClientError>> + 'static + Send + use<>
        {
            let (sender, reciever) =
                oneshot::async_channel::<Result<OperationOutput, HttpClientError>>();
            self.sender
                .send((Operation::FetchMany(input), sender))
                .unwrap();
            async move {
                return reciever
                    .await
                    .map_err(|_| HttpClientError {
                        status: hyper::StatusCode::INTERNAL_SERVER_ERROR,
                        payload: serde_json::to_value(format!("internal server error")).unwrap(),
                    })?
                    .map(|e| match e {
                        OperationOutput::FetchMany(e) => Ok(e),
                        _ => Err(HttpClientError {
                            status: hyper::StatusCode::INTERNAL_SERVER_ERROR,
                            payload: serde_json::to_value(format!("internal server error"))
                                .unwrap(),
                        }),
                    })?;
            }
        }
    }

    pub struct JsonClientSetting {
        pub check_for_unique_filters_on_update: bool,
    }

    impl JsonClientSetting {
        pub fn default_setting() -> Self {
            Self {
                check_for_unique_filters_on_update: true,
            }
        }
    }

    /// JsonClient suitable for using in backends
    ///
    /// heavely relies on runtime-friendly extentions traits to this crate's generic traits
    /// so there is no cost to use this if you implement those base traits
    ///
    /// allows you to execute notable `impl Operation` types like `FetchOne`, `InsertOne`, `UpdateOne`, and `DeleteOne`,
    /// along with other mutable operations like 'add_collection', 'drop_collection', 'modify_collection'.
    ///
    /// I'm planning to make "DynamicJsonClient" that provides more mutable operations to extend types, links, and errors, but this is good enough for now.
    pub struct JsonClient {
        pub(crate) sender: tokio_mpsc::UnboundedSender<(
            Operation,
            oneshot::Sender<Result<OperationOutput, HttpClientError>>,
        )>,
    }

    impl JsonClient {
        pub fn new_mongo_db(tbt: Infallible) {
            let _ = tbt;
            todo!()
        }
    }
}

pub mod add_collection {
    use std::{marker::PhantomData, ops::Not, sync::Arc};

    use crate::{
        database_extention::DatabaseExt,
        fix_executor::ExecutorTrait,
        json_client_v0::{dynamic_collection::DynamicField, sqlx_type_ident::SqlxTypeHandler},
        json_client_v1::dynamic_collection::DynamicCollection,
        json_client_v1::{http_client_error::HttpClientError, sqlx_executor::ClientData},
        on_migrate::OnMigrate,
        query_builder::{Expression, StatementBuilder},
    };

    use convert_case::{Case, Casing};
    use serde::{Deserialize, Serialize};
    use sqlx::{Database, IntoArguments};

    #[derive(Debug, Serialize, Deserialize, Clone)]
    pub enum TypeSpec {
        String,
        Boolean,
    }

    #[derive(Debug, Deserialize)]
    pub struct AddCollectionInput {
        pub name: String,
        pub fields: Vec<DynamicField<TypeSpec>>,
    }

    pub type AddCollectionOutput = ();

    pub(crate) fn add_collection<S: Database>(
        this: Arc<ClientData<S>>,
        input: AddCollectionInput,
    ) -> impl Future<Output = Result<AddCollectionOutput, HttpClientError>> + 'static + Send + use<S>
    where
        S: DatabaseExt,
        bool: for<'d> sqlx::Decode<'d, S> + sqlx::Type<S> + for<'q> sqlx::Encode<'q, S>,
        std::string::String:
            sqlx::Type<S> + for<'q> sqlx::Encode<'q, S> + for<'d> sqlx::Decode<'d, S>,
        for<'a> &'a str: sqlx::ColumnIndex<<S as sqlx::Database>::Row>,
        DynamicCollection<S>: OnMigrate<Statements: Expression<'static, S>>,
        for<'a> S::Arguments<'a>: IntoArguments<'a, S>,
        S: ExecutorTrait,
    {
        async move {
            if input.name.is_empty() {
                Err("Name is empty")?
            }
            if input.name.is_case(Case::Snake).not() {
                Err("Name is not in snake case")?
            }
            if input.name.starts_with("ct_") {
                Err("Name starts with ct_")?
            }
            if input.name.starts_with("meta_") {
                Err("Name starts with meta_")?
            }

            if input.fields.iter().any(|e| e.name.starts_with("id")) {
                Err("Field name starts with id")?
            }
            if input.fields.iter().any(|e| e.name.starts_with("fk_")) {
                Err("Field name starts with fk_")?
            }

            let dc = DynamicCollection {
                name: input.name.to_case(Case::Pascal),
                name_lower_case: input.name.clone(),
                fields: input
                    .fields
                    .into_iter()
                    .map(|e| {
                        let type_info: Box<dyn SqlxTypeHandler<S> + Send + Sync> = match e.type_info
                        {
                            TypeSpec::String => Box::new(PhantomData::<String>),
                            TypeSpec::Boolean => Box::new(PhantomData::<bool>),
                        };
                        DynamicField {
                            name: e.name,
                            is_optional: e.is_optional,
                            type_info,
                        }
                    })
                    .collect(),
            };

            let mig = StatementBuilder::<S>::new_no_data(dc.clone().statments())
                .expect("bug: migration should");

            let mut this_col = this.collections.write().await;
            let mut this_mig = this.migration.write().await;

            if this_col.get(&input.name).is_some() {
                Err("collection exist")?
            }

            let mut conn = this.pool.acquire().await.unwrap();

            S::execute(&mut conn, mig.as_str()).await.unwrap();

            this_mig.push(mig);
            this_col.insert(input.name, dc.into());

            drop(this_mig);
            drop(this_col);

            Ok(())
        }
    }
}

pub mod add_link {
    use std::sync::Arc;

    use sqlx::Database;

    use crate::{
        database_extention::DatabaseExt,
        fix_executor::ExecutorTrait,
        json_client_v1::{
            http_client_error::HttpClientError,
            json_client::{AddLinkInput, AddLinkOutput},
            sqlx_executor::ClientData,
        },
        links::{
            DefaultRelationKey, relation_optional_to_many::OptionalToMany, timestamp::Timestamp,
        },
        on_migrate::OnMigrate,
        prelude::sql::ManyPossible,
        query_builder::{StatementBuilder, functional_expr::ManyImplExpression},
    };

    pub(crate) fn add_link<S: Database>(
        this: Arc<ClientData<S>>,
        input: AddLinkInput,
    ) -> impl Future<Output = Result<AddLinkOutput, HttpClientError>> + 'static + Send + use<S>
    where
        S: DatabaseExt + ExecutorTrait,
    {
        async move {
            match input {
                crate::json_client_v0::add_link::AddLinkInput::OptionalToMany { from, to } => {
                    let mut linfo_gaurd = this.link_info.write().await;

                    if linfo_gaurd
                        .optional_to_many
                        .contains(&(to.clone(), from.clone()))
                    {
                        Err("link already exists")?
                    }

                    let col_gaurd = this.collections.read().await;
                    let from_gaurd = col_gaurd
                        .get(from.as_str())
                        .ok_or("collection doesn't exist")?
                        .read()
                        .await;
                    let to_gaurd = col_gaurd
                        .get(to.as_str())
                        .ok_or("collection doesn't exist")?
                        .read()
                        .await;

                    let mig =
                        StatementBuilder::new_no_data(OnMigrate::statments(&OptionalToMany {
                            fk_unique_id: DefaultRelationKey,
                            from: from_gaurd.clone(),
                            to: to_gaurd.clone(),
                        }))
                        .expect("bug: migrations should not have any aditional data");

                    let mut mig_gaurd = this.migration.write().await;

                    let mut conn = this.pool.acquire().await.unwrap();
                    // panic!("{}", mig.as_str());
                    S::execute(&mut conn, mig.as_str()).await.unwrap();

                    mig_gaurd.push(mig);
                    linfo_gaurd
                        .optional_to_many
                        .insert((to.clone(), from.clone()));

                    drop(linfo_gaurd);
                    drop(mig_gaurd);
                    drop(from_gaurd);
                    drop(to_gaurd);
                    drop(col_gaurd);

                    Ok(())
                }
                AddLinkInput::Timestamp { collection } => {
                    let cols_gaurd = this.collections.read().await;
                    let col_gaurd = cols_gaurd
                        .get(collection.as_str())
                        .ok_or("collection doesn't exist")?
                        .read()
                        .await;

                    let mig = StatementBuilder::new_no_data(
                        ManyImplExpression::new(
                            ManyPossible(
                                Timestamp {
                                    collection: col_gaurd.clone(),
                                }
                                .statments(),
                            ),
                            "",
                            " ",
                        )
                        .unwrap(),
                    )
                    .unwrap();

                    let mut mig_gaurd = this.migration.write().await;
                    let mut linfo_gaurd = this.link_info.write().await;

                    let mut conn = this.pool.acquire().await.unwrap();
                    S::execute(&mut conn, mig.as_str()).await.unwrap();

                    mig_gaurd.push(mig);
                    linfo_gaurd.timestamped.insert(collection);

                    drop(linfo_gaurd);
                    drop(mig_gaurd);
                    drop(col_gaurd);
                    drop(cols_gaurd);

                    Ok(())
                }
            }
        }
    }
}

pub mod dynamic_order_by {
    use std::collections::BTreeMap;

    use sqlx::Database;

    use crate::{
        database_extention::DatabaseExt,
        extentions::common_expressions::Scoped,
        from_row::{FromRowAlias, FromRowData, FromRowError},
        json_client_v0::sqlx_type_ident::SqlxTypeHandler,
        json_client_v1::json_client::OrderDirection,
        json_client_v1::partial_serde::PartialSerializeV1,
        query_builder::{Expression, OpExpression, StatementBuilder},
    };

    pub struct DynamicOrderBy<S> {
        pub table: String,
        pub col: String,
        pub sqlx_ident: Box<dyn SqlxTypeHandler<S> + Send + Sync>,
        pub direction: OrderDirection,
    }

    impl<S> Clone for DynamicOrderBy<S> {
        fn clone(&self) -> Self {
            Self {
                table: self.table.clone(),
                col: self.col.clone(),
                sqlx_ident: self.sqlx_ident.clone_self(),
                direction: self.direction.clone(),
            }
        }
    }

    impl<S> Scoped for Vec<DynamicOrderBy<S>> {
        type Scoped = Vec<DynamicOrderBy<S>>;
        fn scoped(&self) -> Self::Scoped {
            self.iter()
                .map(|e| DynamicOrderBy {
                    table: e.table.clone(),
                    col: e.col.clone(),
                    sqlx_ident: e.sqlx_ident.clone_self(),
                    direction: e.direction.clone(),
                })
                .collect()
        }
    }

    impl<S> OpExpression for DynamicOrderBy<S> where S: Database {}

    impl<'q, S> Expression<'q, S> for DynamicOrderBy<S>
    where
        S: DatabaseExt,
    {
        fn expression(self, ctx: &mut StatementBuilder<'q, S>) {
            ctx.sanitize(self.table.as_str());
            ctx.syntax(".");
            ctx.sanitize(self.col.as_str());
            ctx.syntax(" ");
            match self.direction {
                OrderDirection::Asc => ctx.syntax("ASC"),
                OrderDirection::Desc => ctx.syntax("DESC"),
            }
        }
    }

    impl<S> FromRowData for DynamicOrderBy<S> {
        type RData = (String, PartialSerializeV1);
    }

    impl<'r, S: Database> FromRowAlias<'r, S::Row> for DynamicOrderBy<S> {
        fn no_alias(&self, row: &'r S::Row) -> Result<Self::RData, FromRowError> {
            let ret = self.sqlx_ident.from_row_no_alias(false, &self.col, row)?;

            Ok((self.col.clone(), PartialSerializeV1::new(ret)))
        }

        fn pre_alias(
            &self,
            row: crate::from_row::RowPreAliased<'r, S::Row>,
        ) -> Result<Self::RData, crate::from_row::FromRowError>
        where
            S::Row: sqlx::Row,
        {
            let ret = self
                .sqlx_ident
                .from_row_pre_alias_partial(false, &self.col, row)?;

            Ok((self.col.clone(), ret))
        }

        fn post_alias(
            &self,
            row: crate::from_row::RowPostAliased<'r, S::Row>,
        ) -> Result<Self::RData, crate::from_row::FromRowError>
        where
            S::Row: sqlx::Row,
        {
            let _ = row;
            panic!("in the process of deprecating this method");
        }

        fn two_alias(
            &self,
            row: crate::from_row::RowTwoAliased<'r, S::Row>,
        ) -> Result<Self::RData, crate::from_row::FromRowError>
        where
            S::Row: sqlx::Row,
        {
            let ret = self
                .sqlx_ident
                .from_row_two_alias_partial(false, &self.col, row)?;

            Ok((self.col.clone(), ret))
        }
    }

    impl<S> FromRowData for Vec<DynamicOrderBy<S>> {
        type RData = BTreeMap<String, PartialSerializeV1>;
    }

    impl<'r, S> FromRowAlias<'r, S::Row> for Vec<DynamicOrderBy<S>>
    where
        S: Database,
    {
        fn no_alias(&self, row: &'r S::Row) -> Result<Self::RData, FromRowError> {
            let mut ret = BTreeMap::new();
            for each in self.iter() {
                ret.insert(
                    each.col.clone(),
                    each.sqlx_ident
                        .from_row_no_alias_partial(false, &each.col, row)?,
                );
            }

            Ok(ret)
        }

        fn pre_alias(
            &self,
            row: crate::from_row::RowPreAliased<'r, S::Row>,
        ) -> Result<Self::RData, FromRowError>
        where
            S::Row: sqlx::Row,
        {
            let mut ret = BTreeMap::new();
            for each in self.iter() {
                ret.insert(
                    each.col.clone(),
                    each.sqlx_ident
                        .from_row_pre_alias_partial(false, &each.col, row.clone())?,
                );
            }

            Ok(ret)
        }

        fn post_alias(
            &self,
            row: crate::from_row::RowPostAliased<'r, S::Row>,
        ) -> Result<Self::RData, FromRowError>
        where
            S::Row: sqlx::Row,
        {
            let _ = row;
            panic!("in the process of deprecating this method");
        }

        fn two_alias(
            &self,
            row: crate::from_row::RowTwoAliased<'r, S::Row>,
        ) -> Result<Self::RData, FromRowError>
        where
            S::Row: sqlx::Row,
        {
            let mut ret = BTreeMap::new();
            for each in self.iter() {
                ret.insert(
                    each.col.clone(),
                    each.sqlx_ident
                        .from_row_two_alias_partial(false, &each.col, row.clone())?,
                );
            }

            Ok(ret)
        }
    }
}

pub mod supported_filter {
    use crate::expressions::ColumnEqual;
    use crate::json_client_v1::dynamic_collection::DynamicCollection;
    use crate::query_builder::functional_expr::BoxedExpression;
    use crate::{
        database_extention::DatabaseExt, json_client_v1::partial_serde::PartialDeserializeV1,
    };
    use serde::{Deserialize, Serialize};

    #[derive(Deserialize, Clone, Debug)]
    #[non_exhaustive]
    pub enum SupportedFilter {
        ColEq(ColumnEqual<String, PartialDeserializeV1>),
    }

    #[derive(Debug, Serialize)]
    pub enum InvalidFilter {
        FieldNotFound(String),
        TypeMismatch(String),
    }

    pub fn parse_supported_filter<'q, S>(
        input: Vec<SupportedFilter>,
        base: &DynamicCollection<S>,
    ) -> Result<Vec<Box<dyn BoxedExpression<S> + Send>>, InvalidFilter>
    where
        S: DatabaseExt,
    {
        let mut ret: Vec<Box<dyn BoxedExpression<S> + Send>> = vec![];
        for each in input {
            match each {
                SupportedFilter::ColEq(ColumnEqual { col, eq }) => {
                    if let Some(o) = base.fields.iter().find(|f| f.name == col) {
                        if let Ok(s) = o.type_info.to_bind_partial(eq) {
                            ret.push(Box::new(ColumnEqual { col, eq: s }));
                        } else {
                            return Err(InvalidFilter::TypeMismatch(col));
                        }
                    } else {
                        return Err(InvalidFilter::FieldNotFound(col));
                    };
                }
            }
        }

        Ok(ret)
    }
}

pub mod fetch_many {
    use crate::{
        database_extention::DatabaseExt,
        extentions::named_bind::NamedBind,
        fix_executor::ExecutorTrait,
        json_client_v0::fetch_many::{
            SupportedLinkFetchMany, extending_link_trait::JsonLinkFetchMany,
        },
        json_client_v1::dynamic_collection::DynamicCollection,
        json_client_v1::supported_filter::parse_supported_filter,
        json_client_v1::{
            dynamic_order_by::DynamicOrderBy,
            http_client_error::HttpClientError,
            json_client::{FetchManyInput, FetchManyOutput},
            sqlx_executor::ClientData,
        },
        links::{
            DefaultRelationKey, relation_optional_to_many::OptionalToMany, timestamp::Timestamp,
        },
        operations::{CollectionOutput, Operation, fetch_many::FetchMany},
    };
    use sqlx::Database;
    use std::{ops::Not, sync::Arc};

    pub mod extending_link_trait {
        use serde::Serialize;
        use sqlx::Database;
        use tracing::warn;

        use crate::from_row::{FromRowAlias, FromRowData};
        use crate::json_client_v1::partial_serde::PartialSerializeV1;
        use crate::operations::OperationOutput;
        use crate::operations::boxed_operation::BoxedOperation;
        use crate::operations::fetch_many::LinkFetch;
        use crate::query_builder::ManyBoxedExpressions;
        use crate::select_items_trait_object::SelectItemsTraitObject;
        use crate::select_items_trait_object::ToImplSelectItems;
        use crate::{database_extention::DatabaseExt, extentions::common_expressions::Aliased};
        use core::fmt;
        use std::any::Any;
        use std::ops::{Deref, DerefMut};

        pub trait JsonLinkFetchMany<S> {
            fn select_items_expr(&self) -> Box<dyn SelectItemsTraitObject<S, ()>>;
            fn post_select_each_2(&self, item: &Box<dyn Any + Send>, poi: &mut Box<dyn Any + Send>);
            fn post_operation_input_init_2(&self) -> Box<dyn Any + Send>;
            fn post_select_2(
                &self,
                input: Box<dyn Any + Send>,
            ) -> Box<dyn BoxedOperation<S> + Send>;
            fn take_2(
                &self,
                item: Box<dyn Any + Send>,
                op: &mut Box<dyn Any + Send>,
            ) -> PartialSerializeV1;
            fn join_expr(&self) -> Box<dyn ManyBoxedExpressions<S> + Send>;
            fn wheres_expr(&self) -> Box<dyn ManyBoxedExpressions<S> + Send>;
        }

        impl<S, T> JsonLinkFetchMany<S> for T
        where
            T: Clone + Send + 'static,
            T::SelectItems: Send,
            T::SelectItems: FromRowData,
            S: DatabaseExt,
            T: LinkFetch,
            T::SelectItems: Send
                + Aliased<
                    NumAliased: 'static + Send + ManyBoxedExpressions<S>,
                    Aliased: 'static + Send + ManyBoxedExpressions<S>,
                >,
            T::OpInput: 'static + Send,
            T::Op: Send + 'static + BoxedOperation<S>,
            T::Op: OperationOutput,
            T::Output: Serialize,
            T::SelectItems: FromRowData<RData: Send + 'static>,
            T::SelectItems: for<'r> FromRowAlias<'r, S::Row>,
            T::Join: Send + 'static + ManyBoxedExpressions<S>,
            T::Wheres: Send + 'static + ManyBoxedExpressions<S>,
            // for PartialSerialize
            T::Output: Clone + fmt::Debug,
        {
            fn join_expr(&self) -> Box<dyn ManyBoxedExpressions<S> + Send> {
                Box::new(self.non_duplicating_join_expressions())
            }
            fn wheres_expr(&self) -> Box<dyn ManyBoxedExpressions<S> + Send> {
                Box::new(self.where_expressions())
            }
            fn take_2(
                &self,
                item: Box<dyn Any + Send>,
                op: &mut Box<dyn Any + Send>,
            ) -> PartialSerializeV1 {
                let s = self.take_many(
                    *item
                        .downcast::<<T::SelectItems as FromRowData>::RData>()
                        .unwrap(),
                    op.downcast_mut::<<T::Op as OperationOutput>::Output>()
                        .unwrap(),
                );

                PartialSerializeV1::new(s)
            }
            fn select_items_expr(&self) -> Box<dyn SelectItemsTraitObject<S, ()>> {
                Box::new(ToImplSelectItems {
                    select_items: self.non_aggregating_select_items(),
                    cast_from_row_result: (),
                })
            }
            fn post_select_each_2(
                &self,
                item: &Box<dyn Any + Send>,
                poi: &mut Box<dyn Any + Send>,
            ) {
                let ite_down = item
                    .deref()
                    .downcast_ref::<<T::SelectItems as FromRowData>::RData>();

                let poi_down = poi.deref_mut().downcast_mut::<T::OpInput>();

                self.operation_fix_on_many(ite_down.unwrap(), poi_down.unwrap())
            }

            fn post_operation_input_init_2(&self) -> Box<dyn Any + Send> {
                let ret = self.operation_initialize_input();

                Box::new(ret)
            }
            fn post_select_2(
                &self,
                input: Box<dyn Any + Send>,
            ) -> Box<dyn BoxedOperation<S> + Send> {
                Box::new(self.operation_construct(*input.downcast::<T::OpInput>().unwrap()))
            }
        }

        impl<'r, S> LinkFetch for Box<dyn JsonLinkFetchMany<S> + Send + 'r>
        where
            Box<dyn SelectItemsTraitObject<S, ()>>: FromRowData<RData = Box<dyn Any + Send>>,
            Box<dyn BoxedOperation<S> + Send>: OperationOutput<Output = Box<dyn Any + Send>>,
        {
            type SelectItems = Box<dyn SelectItemsTraitObject<S, ()>>;

            fn non_aggregating_select_items(&self) -> Self::SelectItems {
                self.select_items_expr()
            }

            fn operation_fix_on_many(&self, item: &Box<dyn Any + Send>, poi: &mut Self::OpInput)
            where
                Self::SelectItems: FromRowData,
            {
                self.post_select_each_2(item, poi)
            }

            fn take_many(
                &self,
                item: <Self::SelectItems as FromRowData>::RData,
                op: &mut <Self::Op as OperationOutput>::Output,
            ) -> Self::Output
            where
                Self::SelectItems: FromRowData,
                Self::Op: OperationOutput,
            {
                self.take_2(item, op)
            }

            type Join = Box<dyn ManyBoxedExpressions<S> + Send>;

            fn non_duplicating_join_expressions(&self) -> Self::Join {
                self.join_expr()
            }

            type Wheres = Box<dyn ManyBoxedExpressions<S> + Send>;

            fn where_expressions(&self) -> Self::Wheres {
                self.wheres_expr()
            }

            type Output = PartialSerializeV1;

            type OpInput = Box<dyn Any + Send>;

            fn operation_initialize_input(&self) -> Self::OpInput {
                let ret = self.post_operation_input_init_2();

                ret
            }

            type Op = Box<dyn BoxedOperation<S> + Send>;

            fn operation_construct(&self, input: Self::OpInput) -> Self::Op
            where
                Self::SelectItems: FromRowData,
            {
                self.post_select_2(input)
            }
        }

        impl<'r, S> LinkFetch for Vec<Box<dyn JsonLinkFetchMany<S> + Send + 'r>>
        where
            S: Database,
            Vec<Box<dyn SelectItemsTraitObject<S, ()>>>:
                FromRowData<RData = Vec<Box<dyn Any + Send>>>,
            Vec<Box<dyn BoxedOperation<S> + Send>>:
                OperationOutput<Output = Vec<Box<dyn Any + Send>>>,
        {
            type SelectItems = Vec<Box<dyn SelectItemsTraitObject<S, ()>>>;

            fn non_aggregating_select_items(&self) -> Self::SelectItems {
                let s = self
                    .iter()
                    .map(|each| each.select_items_expr())
                    .collect::<Vec<_>>();

                s
            }

            fn operation_fix_on_many(
                &self,
                item: &Vec<Box<dyn Any + Send>>,
                poi: &mut Vec<Box<dyn Any + Send>>,
            ) where
                Self::SelectItems: FromRowData,
            {
                for (i, each) in self.iter().enumerate() {
                    let corresponding_poi = poi.get_mut(i).unwrap();
                    let corresponding_item = item.get(i).unwrap();
                    each.post_select_each_2(corresponding_item, corresponding_poi);
                }
            }

            fn take_many(
                &self,
                item: Vec<Box<dyn Any + Send>>,
                op: &mut Vec<Box<dyn Any + Send>>,
            ) -> Self::Output
            where
                Self::SelectItems: FromRowData,
                Self::Op: OperationOutput,
            {
                let mut ret = vec![];
                for (i, (each, item)) in self.iter().zip(item.into_iter()).enumerate() {
                    let corresponding_op = op.get_mut(i).unwrap();
                    ret.push(each.take_many(item, corresponding_op));
                }

                ret
            }

            type Join = Box<dyn ManyBoxedExpressions<S> + Send>;

            fn non_duplicating_join_expressions(&self) -> Self::Join {
                let first = self.iter().next().unwrap();
                warn!("multiple links");
                Box::new(first.join_expr())
            }

            type Wheres = ();

            fn where_expressions(&self) -> Self::Wheres {}

            type Output = Vec<PartialSerializeV1>;

            type OpInput = Vec<Box<dyn Any + Send>>;

            fn operation_initialize_input(&self) -> Self::OpInput {
                let ret = self
                    .iter()
                    .map(|each| each.post_operation_input_init_2())
                    .collect::<Vec<_>>();
                ret
            }

            type Op = Vec<Box<dyn BoxedOperation<S> + Send>>;

            fn operation_construct(&self, input: Self::OpInput) -> Self::Op
            where
                Self::SelectItems: FromRowData,
            {
                let mut ret = vec![];
                for (each, input) in self.iter().zip(input.into_iter()) {
                    let res = each.operation_construct(input);
                    ret.push(res);
                }
                ret
            }
        }
    }

    pub(crate) fn fetch_many<S: Database>(
        this: Arc<ClientData<S>>,
        input: FetchManyInput,
    ) -> impl Future<Output = Result<FetchManyOutput, HttpClientError>> + 'static + Send + use<S>
    where
        S: DatabaseExt + ExecutorTrait,
        OptionalToMany<DefaultRelationKey, DynamicCollection<S>, DynamicCollection<S>>:
            JsonLinkFetchMany<S>,
        Timestamp<DynamicCollection<S>>: JsonLinkFetchMany<S>,
        for<'q> &'q str: sqlx::ColumnIndex<<S as sqlx::Database>::Row>,
        String: for<'q> sqlx::Encode<'q, S> + for<'q> sqlx::Decode<'q, S> + sqlx::Type<S>,
        i64: for<'q> sqlx::Encode<'q, S> + for<'q> sqlx::Decode<'q, S> + sqlx::Type<S>,
    {
        async move {
            let cols_gaurd = this.collections.read().await;
            let col_gaurd = cols_gaurd
                .get(input.base.as_str())
                .ok_or("collection doesn't exist")?
                .read()
                .await;
            let rel_gaurd = this.link_info.read().await;

            let base = col_gaurd.clone();

            let mut all_gaurds = vec![col_gaurd];

            let wheres = parse_supported_filter(input.filters, &base)?;

            let links = {
                let mut links =
                    Vec::<Box<dyn extending_link_trait::JsonLinkFetchMany<S> + Send>>::new();
                for each in input.links {
                    match each {
                        SupportedLinkFetchMany::OptionalToMany { to } => {
                            let to = if let Some(to) = cols_gaurd.get(to.as_str()) {
                                let to_gaurd = to.read().await;
                                let to = to_gaurd.clone();
                                all_gaurds.push(to_gaurd);
                                to
                            } else {
                                Err(String::from("related collection is not found"))?
                            };

                            if rel_gaurd.optional_to_many.contains(&(
                                to.name_lower_case.clone(),
                                base.name_lower_case.clone(),
                            )) {
                                links.push(Box::new(OptionalToMany {
                                    fk_unique_id: DefaultRelationKey,
                                    from: base.clone(),
                                    to,
                                }));

                                continue;
                            } else if rel_gaurd.optional_to_many.contains(&(
                                base.name_lower_case.clone(),
                                to.name_lower_case.clone(),
                            )) {
                                todo!("reverse optional to many ")
                            } else {
                                Err("relation doesn't exist between x, y")?
                            };
                        }
                        SupportedLinkFetchMany::Timestamp => {
                            if rel_gaurd.timestamped.contains(input.base.as_str()).not() {
                                Err("collection is not timestamped")?
                            }

                            links.push(Box::new(Timestamp {
                                collection: base.clone(),
                            }));
                        }
                    }
                }
                links
            };

            let order_by = {
                let mut order_by = vec![];

                for any in input.pagination.order_by {
                    let found = base.fields.iter().find(|e| e.name == any.col);

                    if let Some(found) = found {
                        order_by.push(DynamicOrderBy {
                            table: base.name.clone(),
                            col: any.col,
                            sqlx_ident: found.type_info.clone_self(),
                            direction: any.direction,
                        });
                    } else {
                        Err("column doesn't exist")?
                    }
                }

                order_by
            };

            let first_item = if let Some(first_item) = input.pagination.first_item {
                let mut named_binds = vec![];
                for (key, value) in first_item.attributes {
                    let found = base.fields.iter().find(|e| e.name == key);

                    if let Some(found) = found {
                        let bind = found
                            .type_info
                            .to_bind_partial(value)
                            .map_err(|_| "faile to parse")?;
                        named_binds.push(NamedBind {
                            table: base.name.clone(),
                            name: key.clone(),
                            value: bind,
                        });
                    } else {
                        Err("column doesn't exist")?
                    }
                }

                Some((first_item.id, named_binds))
            } else {
                None
            };

            let mut conn = this.pool.acquire().await.unwrap();

            let s = FetchMany {
                base,
                wheres,
                links,
                cursor_order_by: order_by,
                cursor_first_item: first_item,
                limit: input.pagination.limit,
            };

            let out = Operation::<S>::exec_operation(s, &mut conn).await;

            let out = crate::operations::fetch_many::ManyOutput {
                items: out.items,
                next_item: out.next_item.map(|(id, next)| CollectionOutput {
                    id,
                    attributes: next,
                }),
            };

            // let why: &serde_json::Map<String, serde_json::Value> =
            //     &out.next_item.as_ref().unwrap().attributes;

            drop(rel_gaurd);
            drop(all_gaurds);
            drop(cols_gaurd);

            Ok(out)
        }
    }
}

pub mod insert_one {
    use std::sync::Arc;

    use sqlx::Database;

    use crate::{
        database_extention::DatabaseExt,
        fix_executor::ExecutorTrait,
        json_client_v1::{
            http_client_error::HttpClientError,
            json_client::{InsertOneInput, InsertOneOutput},
            sqlx_executor::ClientData,
        },
    };

    pub(crate) fn insert_one<S: Database>(
        this: Arc<ClientData<S>>,
        input: InsertOneInput,
    ) -> impl Future<Output = Result<InsertOneOutput, HttpClientError>> + 'static + Send + use<S>
    where
        S: DatabaseExt + ExecutorTrait,
    {
        let _ = (this, input);
        async move { todo!() }
    }
}

// pub mod database_for_json_client

// pub mod supported_filter

// pub mod links_utils

// pub mod dynamic_collection

// pub mod add_collection

// pub mod add_link

// pub mod fetch_many

// pub mod fetch_one

// to be refactored
// pub mod json_link_fetch_one_extention

// to be refactored
// pub mod update_one

// to be refactored
// pub mod supported_links_on_fetch_one

// to be refactored
// pub mod insert_one

// to be refactored
// pub mod delete_one

// mod as_router

pub mod partial_serde {
    use core::fmt;
    use std::{
        ops::{Deref, Range},
        sync::Arc,
    };

    use serde::{
        Deserialize, Deserializer, Serialize,
        de::{DeserializeOwned, Visitor},
    };

    #[derive(Clone, Debug, Serialize, Deserialize)]
    pub struct PartialSerializeV1(serde_json::Value);

    impl PartialSerializeV1 {
        pub fn new<T: 'static + Clone + Serialize + fmt::Debug>(value: T) -> Self {
            Self(serde_json::to_value(value).expect("claw_ql_bug: when serialize ever fail?"))
        }
    }

    #[derive(Debug, Clone)]
    pub struct PartialDeserializeV1(String);

    impl PartialDeserializeV1 {
        pub fn continue_deserialize<T>(self) -> Result<T, serde_json::Error>
        where
            T: DeserializeOwned,
        {
            let value = serde_json::from_str::<T>(&self.0)?;
            Ok(value)
        }
    }

    impl<'de> Deserialize<'de> for PartialDeserializeV1 {
        fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
        where
            D: Deserializer<'de>,
        {
            let value = deserializer.deserialize_any(AnyVisitor)?;

            Ok(PartialDeserializeV1(value))
        }
    }

    pub struct AnyVisitor;

    // this is not a complete set of supported types, but usually PartialDeserialize is used for maps so this is good enough
    impl<'de> Visitor<'de> for AnyVisitor {
        type Value = String;

        fn expecting(&self, formatter: &mut std::fmt::Formatter) -> std::fmt::Result {
            formatter.write_str("any value")
        }
        fn visit_str<E>(self, v: &str) -> Result<Self::Value, E>
        where
            E: serde::de::Error,
        {
            Ok(format!("\"{}\"", v))
        }
        fn visit_string<E>(self, v: String) -> Result<Self::Value, E>
        where
            E: serde::de::Error,
        {
            Ok(format!("\"{}\"", v))
        }
        fn visit_bool<E>(self, v: bool) -> Result<Self::Value, E>
        where
            E: serde::de::Error,
        {
            Ok(String::from(if v { "true" } else { "false" }))
        }

        // here I deserialize maps as if they are serde_json::Value
        // there might be a better performant way to do this, but for now this is functional
        fn visit_map<A>(self, mut map: A) -> Result<Self::Value, A::Error>
        where
            A: serde::de::MapAccess<'de>,
        {
            static OPEN_BRACKET: char = '{';
            static CLOSE_BRACKET: char = '}';
            let mut ret = String::from(OPEN_BRACKET);

            if let Some((key, value)) = map.next_entry::<String, serde_json::Value>()? {
                ret.push('"');
                ret.push_str(&key);
                ret.push_str("\": ");
                ret.push_str(&value.to_string());
            }

            while let Some((key, value)) = map.next_entry::<String, serde_json::Value>()? {
                ret.push_str(", ");
                ret.push('"');
                ret.push_str(&key);
                ret.push_str("\": ");
                ret.push_str(&value.to_string());
            }
            ret.push(CLOSE_BRACKET);
            Ok(ret)
        }
    }

    pub struct ArcSubStrJCV1 {
        inner: Arc<str>,
        range: Range<usize>,
    }

    impl ArcSubStrJCV1 {
        pub fn from_arc(arc: &Arc<str>, range: Range<usize>) -> Self {
            Self {
                inner: arc.clone(),
                range,
            }
        }
        pub fn as_str(&self) -> &str {
            &self.inner[self.range.clone()]
        }
    }

    impl Deref for ArcSubStrJCV1 {
        type Target = str;
        fn deref(&self) -> &Self::Target {
            &self.inner[self.range.clone()]
        }
    }

    impl AsRef<str> for ArcSubStrJCV1 {
        fn as_ref(&self) -> &str {
            &self.inner[self.range.clone()]
        }
    }

    impl AsRef<[u8]> for ArcSubStrJCV1 {
        fn as_ref(&self) -> &[u8] {
            self.inner[self.range.clone()].as_bytes()
        }
    }

    #[allow(unused)]
    #[cfg(test)]
    mod test {
        use std::ops::{Deref, Range};

        use serde::Deserialize;

        use crate::{
            json_client_v1::partial_serde::{
                ArcSubStrJCV1, PartialDeserializeV1, PartialSerializeV1,
            },
            operations::LinkedOutput,
        };

        #[derive(Debug, Deserialize)]
        struct Input<T> {
            base: String,
            data: T,
        }

        #[test]
        fn test_partial_serde() {
            let input = "
{
    'base': 'todo',
    'data': {
        'title': 'new_todo',
        'done': false,
        'description': 'description',
        'extra': {
            'extra_title': 'extra_title'
        }
    }
}
"
            .replace("'", "\"");

            let s = serde_json::from_str::<Input<PartialDeserializeV1>>(&input).unwrap();

            pretty_assertions::assert_eq!(s.base, String::from("todo"));
            pretty_assertions::assert_eq!(
                &s.data.0,
                "{\"title\": \"new_todo\", \"done\": false, \"description\": \"description\", \"extra\": {\"extra_title\":\"extra_title\"}}"
            );

            let input = "
[
    'first_value', 'second_value'
]
"
            .replace("'", "\"");

            let s = serde_json::from_str::<(String, PartialDeserializeV1)>(&input).unwrap();

            pretty_assertions::assert_eq!(s.0, String::from("first_value"));
            pretty_assertions::assert_eq!(&s.1.0, "\"second_value\"");
        }

        #[test]
        fn arc_str() {
            use std::sync::Arc;

            let input: Arc<str> = Arc::from("hello world");

            let clone_of_first_alloc: Arc<str> = input.clone();
            let slice_of_first_alloc: ArcSubStrJCV1 = ArcSubStrJCV1::from_arc(&input, 0..5);

            assert_eq!(slice_of_first_alloc.as_str(), "hello");

            assert_eq!(Arc::strong_count(&input), 3);
        }
    }
}
