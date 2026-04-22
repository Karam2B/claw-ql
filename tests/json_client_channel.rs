#![allow(unused)]
#![warn(unused_must_use)]

use std::io::Write;

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

#[tokio::test]
async fn test_json_client() {
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
        "INSERT INTO Category (title) VALUES 
        ('category_1'), ('category_2'), ('category_3');
        ",
    )
    .execute(&pool)
    .await
    .unwrap();

    sqlx::query(
        "INSERT INTO Todo
            (title, done, description, fk_category_def)
        VALUES
            ('first_todo', true, 'description_1', 1),
            ('second_todo', false, 'description_2', NULL),
            ('third_todo', true, 'description_3', 1),
            ('fourth_todo', false, 'description_4', NULL);
    ",
    )
    .execute(&pool)
    .await
    .unwrap();

    let all =
        sqlx::query("SELECT * FROM Todo LEFT JOIN Category ON Todo.fk_category_def = Category.id")
            .fetch_all(&pool)
            .await
            .unwrap();

    let mut conn = pool.begin().await.unwrap();

    let all = Sqlite::fetch_all(
        &mut conn,
        Executable {
            string: "SELECT * FROM Todo LEFT JOIN Category ON Todo.fk_category_def = Category.id",
            arguments: Default::default(),
        },
    )
    .await
    .unwrap();

    conn.commit().await.unwrap();

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
                    "limit": 10,
                    "first_item": null,
                    "order_by": null,
                },
            }))
            .unwrap(),
        )
        .await
        .unwrap();
}
