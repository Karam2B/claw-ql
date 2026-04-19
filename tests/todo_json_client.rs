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

#[claw_ql_macros::skip]
async fn concept_check_unique_filter() {
    let mut conn = Sqlite::connect_in_memory_2().await;

    let todo = DynamicCollection {
        name: "Todo".to_string(),
        name_lower_case: "todo".to_string(),
        fields: vec![
            DynamicField {
                name: "title".to_string(),
                is_optional: false,
                type_info: Box::new(PhantomData::<String>) as Box<dyn SqlxTypeIdent<Sqlite>>,
            },
            DynamicField {
                name: "done".to_string(),
                is_optional: false,
                type_info: Box::new(PhantomData::<bool>) as Box<dyn SqlxTypeIdent<Sqlite>>,
            },
            DynamicField {
                name: "description".to_string(),
                is_optional: true,
                type_info: Box::new(PhantomData::<String>) as Box<dyn SqlxTypeIdent<Sqlite>>,
            },
        ],
    };

    ExpressionAsOperation::exec_operation(todo.statments(), &mut conn).await;

    sqlx::query(
        "
        INSERT INTO Todo (title, done, description) VALUES 
            ('first_todo', true, 'description_1'), 
            ('second_todo', false, 'description_2'), 
            ('third_todo', true, 'description_3');
        ",
    )
    .execute(&mut conn)
    .await
    .unwrap();

    // pub enum SupportedWhere {
    //     ColEq(String, JsonValue),
    // }

    // impl SupportedFilter {
    //     pub fn safety_check(&self, base: &DynamicCollection<Sqlite>) -> Result<(), String> {
    //         match self {
    //             SupportedFilter::ColEq(col, _) => {
    //                 if col == "id" || base.fields.iter().any(|f| f.name == *col) {
    //                     Ok(())
    //                 } else {
    //                     Err(format!("{} does not exist in the collection", col))
    //                 }
    //             }
    //         }
    //     }
    // }

    // impl IsUniqueFilter<DynamicCollection<Sqlite>> for SupportedFilter {
    //     fn is_unique(&self, base: &DynamicCollection<Sqlite>) -> bool {
    //         match self {
    //             SupportedFilter::ColEq(col, _) => col == "id",
    //             _ => false,
    //         }
    //     }
    // }

    // impl OpExpression for SupportedFilter {}
    // impl Expression<'static, Sqlite> for SupportedFilter {
    //     fn expression(self, ctx: &mut QueryBuilder<'static, Sqlite>) {
    //         match self {
    //             SupportedFilter::ColEq(col, value) => {
    //                 ctx.sanitize(&col);
    //                 ctx.syntax(&" = ");
    //                 ctx.bind(value);
    //             }
    //         }
    //     }
    // }

    let s = Operation::<Sqlite>::exec_operation(
        UpdateOne {
            partial: json!({
                "title": ["set", "new_title"],
                "done": ["keep"],
                "description": ["keep"],
            }),
            wheres: vec![{
                let s = SupportedWhere::ColEq("id".to_string(), 1.into());
                s.safety_check(&cl).unwrap();
                s
            }],
            handler: cl,
            links: (),
        }
        .safety_check()
        .unwrap(),
        &mut conn,
    )
    .await;

    pretty_assertions::assert_eq!(
        to_value(s).unwrap(),
        json!({
            "id": 1,
            "attributes": {
                "title": "new_title",
                "done": true,
                "description": "description_1",
            },
            "links": null,
        })
    );
}

#[claw_ql_macros::skip]
async fn concept_test_json_client_supported_links() {
    use tokio::sync::RwLock as TrwLock;

    let mut conn = Sqlite::connect_in_memory_2().await;

    let todo_cl = DynamicCollection {
        name: "Todo".to_string(),
        name_lower_case: "todo".to_string(),
        fields: vec![
            DynamicField {
                name: "title".to_string(),
                is_optional: false,
                type_info: Box::new(PhantomData::<String>) as Box<dyn SqlxTypeHandler<Sqlite>>,
            },
            DynamicField {
                name: "done".to_string(),
                is_optional: false,
                type_info: Box::new(PhantomData::<bool>) as Box<dyn SqlxTypeHandler<Sqlite>>,
            },
            DynamicField {
                name: "description".to_string(),
                is_optional: true,
                type_info: Box::new(PhantomData::<String>) as Box<dyn SqlxTypeHandler<Sqlite>>,
            },
        ],
    };

    let category_cl = DynamicCollection {
        name: "Category".to_string(),
        name_lower_case: "category".to_string(),
        fields: vec![DynamicField {
            name: "title".to_string(),
            is_optional: false,
            type_info: Box::new(PhantomData::<String>) as Box<dyn SqlxTypeHandler<Sqlite>>,
        }],
    };

    let sudo_jc = JsonClient {
        collections: HashMap::from([
            ("todo".to_string(), Arc::new(TrwLock::new(todo_cl.clone()))),
            (
                "category".to_string(),
                Arc::new(TrwLock::new(category_cl.clone())),
            ),
        ]),
        migrations: vec![],
        options: JsonClientOption::default_setting(),
        pool: Sqlite::connect_in_memory().await,
        links: LinkInformations {
            optional_to_many: HashSet::from([("todo".to_string(), "category".to_string())]),
        },
    };

    ExpressionAsOperation(todo_cl.statments())
        .exec_operation(&mut conn)
        .await;
    ExpressionAsOperation(category_cl.statments())
        .exec_operation(&mut conn)
        .await;
    ExpressionAsOperation(
        OptionalToMany {
            from: todo_cl.clone(),
            to: category_cl.clone(),
            foriegn_key: "default".to_string(),
        }
        .statments(),
    )
    .exec_operation(&mut conn)
    .await;

    sqlx::query(
        "
        INSERT INTO Category (title) VALUES ('category_1'), ('category_2'), ('category_3');
        INSERT INTO Todo (title, done, description, fk_todo_category_default) VALUES 
            ('first_todo', true, 'description_1', 1), 
            ('second_todo', false, 'description_2', NULL), 
            ('third_todo', true, 'description_3', NULL),
            ('fourth_todo', false, 'description_4', 3),
            ('fifth_todo', true, 'description_5', 1);
        ",
    )
    .execute(&mut conn)
    .await
    .unwrap();

    let s = Operation::<Sqlite>::exec_operation(
        FetchOne {
            base: todo_cl.clone(),
            wheres: (),
            links: OptionalToMany {
                foriegn_key: "default".to_string(),
                from: todo_cl.clone(),
                to: category_cl.clone(),
            },
        },
        &mut conn,
    )
    .await
    .unwrap();

    pretty_assertions::assert_eq!(
        to_value(s).unwrap(),
        json!({
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
                },
            },
        })
    );

    // let jc = JsonClient::from(conn);

    let first = match_and_cast_to_fetch(
        claw_ql::json_client::supported_links_on_fetch_one::on_request(
            json!({
                "ty": "optional_to_many",
                "to": "category",
            }),
            "todo".to_string(),
            &sudo_jc,
        )
        .unwrap(),
        &sudo_jc,
    )
    .await;

    let s = Operation::<Sqlite>::exec_operation(
        FetchOne {
            base: todo_cl.clone(),
            wheres: (),
            links: vec![first],
        },
        &mut conn,
    )
    .await
    .unwrap();

    pretty_assertions::assert_eq!(
        to_value(s).unwrap(),
        json!({
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
        })
    );
}

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

    panic!("this is four: {:?}", all.len());

    let s = jc
        .fetch_many(
            from_value(json!({
                "base": "todo",
                "filters": [],
                "links": [
                    { "ty": "optional_to_many", "to": "category", },
                ],
                "limit": 10,
                "cursor_first_item": null,
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
