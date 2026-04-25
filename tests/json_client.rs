#![allow(unused)]
#![warn(unused_must_use)]

use core::fmt;
use std::{io::Write, marker::PhantomData};

use claw_ql::{
    connect_in_memory::ConnectInMemory,
    execute::Executable,
    fix_executor::ExecutorTrait,
    json_client::dynamic_collection::DynamicField,
    json_client_channel::{
        add_collection::{AddCollectionInput, TypeSpec},
        json_client::{JsonClient, JsonClientSetting},
    },
};
use serde_json::{from_value, json};
use sqlx::Sqlite;
use tracing::{
    Event, Subscriber,
    field::{Field, Visit},
};
use tracing_subscriber::{
    Layer, Registry,
    layer::{Context, SubscriberExt},
    util::SubscriberInitExt,
};

#[tokio::test]
async fn test_json_client() {
    tracing_subscriber::registry()
        .with(tracing_subscriber::fmt::layer())
        .try_init()
        .unwrap();

    let pool = Sqlite::connect_in_memory().await;
    let (jc, ex) = JsonClient::new_sqlx_db(pool.clone(), JsonClientSetting::default_setting());

    tokio::spawn(ex.run());

    jc.add_collection(
        from_value(json!({
            "name": "todo",
            "fields": [
                {
                    "name": "title",
                    "type_info": "String",
                    "is_optional": false,
                },
                {
                    "name": "description",
                    "type_info": "String",
                    "is_optional": true,
                },
                {
                    "name": "done",
                    "type_info": "Boolean",
                    "is_optional": false,
                },
            ],
        }))
        .unwrap(),
    )
    .await
    .unwrap();

    jc.add_collection(
        from_value(json!({
            "name": "category",
            "fields": [
                {
                    "name": "title",
                    "type_info": "String",
                    "is_optional": false,
                },
            ],
        }))
        .unwrap(),
    )
    .await
    .unwrap();

    jc.add_link(
        from_value(json!({
            "ty": "optional_to_many",
            "from": "todo",
            "to": "category",
        }))
        .unwrap(),
    )
    .await
    .unwrap();

    jc.add_link(
        from_value(json!({
            "ty": "timestamp",
            "collection": "todo",
        }))
        .unwrap(),
    )
    .await
    .unwrap();

    sqlx::query(
        "
        INSERT INTO Category (title) VALUES 
            ('category_1'), ('category_2'), ('category_3');

        INSERT INTO Todo
            (title, done, description, fk_category_def, created_at, updated_at)
        VALUES
            ('first_todo', true, 'description_1', 1, 'test', 'test'),
            ('second_todo', false, 'description_2', NULL, 'test', 'test'),
            ('third_todo', true, 'description_3', 1, 'test', 'test'),
            ('fourth_todo', false, 'description_4', 2, 'test', 'test'),
            ('fifth_todo', false, 'description_5', NULL, 'test', 'test')
            ;
    ",
    )
    .execute(&pool)
    .await
    .unwrap();

    let s = jc
        .fetch_many(
            from_value(json!({
                "base": "todo",
                "filters": [],
                "links": [
                    { "ty": "optional_to_many", "to": "category", },
                    { "ty": "timestamp" },
                ],
                "pagination": {
                    "limit": 3,
                    "first_item": { "id": 5, "attributes": { "title": "fifth_todo" } },
                    "order_by": [
                        { "col": "title", "direction": "asc" }
                    ],
                },
            }))
            .unwrap(),
        )
        .await
        .unwrap();

    pretty_assertions::assert_eq!(
        serde_json::to_value(s).unwrap(),
        json!({
            "next_item": { "id": 2, "attributes": { "title": "second_todo" } },
            "items": [
                {
                    "id": 5,
                    "attributes": {
                        "title": "fifth_todo",
                        "done": false,
                        "description": "description_5",
                    },
                    "links": [
                        null,
                        { "created_at": "test", "updated_at": "test", }
                    ]
                },
                {
                    "id": 1,
                    "attributes": {
                        "title": "first_todo",
                        "done": true,
                        "description": "description_1",
                    },
                    "links": [
                        { "id": 1, "attributes": { "title": "category_1", } },
                        { "created_at": "test", "updated_at": "test", }
                    ]
                },
                {
                    "id": 4,
                    "attributes": {
                        "title": "fourth_todo",
                        "done": false,
                        "description": "description_4",
                    },
                    "links": [
                        { "id": 2, "attributes": { "title": "category_2", } },
                        { "created_at": "test", "updated_at": "test", }
                    ]
                }
            ]
        })
    );

    panic!("continue here")
}
