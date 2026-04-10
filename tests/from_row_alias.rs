#![allow(unused)]
#![allow(non_camel_case_types)]
#![deny(unused_must_use)]

use std::net::ToSocketAddrs;

use claw_ql::{
    ConnectInMemory,
    from_row::{FromRowAlias, post_alias, pre_alias},
};
use claw_ql_macros::FromRowAlias;
use sqlx::{Row, Sqlite, sqlite::SqliteRow};

pub struct category;

#[derive(PartialEq, Eq, Debug)]
pub struct Category {
    pub title: String,
}

impl<'r> FromRowAlias<'r, SqliteRow> for category {
    fn no_alias(
        &self,
        row: &'r SqliteRow,
    ) -> Result<Self::Data, claw_ql::prelude::from_row_alias::FromRowError> {
        let title = row.try_get("title").map_err(|e| e.try_into().unwrap())?;

        Ok(Category { title })
    }

    fn pre_alias(
        &self,
        row: claw_ql::prelude::from_row_alias::pre_alias<'r, SqliteRow>,
    ) -> Result<Self::Data, claw_ql::prelude::from_row_alias::FromRowError>
    where
        SqliteRow: sqlx::Row,
    {
        Ok(Category {
            title: row
                .0
                .try_get(format!("{}title", row.1).as_str())
                .map_err(|e| e.try_into().unwrap())?,
        })
    }

    fn post_alias(
        &self,
        row: claw_ql::prelude::from_row_alias::post_alias<'r, SqliteRow>,
    ) -> Result<Self::Data, claw_ql::prelude::from_row_alias::FromRowError>
    where
        SqliteRow: sqlx::Row,
    {
        Ok(Category {
            title: row
                .0
                .try_get(format!("title{}", row.1).as_str())
                .map_err(|e| e.try_into().unwrap())?,
        })
    }
}

#[tokio::test]
async fn main() {
    let pool = Sqlite::connect_in_memory().await;

    let row = sqlx::query(
        "
        CREATE TABLE Category ( title TEXT );
        INSERT INTO Category (title) VALUES ('cat_1');
        SELECT title as cat_title, title, title as title_ FROM Category;
    ",
    )
    .fetch_one(&pool)
    .await
    .unwrap();

    let s = category.pre_alias(pre_alias(&row, "cat_")).unwrap();

    assert_eq!(
        s,
        Category {
            title: "cat_1".to_string(),
        },
    );

    let s = category.post_alias(post_alias(&row, "_")).unwrap();

    assert_eq!(
        s,
        Category {
            title: "cat_1".to_string(),
        },
    );

    let s = category.no_alias(&row).unwrap();

    assert_eq!(
        s,
        Category {
            title: "cat_1".to_string(),
        },
    );
}
