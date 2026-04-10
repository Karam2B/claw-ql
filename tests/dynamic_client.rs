#![allow(unused)]
#![warn(unused_must_use)]
use claw_ql::{
    ConnectInMemory,
    collections::{FailedToParseBody, FilterError, FilterIsNotApplicableForCollection, LiqFilter},
    expressions::Col,
    filters::by_id_mod::by_id,
    json_client::{
        JsonClient, JsonError,
        add_collection::{AddCollectionBody, FieldInJson, LiqType},
        axum_router_mod::HttpError,
        select_one::{TypeidIsNotRegistered, WithTypeid},
    },
    json_value_cmp::to_value,
    links::{
        EntryIsNotFound, Link, LiqError, LiqLink, LiqLinkExt, RegisteredError, date_mod,
        relation_optional_to_many::optional_to_many_liq,
    },
    migration::MigrationStep,
    operations::{CollectionOutput, LinkedOutput},
    prelude::{Execute, col, stmt::SelectSt},
};
use claw_ql::{JsonValue, json_value_cmp};
use claw_ql_macros::simple_enum;
use hyper::StatusCode;
use serde::{Deserialize, Serialize};
use serde_json::json;
use sqlx::Sqlite;
use std::{collections::HashMap, marker::PhantomData};

#[tokio::test]
async fn dynamic_js_test() {
    let pool = Sqlite::connect_in_memory().await;

    // migrate_on_empty_database(&schema, &pool).await;

    let mut jc = JsonClient {
        collections: Default::default(),
        errors_log: Default::default(),
        links: HashMap::from_iter([
            (
                "claw_ql::date".to_string(),
                Box::new(date_mod::date_liq {
                    collections: Default::default(),
                }) as Box<dyn LiqLinkExt<Sqlite>>,
            ),
            (
                "claw_ql::optional_to_many".to_string(),
                Box::new(optional_to_many_liq {
                    existing_links: Default::default(),
                }) as Box<dyn LiqLinkExt<Sqlite>>,
            ),
        ]),
        migration: Default::default(),
        filter_extentions: HashMap::from_iter([
            // all filters
            (
                "claw_ql::by_id".to_string(),
                Box::new(PhantomData::<by_id>) as Box<dyn LiqFilter<Sqlite>>,
            ),
        ]),
        type_extentions: HashMap::from_iter([
            (
                "core::bool".to_string(),
                Box::new(PhantomData::<bool>) as Box<dyn LiqType<Sqlite>>,
            ),
            (
                "core::i32".to_string(),
                Box::new(PhantomData::<i32>) as Box<dyn LiqType<Sqlite>>,
            ),
            (
                "core::string".to_string(),
                Box::new(PhantomData::<String>) as Box<dyn LiqType<Sqlite>>,
            ),
        ]),
        error_count: 0.into(),
        db: pool,
    };

    let res = jc
        .add_collection(AddCollectionBody {
            name: "todo".to_string(),
            fields: HashMap::from_iter([
                (
                    "title".to_string(),
                    FieldInJson {
                        typeid: "core::string".to_string(),
                        optional: false,
                    },
                ),
                (
                    "done".to_string(),
                    FieldInJson {
                        typeid: "core::bool".to_string(),
                        optional: false,
                    },
                ),
                (
                    "description".to_string(),
                    FieldInJson {
                        typeid: "core::string".to_string(),
                        optional: true,
                    },
                ),
            ]),
        })
        .await;

    let res = jc
        .add_collection(AddCollectionBody {
            name: "category".to_string(),
            fields: HashMap::from_iter([(
                "title".to_string(),
                FieldInJson {
                    typeid: "core::string".to_string(),
                    optional: false,
                },
            )]),
        })
        .await;

    pretty_assertions::assert_eq!(Ok(()), res);

    create_link(
        &mut jc,
        serde_json::from_value(json!({
            "$typeid": "claw_ql::date", "base": "todo"
        }))
        .unwrap(),
    )
    .await
    .unwrap();

    create_link(
        &mut jc,
        serde_json::from_value(json!({
            "$typeid": "claw_ql::optional_to_many",
            "base": "todo",
            "to": "category",
        }))
        .unwrap(),
    )
    .await
    .unwrap();

    sqlx::query(
        r#"
            INSERT INTO Category (title) VALUES
                ('cat_1'),
                ('cat_2'),
                ('cat_3');
            INSERT INTO Todo (title, done, description, todo_category_default) VALUES
                ('todo_1', 1, null, null),
                ('todo_2', 0, "describtion 2", 3),
                ('todo_3', 1, null, null),
                ('todo_4', 0, null, null),
                ('todo_5', 1, null, null);
            "#,
    )
    .execute(&jc.db)
    .await
    .unwrap();

    let res = select_one(
        &jc,
        serde_json::from_value(json!({
            "collection": "todo",
            "filters": [ { "$typeid": "claw_ql::by_id", "id": 2 } ],
            "links": [
                { "$typeid": "claw_ql::optional_to_many", "to": "category", "id": "default" },
                { "$typeid": "claw_ql::date" }
            ],
        }))
        .unwrap(),
    )
    .await
    .unwrap();

    pretty_assertions::assert_eq!(
        json_value_cmp::json!({
            "id": 2,
            "attr": { "title": "todo_2", "done": false, "description": "describtion 2" },
            "links": [
                { "id": 3, "attr": { "title": "cat_3" } },
                { "created_at": no_cmp, "updated_at": no_cmp},
            ]
        }),
        serde_json::to_value(res).unwrap(),
    );
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
pub struct SelectOneInput {
    pub collection: String,
    #[allow(unused)]
    #[serde(default)]
    pub filters: Vec<WithTypeid>,
    #[serde(default)]
    pub links: Vec<WithTypeid>,
}

#[simple_enum]
#[derive(Debug)]
pub enum SelectOneError {
    CollectionIsNotRegistered,
    FilterIsNotApplicableForCollection,
    FailedToParseBody,
    TypeidIsNotRegistered,
    RegisteredError,
    EntryIsNotFound,
}

claw_ql::links::liq_error_macros::is_subset_of!(SelectOneError);
claw_ql::collections::filter_error_macros::is_subset_of!(SelectOneError);

#[allow(non_camel_case_types)]
#[simple_enum]
#[derive(Debug)]
pub enum Bar {
    String,
    i32,
}

async fn select_one(
    jc: &JsonClient<Sqlite>,
    input: SelectOneInput,
) -> Result<LinkedOutput<serde_json::Value, Vec<serde_json::Value>>, SelectOneError> {
    let cl = jc
        .collections
        .get(&input.collection)
        .ok_or(CollectionIsNotRegistered(input.collection))?;

    let mut st = SelectSt::<Sqlite>::init(cl.table_name_js());

    for WithTypeid { typeid, rest } in input.filters.iter() {
        let filter = if let Some(found) = jc.filter_extentions.get(typeid) {
            found
        } else {
            let existing_filters = jc.filter_extentions.keys().collect::<Vec<_>>();

            return Err(SelectOneError::TypeidIsNotRegistered(
                TypeidIsNotRegistered {
                    requested: typeid.to_string(),
                    to_impl: "filter".to_string(),
                },
            ));
        };

        filter.on_select(rest.clone(), &**cl, &mut st)?;
        // st.where_("");
    }

    let mut links = vec![];
    for WithTypeid { typeid, rest } in input.links {
        let s = jc.links.get(&typeid).ok_or(TypeidIsNotRegistered {
            requested: typeid,
            to_impl: "Link".to_string(),
        })?;

        let s = s.on_select_one(&**cl, rest)?;
        links.push(s)
    }

    #[rustfmt::skip]
    st.select(
        col("id").
        table(cl.table_name_js()).
        alias("local_id")
    );

    cl.on_select(&mut st);

    for link in links.iter_mut() {
        link.on_select(&mut st);
    }

    let res = st
        .fetch_one(&jc.db, |r| {
            use sqlx::Row;
            let id: i64 = r.get("local_id");
            let attr = cl.from_row_scoped(&r);

            for link in links.iter_mut() {
                link.from_row(&r);
            }

            Ok(CollectionOutput {
                id,
                attributes: attr,
            })
        })
        .await;

    let mut res = match res {
        Err(sqlx::Error::RowNotFound) => Err(EntryIsNotFound {
            filters: serde_json::to_value(input.filters).unwrap(),
        })?,
        Err(err) => panic!("bug: {err}"),
        Ok(ok) => ok,
    };

    for link in links.iter_mut() {
        link.sub_op(jc.db.clone()).await
    }
    let links = links.into_iter().map(|e| e.take()).collect::<Vec<_>>();

    Ok(LinkedOutput {
        id: res.id,
        attributes: res.attr,
        link: links,
    })
}

#[derive(Debug)]
pub struct CollectionIsNotRegistered(String);

impl HttpError for CollectionIsNotRegistered {
    fn status_code(&self) -> hyper::StatusCode {
        hyper::StatusCode::BAD_REQUEST
    }
}

impl HttpError for SelectOneError {
    fn status_code(&self) -> hyper::StatusCode {
        select_one_error_auto_match!(self, status_code)
    }
    fn sub_code(&self) -> Option<&'static str> {
        select_one_error_auto_match!(self, sub_code)
    }
    fn sub_message(&self) -> Option<String> {
        select_one_error_auto_match!(self, sub_message)
    }
}

#[derive(Deserialize)]
pub struct CreateLinkInput {
    base: String,
    #[serde(rename = "$typeid")]
    typeid: String,
    #[serde(flatten)]
    rest: serde_json::Value,
}

#[derive(Debug)]
pub struct BaseCollectionIsNotRegistered(String);

#[simple_enum]
#[derive(Debug)]

pub enum CreateLinkError {
    RegisteredError,
    FailedToParseBody,
    TypeidIsNotRegistered,
    BaseCollectionIsNotRegistered,
}

impl From<LiqError> for CreateLinkError {
    fn from(value: LiqError) -> Self {
        match value {
            LiqError::RegisteredError(v) => CreateLinkError::RegisteredError(v),
            LiqError::FailedToParseBody(v) => CreateLinkError::FailedToParseBody(v),
        }
    }
}

async fn create_link(
    jc: &mut JsonClient<Sqlite>,
    input: CreateLinkInput,
) -> Result<JsonValue, CreateLinkError> {
    let mut link = if let Some(s) = jc.links.get_mut(&input.typeid) {
        s
    } else {
        Err(TypeidIsNotRegistered {
            requested: input.typeid.to_string(),
            to_impl: "Link".to_string(),
        })?
    };

    let base = jc
        .collections
        .get(&input.base)
        .ok_or_else(|| BaseCollectionIsNotRegistered(input.base))
        .cloned()?;

    let migration = link.create_link(&jc.collections, &base, input.rest)?;

    for sql in migration.1.iter() {
        sqlx::query(&sql).execute(&jc.db).await.unwrap();
    }

    jc.migration.push(MigrationStep {
        version: 0,
        sql: migration.1,
    });

    Ok(migration.0)
}
