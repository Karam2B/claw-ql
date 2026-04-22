#![allow(unused)]
#![warn(unused_must_use)]
use claw_ql::connect_in_memory::ConnectInMemory;
use claw_ql::database_extention::DatabaseExt;
use claw_ql::debug_row::DebugRow;
use claw_ql::execute::Executable;
use claw_ql::expressions::col_eq;
use claw_ql::fix_executor::ExecutorTrait;
use claw_ql::from_row::FromRowAlias;
use claw_ql::into_infer_from_phantom::IntoInferFromPhantom;
use claw_ql::json_client::add_collection::{AddCollectionInput, TypeSpec};
use claw_ql::json_client::add_link::AddLinkInput;
use claw_ql::json_client::dynamic_collection::DynamicField;
use claw_ql::json_client::fetch_many::{FetchManyInput, SupportedLinkFetchMany};
use claw_ql::json_client::json_client::JsonClient;
use claw_ql::json_value_cmp::to_value;
use claw_ql::links::relation_optional_to_many::OptionalToMany;
use claw_ql::on_migrate::OnMigrate;
use claw_ql::operations::execute_expression::ExpressionAsOperation;
// use claw_ql::operations::fetch_one::FetchOne;
// use claw_ql::operations::insert_one::InsertOne;
// use claw_ql::operations::update_one::UpdateOne;
use claw_ql::debug_row;
use claw_ql::operations::{Operation, SafeOperation};
use claw_ql::query_builder::functional_expr::ManyImplPossible;
use claw_ql::query_builder::{
    Expression, IsOpExpression, ManyExpressions, OpExpression, StatementBuilder,
};
use claw_ql::statements::insert_statement::{InsertStatement, One};
use claw_ql::statements::select_statement::SelectStatement;
use claw_ql::statements::update_statement::UpdateStatement;
use claw_ql::update_mod::Update;
use claw_ql_macros::simple_enum;
use serde_json::json;
use serde_json::{Value as JsonValue, from_value};
use sqlx::sqlite::SqliteRow;
use sqlx::{FromRow, Row, Sqlite, query};
use std::collections::{HashMap, HashSet};
use std::marker::PhantomData;
use std::rc::Rc;
use std::sync::Arc;

#[tokio::test]
async fn test_json_client() {
    let mut jc = JsonClient::<Sqlite>::from(Sqlite::connect_in_memory().await);

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
    .execute(&jc.pool)
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
    .execute(&jc.pool)
    .await
    .unwrap();

    let all =
        sqlx::query("SELECT * FROM Todo LEFT JOIN Category ON Todo.fk_category_def = Category.id")
            .fetch_all(&jc.pool)
            .await
            .unwrap();

    let mut conn = jc.pool.begin().await.unwrap();

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

    pretty_assertions::assert_eq!(
        to_value(s).unwrap(),
        json!({
            "items": [
                {
                    "id": 1,
                    "attributes": {
                        "title": "first_todo",
                        "done": true,
                        "description": "description_1",
                    },
                    "links": [{
                        "id": 1,
                        "attributes": {
                            "title": "category_1",
                        },
                    }],
                }
            ],
            "next_item": null,
        })
    );

    // jc.insert_one(InsertOneInput {
    //     base: "todo".to_string(),
    //     data: json!({
    //         "title": "first_todo",
    //         "done": true,
    //         "description": "description_1",
    //     }),
    //     links: vec![],
    // })
    // .await
    // .unwrap();

    // let fetch_result = jc
    //     .fetch_one(FetchOneInput {
    //         base: "todo".to_string(),
    //         wheres: vec![],
    //         links: vec![json!({
    //             "ty": "optional_to_many",
    //             "to": "category",
    //         })],
    //     })
    //     .await
    //     .unwrap();

    // pretty_assertions::assert_eq!(
    //     to_value(fetch_result).unwrap(),
    //     json!({
    //         "id": 1,
    //         "attributes": {
    //             "title": "first_todo",
    //             "done": true,
    //             "description": "description_1",
    //         },
    //         "links": [null],
    //     })
    // );

    // jc.update_one(UpdateOneInput {
    //     base: "todo".to_string(),
    //     partial: json!({
    //         "title": ["set", "new_title"]
    //     }),
    //     links: vec![],
    //     filters: vec![SupportedFilter::<JsonValue>::ColEq(col_eq {
    //         col: "id".to_string(),
    //         eq: 1.into(),
    //     })],
    // })
    // .await
    // .unwrap();

    // jc.delete_one(DeleteOneInput {
    //     base: "todo".to_string(),
    //     links: vec![],
    //     filters: vec![SupportedFilter::<JsonValue>::ColEq(col_eq {
    //         col: "id".to_string(),
    //         eq: 1.into(),
    //     })],
    // })
    // .await
    // .unwrap();
}
