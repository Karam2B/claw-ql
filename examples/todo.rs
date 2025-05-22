use std::marker::PhantomData;

use claw_ql::operations::select_one::{SelectOneOutput, get_one};
use claw_ql_macros::Collection;
use sqlx::SqlitePool;

#[derive(Collection, Debug, PartialEq)]
pub struct Todo {
    pub title: String,
    pub done: bool,
    pub description: Option<String>,
}

#[derive(Collection, Debug, PartialEq)]
pub struct Category {
    pub title: String,
}

#[derive(Collection, Debug, PartialEq)]
pub struct Tag {
    pub title: String,
}

#[tokio::main]
async fn main() {
    let pool = SqlitePool::connect("sqlite::memory:").await.unwrap();

    let op = get_one(PhantomData::<Todo>);

    let res = op.exec_op(pool).await;

    let expected = Some(SelectOneOutput {
        id: 0,
        attr: Todo {
            title: String::from("description"),
            done: false,
            description: Some(String::from("description")),
        },
        links: (),
    });

    pretty_assertions::assert_eq!(res, expected,)
}
