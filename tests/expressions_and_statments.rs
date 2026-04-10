#![allow(unused)]
#![warn(unused_must_use)]
#![allow(non_camel_case_types)]

use std::{marker::PhantomData, ops::Not};

use axum::response;
use claw_ql::{
    Buildable, ConnectInMemory, EncodeExtention, Expression, SanitzingMechanisim, SelectListItem,
    WhereItem, direct_builder::direct_bind, execute::Executable, expressions::*,
    sanitize::SanitizeAndHardcode, statements::select_st::SelectSt,
};
use sqlx::{Executor, Sqlite};

#[test]
fn select() {
    let mut st = SelectSt::init("todo", direct_bind::new(Sqlite));
    let ref_str = String::from("toeq");

    st.select("item");
    st.select(col("description").table("todo").alias("todo_desc"));
    st.where_(col_to_eq {
        select: "description",
        to_eq: 5,
    });
    st.where_(col("item").table("todo").to_eq(&ref_str));

    // cannot move a referenced value (lifetime '1 held by `to_eq`)
    // drop(ref_str);

    let (sql, _) = st.build();

    drop(ref_str);

    pretty_assertions::assert_eq!(
        sql,
        format!(
            "SELECT {s1}, {s2} FROM 'todo' WHERE {w1} AND {w2};",
            s1 = "'item'",
            s2 = "'todo'.'description' AS 'todo_desc'",
            w1 = "'description' = $1",
            w2 = "'todo'.'item' = $2"
        )
    );
}

#[tokio::test]
async fn select_and_exec() {
    let mut st = SelectSt::init("todo", direct_bind::new(Sqlite));
    let ref_str = String::from("toeq");

    st.select("item");
    st.select(col("description").table("todo").alias("todo_desc"));
    st.where_(col_to_eq {
        select: "description",
        to_eq: 5,
    });
    st.where_(col("item").table("todo").to_eq(&ref_str));

    // cannot move a referenced value (lifetime '1 held by `to_eq`)
    // drop(ref_str);

    let (string, arguments) = st.build();

    let pool = Sqlite::connect_in_memory().await;
    let s = Executor::execute(
        &pool,
        Executable {
            string: &string,
            arguments,
            db: PhantomData,
        },
    )
    .await;

    if (s.is_err().not()) {
        panic!("there is not table such as todo")
    }
}
