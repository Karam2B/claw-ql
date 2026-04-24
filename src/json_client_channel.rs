pub use crate::json_client::dynamic_collection;
pub use crate::json_client::sqlx_type_ident;
pub use crate::json_client::to_bind_trait;

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

pub mod sqlx_executor {
    use super::json_client::JsonClientSetting;
    use super::json_client::Operation;
    use super::json_client::OperationOutput;
    use crate::database_extention::DatabaseExt;
    use crate::fix_executor::ExecutorTrait;
    use crate::json_client::dynamic_collection::DynamicCollection;

    use crate::json_client::fetch_many::extending_link_trait::JsonLinkFetchMany;
    use crate::json_client_channel::add_collection::add_collection;
    use crate::json_client_channel::add_link::add_link;
    use crate::json_client_channel::fetch_many::fetch_many;
    use crate::json_client_channel::http_client_error::HttpClientError;
    use crate::json_client_channel::json_client::JsonClient;
    use crate::links::DefaultRelationKey;
    use crate::links::relation_optional_to_many::OptionalToMany;
    use crate::on_migrate::OnMigrate;
    use crate::query_builder::Expression;
    use futures::future::Future;
    use oneshot;
    use sqlx::Database;
    use sqlx::IntoArguments;
    use sqlx::Pool;
    use sqlx::Sqlite;
    use std::collections::HashMap;
    use std::convert::Infallible;
    use std::sync::Arc;
    use tokio::sync::RwLock as Trw;
    use tokio::sync::mpsc as tokio_mpsc;

    pub type LinkInformations = crate::json_client::json_client::LinkInformations;

    pub struct ClientData<S: Database> {
        pub(crate) collections: Trw<HashMap<String, Trw<DynamicCollection<S>>>>,
        pub(crate) migration: Trw<Vec<String>>,
        pub(crate) link_info: Trw<LinkInformations>,
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
    {
        pub fn run(mut self) -> impl Future<Output = Infallible> {
            async move {
                loop {
                    let operation = self.reciever.recv().await.unwrap();

                    macro_rules! macr {
                        ($([$operation:ident, $fn:ident]),*) => {
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
                        [AddCollection, add_collection],
                        [AddLink, add_link],
                        [FetchMany, fetch_many],
                        [DropCollection, drop_collection]
                    );
                }
            }
        }
    }
}

pub mod json_client {
    use std::convert::Infallible;
    use tokio::sync::mpsc as tokio_mpsc;

    use crate::json_client as old_mod;
    use crate::json_client_channel::{
        add_collection::{AddCollectionInput, AddCollectionOutput},
        http_client_error::HttpClientError,
    };

    pub type AddLinkInput = old_mod::add_link::AddLinkInput;
    pub type AddLinkOutput = old_mod::add_link::AddLinkOutput;
    pub type FetchManyInput = old_mod::fetch_many::FetchManyInput;
    pub type FetchManyOutput = old_mod::fetch_many::FetchManyOutput;

    #[derive(Debug)]
    #[non_exhaustive]
    pub enum Operation {
        AddCollection(AddCollectionInput),
        AddLink(AddLinkInput),
        FetchMany(FetchManyInput),
        DropCollection(&'static str),
    }

    #[derive(Debug)]
    #[non_exhaustive]
    pub enum OperationOutput {
        AddCollection(AddCollectionOutput),
        AddLink(AddLinkOutput),
        FetchMany(FetchManyOutput),
        DropCollection(&'static str),
    }

    impl JsonClient {
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
                reciever
                    .await
                    .map_err(|_| HttpClientError {
                        status: hyper::StatusCode::INTERNAL_SERVER_ERROR,
                        payload: serde_json::to_value(format!("internal server error")).unwrap(),
                    })?
                    .map(|e| match e {
                        OperationOutput::AddCollection(e) => e,
                        _ => panic!("bug: unexpected operation output"),
                    })
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
                reciever
                    .await
                    .map_err(|_| HttpClientError {
                        status: hyper::StatusCode::INTERNAL_SERVER_ERROR,
                        payload: serde_json::to_value(format!("internal server error")).unwrap(),
                    })?
                    .map(|e| match e {
                        OperationOutput::AddLink(e) => e,
                        _ => panic!("bug: unexpected operation output"),
                    })
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
                reciever
                    .await
                    .map_err(|_| HttpClientError {
                        status: hyper::StatusCode::INTERNAL_SERVER_ERROR,
                        payload: serde_json::to_value(format!("internal server error")).unwrap(),
                    })?
                    .map(|e| match e {
                        OperationOutput::FetchMany(e) => e,
                        _ => panic!("bug: unexpected operation output"),
                    })
            }
        }
        pub fn drop_collection(
            &self,
            input: &'static str,
        ) -> impl Future<Output = Result<&'static str, HttpClientError>> + 'static + Send + use<>
        {
            let (sender, reciever) =
                oneshot::async_channel::<Result<OperationOutput, HttpClientError>>();
            self.sender
                .send((Operation::DropCollection(input), sender))
                .unwrap();
            async move {
                reciever
                    .await
                    .map_err(|_| HttpClientError {
                        status: hyper::StatusCode::INTERNAL_SERVER_ERROR,
                        payload: serde_json::to_value(format!("internal server error")).unwrap(),
                    })?
                    .map(|e| match e {
                        OperationOutput::DropCollection(e) => e,
                        _ => panic!("bug: unexpected operation output"),
                    })
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
        json_client::{dynamic_collection::DynamicCollection, sqlx_type_ident::SqlxTypeHandler},
        json_client_channel::{http_client_error::HttpClientError, sqlx_executor::ClientData},
        on_migrate::OnMigrate,
        query_builder::{Expression, StatementBuilder},
    };

    use super::dynamic_collection::DynamicField;
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

    use convert_case::pattern::toggle;
    use sqlx::Database;

    use crate::{
        database_extention::DatabaseExt,
        fix_executor::ExecutorTrait,
        json_client::dynamic_collection::DynamicCollection,
        json_client_channel::{
            http_client_error::HttpClientError,
            json_client::{AddLinkInput, AddLinkOutput},
            sqlx_executor::ClientData,
        },
        links::{
            DefaultRelationKey, relation_optional_to_many::OptionalToMany, timestamp::Timestamp,
        },
        on_migrate::OnMigrate,
        operations::LinkedOutput,
        prelude::sql::{self, ManyPossible},
        query_builder::{Expression, StatementBuilder, functional_expr::ManyImplExpression},
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
                crate::json_client::add_link::AddLinkInput::OptionalToMany { from, to } => {
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
                            foriegn_key: DefaultRelationKey,
                            from: from_gaurd.clone(),
                            to: to_gaurd.clone(),
                        }))
                        .expect("bug: migrations should not have any aditional data");

                    let mut mig_gaurd = this.migration.write().await;
                    let mut linfo_gaurd = this.link_info.write().await;

                    let mut conn = this.pool.acquire().await.unwrap();
                    // panic!("{}", mig.as_str());
                    S::execute(&mut conn, mig.as_str()).await.unwrap();

                    mig_gaurd.push(mig);
                    linfo_gaurd
                        .optional_to_many
                        .insert((to.clone(), from.clone()), true);
                    linfo_gaurd
                        .optional_to_many
                        .insert((to.clone(), from.clone()), false);

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

pub mod fetch_many {
    use crate::{
        database_extention::DatabaseExt,
        fix_executor::ExecutorTrait,
        json_client::{
            dynamic_collection::DynamicCollection,
            fetch_many::{SupportedLinkFetchMany, extending_link_trait::JsonLinkFetchMany},
            supported_filter::parse_supported_filter,
        },
        json_client_channel::{
            http_client_error::HttpClientError,
            json_client::{FetchManyInput, FetchManyOutput},
            sqlx_executor::ClientData,
        },
        links::{DefaultRelationKey, relation_optional_to_many::OptionalToMany},
    };
    use sqlx::Database;
    use std::sync::Arc;

    pub(crate) fn fetch_many<S: Database>(
        this: Arc<ClientData<S>>,
        input: FetchManyInput,
    ) -> impl Future<Output = Result<FetchManyOutput, HttpClientError>> + 'static + Send + use<S>
    where
        S: DatabaseExt + ExecutorTrait,
        OptionalToMany<DefaultRelationKey, DynamicCollection<S>, DynamicCollection<S>>:
            JsonLinkFetchMany<S>,
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
                let mut links = Vec::<Box<dyn JsonLinkFetchMany<S> + Send>>::new();
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

                            if rel_gaurd.optional_to_many.contains_key(&(
                                base.name_lower_case.clone(),
                                to.name_lower_case.clone(),
                            )) {
                                links.push(Box::new(OptionalToMany {
                                    foriegn_key: DefaultRelationKey,
                                    from: base.clone(),
                                    to,
                                }));
                            } else if rel_gaurd.optional_to_many.contains_key(&(
                                to.name_lower_case.clone(),
                                base.name_lower_case.clone(),
                            )) {
                                todo!("reverse optional to many ")
                            } else {
                                Err("relation doesn't exist between x, y")?
                            };
                        }
                        _ => todo!("unsupported link"),
                    }
                }
                links
            };

            todo!()
        }
    }
}

#[claw_ql_macros::skip]
pub mod links_utils {
    use std::{ops::Not, sync::Arc};

    use crate::{
        json_client::{dynamic_collection::DynamicCollection, json_client::JsonClient},
        links::{DefaultRelationKey, relation_optional_to_many::OptionalToMany},
    };
    use sqlx::Database;
    use tokio::sync::RwLockReadGuard as TrwLockReadGuard;

    pub async fn get_optional_to_many<S: Database>(
        base: &DynamicCollection<S>,
        to: String,
        jc: Arc<super::sqlx_executor::ClientData<S>>,
        all_gaurds: &mut Vec<Arc<TrwLockReadGuard<'_, DynamicCollection<S>>>>,
    ) -> Result<OptionalToMany<DefaultRelationKey, DynamicCollection<S>, DynamicCollection<S>>, ()>
    {
        if jc
            .links
            .optional_to_many
            .contains_key(&(base.name_lower_case.clone(), to.to_string()))
            .not()
        {
            panic!()
        }

        let to = if let Some(e) = jc.collections.get(to.as_str()) {
            e.read().await.clone()
        } else {
            panic!();
        };

        Ok(OptionalToMany {
            foriegn_key: DefaultRelationKey,
            from: base.clone(),
            to,
        })
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
