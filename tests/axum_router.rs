use axum::{Json, Router, body::Bytes, extract::Request, response::IntoResponse, routing::get};
use claw_ql::{
    ConnectInMemory, builder_pattern::BuilderPattern, json_client::builder_pattern::to_json_client,
    migration::to_migrate,
};
use claw_ql_macros::Collection;
use futures::TryStreamExt;
use pretty_assertions::assert_eq;
use serde::{Deserialize, Serialize};
use serde_json::json;
use sqlx::Sqlite;
use tower::ServiceExt;

#[derive(Collection, Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct Todo {
    pub title: String,
    pub done: bool,
    pub description: Option<String>,
}

#[tokio::test]
async fn axum_router() {
    let pool = Sqlite::connect_in_memory().await;

    let schema = BuilderPattern::default()
        .build_component(to_migrate(Sqlite))
        .build_component(to_json_client(pool.clone()))
        .start()
        .add_collection(todo)
        .finish();

    schema.0.migrate(pool.clone()).await;

    // axum works
    let router: Router<()> = Router::new()
        .route("/hi", get(|| async { Json(json!(2)) }))
        .nest("/api", schema.1.unwrap().as_router());

    let req = Request::builder()
        .uri("/hi")
        .body(Json(json!({})).into_response())
        .unwrap();

    let res = String::from_utf8(
        router
            .clone()
            .oneshot(req)
            .await
            .unwrap()
            .into_body()
            .into_data_stream()
            .try_collect::<Vec<Bytes>>()
            .await
            .unwrap()
            .into_iter()
            .flatten()
            .collect::<Vec<_>>(),
    )
    .unwrap();

    let tobe = json!(2).to_string();

    assert_eq!(res, tobe);
}
