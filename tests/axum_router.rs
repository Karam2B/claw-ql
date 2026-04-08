#![allow(unused)]
#![deny(unused_must_use)]
macro_rules! dated {
    ($collection:ident) => {};
}

use claw_ql::{
    json_client::{
        add_collection::{AddCollectionBody, LiqType},
        axum_router_mod::{axum_router, axum_router_dynamic},
    },
    migration::migrate_on_empty_database,
};
use std::{collections::HashMap, marker::PhantomData, mem, ops::Not, sync::Arc};

use axum::{
    Json, Router,
    body::Bytes,
    extract::Path,
    http::HeaderValue,
    routing::{get, post},
};
use claw_ql::{
    ConnectInMemory, Schema,
    filters::by_id_mod::by_id,
    json_client::JsonClient,
    links::{relation::Relation, set_id::SetId, set_new::SetNew},
    migration::{LiqOnMigrate, OnMigrate},
    operations::{
        CollectionOutput, LinkedOutput, delete_one_op::delete_one, insert_one_op::insert_one,
        select_one_op::select_one, update_one_op::update_one,
    },
    update_mod::update,
};
use claw_ql_macros::{Collection, relation};
use futures::TryStreamExt;
use hyper::{HeaderMap, Request};
use serde::{Deserialize, Serialize};
use serde_json::{Value, json};
use sqlx::{Pool, Sqlite, SqlitePool};
use tower::ServiceExt;

async fn test_json_router(
    router: Router<()>,
    uri: &str,
    mut headers: HeaderMap<HeaderValue>,
    body: Value,
) -> Result<Value, String> {
    let res = router
        .oneshot({
            let mut req = Request::builder()
                .uri(uri)
                .header(hyper::header::CONTENT_TYPE, "application/json");
            mem::swap(req.headers_mut().unwrap(), &mut headers);
            req.body(axum::body::Body::from(body.to_string()))
                .map_err(|e| format!("body should be valid utf8 json{}", e.to_string()))?
        })
        .await
        // .expect("one shot failed")
        .map_err(|e| format!("one shot failed {}", e.to_string()))?
        .into_body()
        .into_data_stream()
        .try_collect::<Vec<Bytes>>()
        .await
        .map_err(|e| format!("return should be array of bytes {}", e.to_string()))?
        .into_iter()
        .flatten()
        .collect::<Vec<_>>();

    Ok(serde_json::from_slice::<Value>(&res).map_err(|e| {
        format!(
            "output of route should be convertable to json {}",
            e.to_string()
        )
    })?)
}

#[derive(Collection, Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct Todo {
    pub title: String,
    pub done: bool,
    pub description: Option<String>,
}

dated!(Todo);

#[derive(Collection, Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct Category {
    pub title: String,
}

#[derive(Collection, Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct Tag {
    pub title: String,
}

relation!(optional_to_many Todo Category);
relation!(many_to_many Todo Tag);

#[tokio::test]
async fn axmu_router_test() {
    let pool = Sqlite::connect_in_memory().await;

    let schema = Schema {
        collections: (todo,),
        links: (),
    };

    migrate_on_empty_database(&schema, &pool).await;

    insert_one(Todo {
        title: "todo_1".to_string(),
        done: false,
        description: None,
    })
    .exec_op(&pool)
    .await;

    let jc = Arc::new(JsonClient::from_schema(schema, pool.clone()));

    let router = axum_router(jc);

    let res = test_json_router(router, "/select_one_todo", Default::default(), json!(null))
        .await
        .unwrap();

    pretty_assertions::assert_eq!(
        res,
        json!({
            "id": 1,
            "attr": { "title": "todo_1", "done": false, "description": null },
            "links": {}
        })
    );
}
