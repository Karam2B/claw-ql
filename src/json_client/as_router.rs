use crate::{
    QueryBuilder,
    json_client::{JsonClient, select_one::SelectOneInput},
    operations::LinkedOutput,
    update_mod::update,
};
use axum::{
    Json, Router,
    extract::{Path, State},
    handler::Handler,
    routing::get,
};
use hyper::Request;
use serde::Deserialize;
use serde_json::{Map, Value, json};
use sqlx::{ColumnIndex, Database, Decode, Encode, Sqlite, prelude::Type};
use std::{convert::Infallible, pin::Pin, sync::Arc, task::Poll};
use tower_service::Service;

impl<S: QueryBuilder> JsonClient<S>
where
    S: QueryBuilder<Output = <S as Database>::Arguments<'static>> + Send,
    S: QueryBuilder<Fragment: Send, Context1: Send>,
    for<'c> &'c mut S::Connection: sqlx::Executor<'c, Database = S>,
    for<'e> i64: Encode<'e, S> + Type<S> + Decode<'e, S>,
    for<'e> &'e str: ColumnIndex<S::Row>,
{
    /// note: current implementation is as follow, but breaking changes
    /// can occur at any time!
    /// 1. (todo) no permissions are implemented
    /// 2. collections have the path `/collections/:collection`
    /// 3. (todo) these endpoint implemeted for each collection
    ///    - [ ] GET / `to retrieve all collections`
    ///    - [x] GET /:id `to retrieve one collection`
    ///    - [ ] POST /:id `update one collection`
    ///    - [ ] DELETE /:id `delete one collection`
    ///    - [ ] PUT / `insert one collection`
    /// 4. (todo) no real-time update is implemted (will be at /rt/:collection)
    pub fn as_router(self) -> Router<()> {
        let router = {
            Router::new()
                .route("/{collection}/", get(get_all::<S>).post(insert_one))
                .route(
                    "/{collection}/{id}",
                    get(get_one).put(update_one).delete(delete_one),
                )
                .with_state(Arc::new(self))


    axum::Router::new()
        .route(
            "/select_one_{collection}",
            get(|
    jc: State<Arc<TrwLock<JsonClient<Sqlite>>>>,
                path: Path<String>| async move {
                todo!();
                ()
            }),
        )
        .route(
            "/select_many_{collection}",
            get(|path: Path<String>| async move {
                todo!("select_many");
                return ();
            }),
        )
        .route(
            "/insert_{collection}",
            get(|path: Path<String>| async move {
                todo!("insert");
                return ();
            }),
        )
        .route(
            "/update_{collection}",
            get(|path: Path<String>| async move {
                todo!("update");
                return ();
            }),
        )
        .route(
            "/delete_{collection}",
            get(|path: Path<String>| async move {
                todo!("delete");
                return ();
            }),
        )
        };

#[debug_handler]
async fn add_collection_wrapper(
    jc: State<Arc<TrwLock<JsonClient<Sqlite>>>>,
    body: Json<AddCollectionBody>,
) -> Response {
    let res = jc.write().await.add_collection(body.0).await;

    let res = match res {
        Ok(ok) => ok,
        Err(err) => {
            let _ = err.sub_code();
            let _ = err.sub_message();
            return (err.status_code(), Json(json!(null))).into_response();
        }
    };

    Json(res).into_response()
}

#[debug_handler]
async fn select_one_wrapper(jc: State<Arc<TrwLock<JsonClient<Sqlite>>>>) -> Response {
    todo!()
}

#[cfg(feature = "http")]
pub fn axum_router_dynamic(pool: Pool<Sqlite>) -> Router<()> {
    use std::{collections::HashMap, marker::PhantomData};

    use crate::json_client::add_collection::LiqType;

    let jc = Arc::new(tokio::sync::RwLock::new(JsonClient {
        collections: Default::default(),
        links: Default::default(),
        migration: Vec::default(),
        filter_extentions: Default::default(),
        errors_log: Default::default(),
        error_count: 0.into(),
        type_extentions: HashMap::from_iter([
            (
                "core::i32".to_string(),
                Box::new(PhantomData::<i32>) as Box<dyn LiqType<Sqlite>>,
            ),
            (
                "core::string".to_string(),
                Box::new(PhantomData::<String>) as Box<dyn LiqType<Sqlite>>,
            ),
        ]),
        db: pool,
    }));

    axum::Router::new()
        .route("/admin/add_collection", get(add_collection_wrapper))
        .route("/api/select_one_{collection}", get(select_one_wrapper))
        .with_state(jc)
}

        router
    }
}
