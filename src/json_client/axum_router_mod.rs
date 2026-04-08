use axum::debug_handler;
use axum::extract::State;
use axum::response::{IntoResponse, Response};
use axum::{Json, Router, body::Bytes, extract::Path, routing::get};
use hyper::StatusCode;
use serde::Deserialize;
use serde::Serialize;
use serde_json::json;
use sqlx::Database;
use sqlx::Pool;
use sqlx::Sqlite;
use std::sync::Arc;
use std::sync::RwLock;
use tokio::sync::RwLock as TrwLock;

use crate::json_client::JsonClient;
use crate::json_client::add_collection::AddCollectionBody;

pub trait HttpError {
    fn status_code(&self) -> StatusCode;
    fn sub_code(&self) -> Option<&'static str> {
        None
    }
    fn sub_message(&self) -> Option<String> {
        None
    }
}

impl HttpError for String {
    fn status_code(&self) -> StatusCode {
        // string lacks semantics, for now it is internal error
        StatusCode::INTERNAL_SERVER_ERROR
    }
    fn sub_message(&self) -> Option<String> {
        Some(self.clone())
    }
}

impl HttpError for serde_json::Value {
    fn status_code(&self) -> StatusCode {
        todo!()
    }
}

pub fn axum_router(jc: Arc<JsonClient<Sqlite>>) -> Router<()> {
    axum::Router::new()
        .route(
            "/select_one_{collection}",
            get(|path: Path<String>| async move {
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
}

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
