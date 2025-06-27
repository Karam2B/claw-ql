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
use sqlx::{ColumnIndex, Database, Decode, Encode, prelude::Type};
use std::{convert::Infallible, pin::Pin, sync::Arc, task::Poll};
use tower_service::Service;

impl<S: QueryBuilder> JsonClient<S>
where
    S: QueryBuilder<Output = <S as Database>::Arguments<'static>>,
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
                .route("/{collection}/", get(get_one).post(insert_one))
                .route(
                    "/{collection}/{id}",
                    get(get_one).put(update_one).delete(delete_one),
                )
                .with_state(Arc::new(self))
        };

        router
    }
}

#[derive(Deserialize)]
struct CollectionAndId {
    collection: String,
    id: i64,
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
pub struct SelectOneInput2 {
    #[serde(default)]
    pub filters: Map<String, Value>,
    #[serde(default)]
    pub links: Map<String, Value>,
}

#[inline]
async fn get_all<S: Database>(
    jc: State<Arc<JsonClient<S>>>,
    path: Path<String>,
    body: Json<SelectOneInput2>,
) -> Result<Json<LinkedOutput<Value, Map<String, Value>>>, String>
where
    S: QueryBuilder<Output = <S as Database>::Arguments<'static>>,
    for<'c> &'c mut S::Connection: sqlx::Executor<'c, Database = S>,
    for<'e> i64: Encode<'e, S> + Type<S> + Decode<'e, S>,
    for<'e> &'e str: ColumnIndex<S::Row>,
{
    Ok(Json(
        jc.0.select_one_serialized(SelectOneInput {
            collection: path.0,
            filters: body.0.filters,
            links: body.0.links,
        })
        .await?,
    ))
}
async fn get_one<S: Database>(jc: State<Arc<JsonClient<S>>>, path: Path<CollectionAndId>)
where
    S: QueryBuilder<Output = <S as Database>::Arguments<'static>>,
    for<'c> &'c mut S::Connection: sqlx::Executor<'c, Database = S>,
    for<'e> i64: Encode<'e, S> + Type<S> + Decode<'e, S>,
    for<'e> &'e str: ColumnIndex<S::Row>,
{
}
async fn insert_one<S: Database>(jc: State<Arc<JsonClient<S>>>, path: Path<String>)
where
    S: QueryBuilder<Output = <S as Database>::Arguments<'static>>,
    for<'c> &'c mut S::Connection: sqlx::Executor<'c, Database = S>,
    for<'e> i64: Encode<'e, S> + Type<S> + Decode<'e, S>,
    for<'e> &'e str: ColumnIndex<S::Row>,
{
    todo!()
}
async fn update_one<S: Database>(jc: State<Arc<JsonClient<S>>>, path: Path<CollectionAndId>)
where
    S: QueryBuilder<Output = <S as Database>::Arguments<'static>>,
    for<'c> &'c mut S::Connection: sqlx::Executor<'c, Database = S>,
    for<'e> i64: Encode<'e, S> + Type<S> + Decode<'e, S>,
    for<'e> &'e str: ColumnIndex<S::Row>,
{
    todo!()
}
async fn delete_one<S: Database>(jc: State<Arc<JsonClient<S>>>, path: Path<CollectionAndId>)
where
    S: QueryBuilder<Output = <S as Database>::Arguments<'static>>,
    for<'c> &'c mut S::Connection: sqlx::Executor<'c, Database = S>,
    for<'e> i64: Encode<'e, S> + Type<S> + Decode<'e, S>,
    for<'e> &'e str: ColumnIndex<S::Row>,
{
    todo!()
}
