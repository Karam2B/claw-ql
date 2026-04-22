#![allow(unused)]
use std::{
    any::Any,
    marker::PhantomData,
    ops::{Deref, DerefMut},
};

use claw_ql::{
    connect_in_memory::ConnectInMemory,
    database_extention::DatabaseExt,
    extentions::common_expressions::StrAliased,
    from_row::{
        FromRowAlias, FromRowData, FromRowError, RowPostAliased, RowPreAliased, RowTwoAliased,
    },
    json_client::dynamic_collection::{DynamicCollection, DynamicField},
    links::{DefaultRelationKey, relation_optional_to_many::OptionalToMany, timestamp::Timestamp},
    operations::{
        Operation, OperationOutput,
        boxed_operation::BoxedOperation,
        fetch_many::{FetchMany, LinkFetchMany, SortOnlyById},
    },
    query_builder::{ManyBoxedExpressions, ManyExpressions},
    select_items_trait_object::{SelectItemsTraitObject, ToImplSelectItems},
    temp_fetch_many_for_vec::JsonLinkFetchMany,
};
use serde::Serialize;
use serde_json::json;
use sqlx::{Database, Sqlite};

#[tokio::test]
async fn test_ref_link() {
    let mut db = Sqlite::connect_in_memory_2().await;

    sqlx::query(
        "
        CREATE TABLE Category ( 
            id INTEGER PRIMARY KEY AUTOINCREMENT,
            title TEXT
        );
        CREATE TABLE Todo ( 
            id INTEGER PRIMARY KEY AUTOINCREMENT,
            title TEXT, 
            done BOOLEAN, 
            description TEXT, 
            fk_category_def INTEGER, FOREIGN KEY (fk_category_def) REFERENCES Category(id)
        );

        INSERT INTO Category (title) VALUES 
        ('category_1'), ('category_2'), ('category_3');

        INSERT INTO Todo
            (title, done, description, fk_category_def)
        VALUES
            ('first_todo', true, 'description_1', 1),
            ('second_todo', false, 'description_2', NULL),
            ('third_todo', true, 'description_3', 2),
            ('fourth_todo', false, 'description_4', 2);
    
    ",
    )
    .execute(&mut db)
    .await
    .unwrap();

    let todo_collection = DynamicCollection::<Sqlite> {
        name: "Todo".to_string(),
        name_lower_case: "todo".to_string(),
        fields: vec![
            DynamicField {
                name: "title".to_string(),
                is_optional: false,
                type_info: Box::new(PhantomData::<String>),
            },
            DynamicField {
                name: "done".to_string(),
                is_optional: false,
                type_info: Box::new(PhantomData::<bool>),
            },
            DynamicField {
                name: "description".to_string(),
                is_optional: true,
                type_info: Box::new(PhantomData::<String>),
            },
        ],
    };

    let category_collection = DynamicCollection::<Sqlite> {
        name: "Category".to_string(),
        name_lower_case: "category".to_string(),
        fields: vec![DynamicField {
            name: "title".to_string(),
            is_optional: false,
            type_info: Box::new(PhantomData::<String>),
        }],
    };

    let optional_to_many = || {
        Box::new(OptionalToMany {
            from: todo_collection.clone(),
            to: category_collection.clone(),
            foriegn_key: DefaultRelationKey,
        }) as Box<dyn JsonLinkFetchMany<Sqlite> + Send>
    };

    let timestamp = || {
        Box::new(Timestamp {
            collection: todo_collection.clone(),
        }) as Box<dyn claw_ql::temp_fetch_many_for_vec::JsonLinkFetchMany<Sqlite> + Send>
    };

    let result = FetchMany {
        base: todo_collection.clone(),
        wheres: (),
        links: optional_to_many(),
        cursor_order_by: SortOnlyById,
        cursor_first_item: None::<(i64, ())>,
        limit: 10,
    };

    let output = Operation::<Sqlite>::exec_operation(result, &mut db).await;

    pretty_assertions::assert_eq!(
        serde_json::to_value(output).unwrap(),
        json!({
            "items": [
                {
                    "id": 1,
                    "attributes": {
                        "title": "first_todo",
                        "done": true,
                        "description": "description_1",

                    },
                    "links": {
                        "id": 1,
                        "attributes": {
                            "title": "category_1",
                        }
                    }
                },
                {
                    "id": 2,
                    "attributes": {
                        "title": "second_todo",
                        "done": false,
                        "description": "description_2",
                    },
                    "links": null
                },
                {
                    "id": 3,
                    "attributes": {
                        "title": "third_todo",
                        "done": true,
                        "description": "description_3",
                    },
                    "links": {
                        "id": 2,
                        "attributes": {
                            "title": "category_2",
                        }
                    }
                },
                {
                    "id": 4,
                    "attributes": {
                        "title": "fourth_todo",
                        "done": false,
                        "description": "description_4",
                    },
                    "links": {
                        "id": 2,
                        "attributes": {
                            "title": "category_2",
                        }
                    }
                }
            ],
            "next_item": null,
        })
    );

    let result = FetchMany {
        base: todo_collection.clone(),
        wheres: (),
        links: vec![optional_to_many(), timestamp()],
        cursor_order_by: SortOnlyById,
        cursor_first_item: None::<(i64, ())>,
        limit: 10,
    };

    todo!("multiple links");

    // let output = Operation::<Sqlite>::exec_operation(result, &mut db).await;

    // pretty_assertions::assert_eq!(
    //     serde_json::to_value(output).unwrap(),
    //     json!({
    //         "items": [
    //             {
    //                 "id": 1,
    //                 "attributes": {
    //                     "title": "first_todo",
    //                     "done": true,
    //                     "description": "description_1",

    //                 },
    //                 "links": {
    //                     "id": 1,
    //                     "attributes": {
    //                         "title": "category_1",
    //                     }
    //                 }
    //             },
    //             {
    //                 "id": 2,
    //                 "attributes": {
    //                     "title": "second_todo",
    //                     "done": false,
    //                     "description": "description_2",
    //                 },
    //                 "links": null
    //             },
    //             {
    //                 "id": 3,
    //                 "attributes": {
    //                     "title": "third_todo",
    //                     "done": true,
    //                     "description": "description_3",
    //                 },
    //                 "links": {
    //                     "id": 2,
    //                     "attributes": {
    //                         "title": "category_2",
    //                     }
    //                 }
    //             },
    //             {
    //                 "id": 4,
    //                 "attributes": {
    //                     "title": "fourth_todo",
    //                     "done": false,
    //                     "description": "description_4",
    //                 },
    //                 "links": {
    //                     "id": 2,
    //                     "attributes": {
    //                         "title": "category_2",
    //                     }
    //                 }
    //             }
    //         ],
    //         "next_item": null,
    //     })
    // );
}
