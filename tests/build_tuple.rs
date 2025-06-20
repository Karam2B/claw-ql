#![allow(non_camel_case_types)]
#![allow(dead_code)]
#![allow(unused)]
#![warn(unused_must_use)]
use std::marker::PhantomData;

use claw_ql::{builder_pattern::BuilderPattern, links::relation::Relation};
use claw_ql_macros::{Collection, relation};
use serde::{Deserialize, Serialize};
use sqlx::{Sqlite, SqlitePool};

#[derive(Collection, Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct Todo {
    pub title: String,
    pub done: bool,
    pub description: Option<String>,
}

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

pub trait BuildContext {
    type Context;
    fn init_context(&self) -> Self::Context;
}

pub trait AddCollection<Tuple, Next>: BuildContext {
    fn add_col(next: &Next, ctx: &mut Self::Context);
}

pub trait AddLink<Tuple, Next>: BuildContext {
    fn add_link(next: &Next, ctx: &mut Self::Context);
}

pub trait Finish<C>: BuildContext {
    type Result;
    fn finish(self, ctx: Self::Context) -> Self::Result;
}

#[tokio::test]
async fn test() {
    tracing_subscriber::fmt()
        .with_max_level(tracing::Level::DEBUG)
        .init();

    let pool = SqlitePool::connect("sqlite::memory:").await.unwrap();

    let res = {
        BuilderPattern::default()
            .build_mode(experiment(PhantomData))
            .add_collection(todo)
            .add_collection(tag)
            .add_collection(category)
            .add_link(Relation {
                from: todo,
                to: tag,
            })
            .add_link(Relation {
                from: todo,
                to: category,
            })
            .finish()
    };
}

struct experiment<C>(PhantomData<C>);

impl<C> BuildContext for experiment<C> {
    type Context = ();

    fn init_context(&self) -> Self::Context {}
}

impl<T: BuildTuple, N> AddCollection<T, N> for experiment<T> {
    fn add_col(next: &N, ctx: &mut Self::Context) {}
}

impl<T, L, N> AddLink<L, N> for experiment<T> {
    fn add_link(next: &N, ctx: &mut Self::Context) {}
}

impl<C> Finish<C> for experiment<C> {
    type Result = ();
    fn finish(self, ctx: Self::Context) -> Self::Result {}
}
