//! todo list
//!
//! - [x] add where clase
//! - [x] make sql macro
//! - [x] clear out junk out of codebase
//! - [x] basic migrate function
//! - [ ] make readme
//!
//!
//! - [ ] MAJOR REALEASE
//!
//! - [ ] create internal ticket system!
//! - [ ] figure out nested where op
//! - [ ] figure out nested links
//! - [ ] json_client create link
//! - [ ] json_client modify link
//! - [ ] add many_to_many link type
//! - [ ] add one_to_many link type
//! - [ ] add date link type
//! - [ ] add fetch many operation
//! - [ ] add insert operation
//! - [ ] add update operation
//! - [ ] add delete operation
//! - [ ] add more where operations

pub mod collections;
pub mod connect_in_memory;
pub mod database_extention;
pub mod dyn_vec;
pub mod execute;
pub mod expressions;
pub mod extend_sqlite;
pub mod extentions;
pub mod from_row;
pub mod json_client;
pub mod json_client_channel;
pub mod json_value_cmp;
#[cfg(test)]
pub mod lifetime_guide;
pub mod links;
pub mod on_migrate;
pub mod operations;
pub mod prelude;
pub mod query_builder;
pub mod row_utils;
pub mod schema;
pub mod singleton;
pub mod statements;
pub mod test_module;
pub mod tuple_trait;
pub mod update_mod;
pub mod valid_syntax;
pub mod macros {
    pub use claw_ql_macros::*;
}

pub mod select_items_trait_object {
    use crate::{
        extentions::common_expressions::Aliased,
        from_row::{
            FromRowAlias, FromRowData, FromRowError, RowPostAliased, RowPreAliased, RowTwoAliased,
        },
        query_builder::{ManyBoxedExpressions, functional_expr::ManyFlat},
    };
    use sqlx::Database;
    use std::any::Any;

    pub trait SelectItemsTraitObject<S, CastFromRowResult>: Send {
        fn str_alias_erase(&self, alias: &'static str) -> Box<dyn ManyBoxedExpressions<S> + Send>;
        fn num_alias_erase(
            &self,
            num: usize,
            alias: &'static str,
        ) -> Box<dyn ManyBoxedExpressions<S> + Send>;
        fn no_alias_2<'r>(&self, row: &'r S::Row) -> Result<Box<dyn Any + Send>, FromRowError>
        where
            S: Database;
        fn pre_alias_2<'r>(
            &self,
            row: RowPreAliased<'r, S::Row>,
        ) -> Result<Box<dyn Any + Send>, FromRowError>
        where
            S: Database,
            S::Row: sqlx::Row;
        fn post_alias_2<'r>(
            &self,
            row: RowPostAliased<'r, S::Row>,
        ) -> Result<Box<dyn Any + Send>, FromRowError>
        where
            S: Database,
            S::Row: sqlx::Row;
        fn two_alias_2<'r>(
            &self,
            row: RowTwoAliased<'r, S::Row>,
        ) -> Result<Box<dyn Any + Send>, FromRowError>
        where
            S: Database,
            S::Row: sqlx::Row;
    }

    pub struct ToImplSelectItems<Se, CastFromRowResult> {
        pub select_items: Se,
        pub cast_from_row_result: CastFromRowResult,
    }

    impl<Se, S> SelectItemsTraitObject<S, ()> for ToImplSelectItems<Se, ()>
    where
        Se: Send,
        Se: Aliased<Aliased: 'static + Send + ManyBoxedExpressions<S>>,
        Se: Aliased<NumAliased: 'static + Send + ManyBoxedExpressions<S>>,
        Se: for<'r> FromRowAlias<'r, S::Row>,
        Se: FromRowData<RData: Send + 'static>,
        S: Database,
    {
        fn str_alias_erase(&self, alias: &'static str) -> Box<dyn ManyBoxedExpressions<S> + Send> {
            Box::new(self.select_items.aliased(alias))
        }
        fn num_alias_erase(
            &self,
            num: usize,
            alias: &'static str,
        ) -> Box<dyn ManyBoxedExpressions<S> + Send> {
            Box::new(self.select_items.num_aliased(num, alias))
        }
        fn no_alias_2<'r>(&self, row: &'r S::Row) -> Result<Box<dyn Any + Send>, FromRowError> {
            Ok(Box::new(self.select_items.no_alias(row)?))
        }
        fn pre_alias_2<'r>(
            &self,
            row: RowPreAliased<'r, S::Row>,
        ) -> Result<Box<dyn Any + Send>, FromRowError> {
            let ret = self.select_items.pre_alias(row)?;
            Ok(Box::new(ret))
        }
        fn post_alias_2<'r>(
            &self,
            row: RowPostAliased<'r, S::Row>,
        ) -> Result<Box<dyn Any + Send>, FromRowError> {
            Ok(Box::new(self.select_items.post_alias(row)?))
        }
        fn two_alias_2<'r>(
            &self,
            row: RowTwoAliased<'r, S::Row>,
        ) -> Result<Box<dyn Any + Send>, FromRowError> {
            Ok(Box::new(self.select_items.two_alias(row)?))
        }
    }

    impl<'r, S, C> Aliased for Box<dyn SelectItemsTraitObject<S, C> + 'r> {
        type Aliased = Box<dyn ManyBoxedExpressions<S> + Send>;

        fn aliased(&self, alias: &'static str) -> Self::Aliased {
            self.str_alias_erase(alias)
        }

        type NumAliased = Box<dyn ManyBoxedExpressions<S> + Send>;
        fn num_aliased(&self, num: usize, alias: &'static str) -> Self::NumAliased {
            self.num_alias_erase(num, alias)
        }
    }

    impl<'r, S, C> Aliased for Vec<Box<dyn SelectItemsTraitObject<S, C> + 'r>> {
        type Aliased = ManyFlat<Vec<Box<dyn ManyBoxedExpressions<S> + Send>>>;

        fn aliased(&self, alias: &'static str) -> Self::Aliased {
            ManyFlat(
                self.iter()
                    .enumerate()
                    .map(|(i, each)| each.num_alias_erase(i, alias))
                    .collect::<Vec<_>>(),
            )
        }

        type NumAliased = ManyFlat<Box<dyn ManyBoxedExpressions<S> + Send>>;
        fn num_aliased(&self, _: usize, _: &'static str) -> Self::NumAliased {
            panic!("bug: nesting where it was not expected");
        }
    }

    impl<'r, S> FromRowData for Box<dyn SelectItemsTraitObject<S, ()> + 'r> {
        type RData = Box<dyn Any + Send>;
    }

    impl<'r, 'b, S: Database> FromRowAlias<'r, S::Row> for Box<dyn SelectItemsTraitObject<S, ()> + 'b> {
        fn no_alias(&self, row: &'r S::Row) -> Result<Self::RData, FromRowError> {
            Ok(self.no_alias_2(row)?)
        }
        fn pre_alias(&self, row: RowPreAliased<'r, S::Row>) -> Result<Self::RData, FromRowError> {
            Ok(self.pre_alias_2(row)?)
        }
        fn post_alias(&self, row: RowPostAliased<'r, S::Row>) -> Result<Self::RData, FromRowError> {
            Ok(self.post_alias_2(row)?)
        }

        fn two_alias(&self, row: RowTwoAliased<'r, S::Row>) -> Result<Self::RData, FromRowError>
        where
            S::Row: sqlx::Row,
        {
            Ok(self.two_alias_2(row)?)
        }
    }
    impl<'r, S> FromRowData for Vec<Box<dyn SelectItemsTraitObject<S, ()> + 'r>> {
        type RData = Vec<Box<dyn Any + Send>>;
    }

    impl<'r, 'b, S: Database> FromRowAlias<'r, S::Row>
        for Vec<Box<dyn SelectItemsTraitObject<S, ()> + 'b>>
    {
        fn no_alias(&self, row: &'r S::Row) -> Result<Self::RData, FromRowError> {
            let mut v = vec![];
            for (i, each) in self.iter().enumerate() {
                v.push(each.two_alias(RowTwoAliased {
                    row: row,
                    str_alias: "",
                    num_alias: Some(i),
                })?);
            }
            Ok(v)
        }
        fn pre_alias(&self, row: RowPreAliased<'r, S::Row>) -> Result<Self::RData, FromRowError> {
            let mut v = vec![];
            for (i, each) in self.iter().enumerate() {
                v.push(each.two_alias(RowTwoAliased {
                    row: row.row,
                    str_alias: row.alias,
                    num_alias: Some(i),
                })?);
            }
            Ok(v)
        }
        fn post_alias(&self, _: RowPostAliased<'r, S::Row>) -> Result<Self::RData, FromRowError> {
            panic!("in the process of deprecating this method");
        }

        fn two_alias(&self, _: RowTwoAliased<'r, S::Row>) -> Result<Self::RData, FromRowError>
        where
            S::Row: sqlx::Row,
        {
            panic!("nesting where it was not expected");
        }
    }

    #[cfg(test)]
    mod test {
        use std::marker::PhantomData;

        use crate::{
            connect_in_memory::ConnectInMemory,
            json_client::{
                dynamic_collection::{DynamicCollection, DynamicField},
                fetch_many::extending_link_trait::JsonLinkFetchMany,
            },
            links::{
                DefaultRelationKey, relation_optional_to_many::OptionalToMany, timestamp::Timestamp,
            },
            operations::{Operation, fetch_many::FetchMany},
        };
        use serde_json::json;
        use sqlx::Sqlite;

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
            created_at TEXT NOT NULL DEFAULT CURRENT_TIMESTAMP,
            updated_at TEXT NOT NULL DEFAULT CURRENT_TIMESTAMP,
            fk_category_def INTEGER, FOREIGN KEY (fk_category_def) REFERENCES Category(id)
        );

        

        INSERT INTO Category (title) VALUES 
        ('category_1'), ('category_2'), ('category_3');

        INSERT INTO Todo
            (title, done, description, fk_category_def, created_at, updated_at)
        VALUES
            ('first_todo', true, 'description_1', 1, 'test_0', 'test_1'),
            ('second_todo', false, 'description_2', NULL, 'test_2', 'test_3'),
            ('third_todo', true, 'description_3', 2, 'test_4', 'test_5'),
            ('fourth_todo', false, 'description_4', 2, 'test_6', 'test_7');
    
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
                }) as Box<dyn JsonLinkFetchMany<Sqlite> + Send>
            };

            let result = FetchMany {
                base: todo_collection.clone(),
                wheres: (),
                links: optional_to_many(),
                cursor_order_by: (),
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
                cursor_order_by: (),
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
                            "links": [
                                { "id": 1, "attributes": { "title": "category_1", } },
                                { "created_at": "test_0", "updated_at": "test_1", }
                            ]
                        },
                        {
                            "id": 2,
                            "attributes": {
                                "title": "second_todo",
                                "done": false,
                                "description": "description_2",
                            },
                            "links": [
                                null,
                                { "created_at": "test_2", "updated_at": "test_3", }
                            ]
                        },
                        {
                            "id": 3,
                            "attributes": {
                                "title": "third_todo",
                                "done": true,
                                "description": "description_3",
                            },
                            "links": [
                                { "id": 2, "attributes": { "title": "category_2", } },
                                { "created_at": "test_4", "updated_at": "test_5", }
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
                                { "created_at": "test_6", "updated_at": "test_7", }
                            ]
                        }
                    ],
                    "next_item": null,
                })
            );
        }
    }
}

pub mod sqlx_error_handling {
    use sqlx::{Database, Error};
    pub trait HandleSqlxResult {
        type Ok;
        #[track_caller]
        fn unwrap_sqlx_error<S: Database>(self) -> Self::Ok;
    }

    impl<T> HandleSqlxResult for Result<T, Error> {
        type Ok = T;
        #[track_caller]
        fn unwrap_sqlx_error<S: Database>(self) -> T {
            match self {
                Ok(ok) => return ok,
                Err(Error::Database(e)) => {
                    match (S::NAME, e.code().map(|e| e.to_string())) {
                        ("SQLite", Some(code)) if code == "1" => {
                            panic!("{e:?}, hint: run migration")
                        }
                        (_, _) => panic!("{e:?}"),
                    };
                }
                Err(e) => match &e {
                    Error::RowNotFound => {
                        panic!(
                            "internal bug: claw_ql should have cleared all sqlx error at this point: {:?}",
                            e
                        );
                    }
                    _ => {
                        panic!("database error: {:?}", e);
                    }
                },
            }
        }
    }
}

pub mod debug_row {
    use core::fmt;
    use sqlx::{Column, Row};

    use crate::from_row::{RowPostAliased, RowPreAliased, RowTwoAliased};

    pub struct DebugRow<T>(pub T);
    impl<'r, R> fmt::Debug for DebugRow<RowTwoAliased<'r, R>>
    where
        R: Row,
    {
        fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
            let mut list = f.debug_list();
            list.entry(&"two_alias");
            list.entry(&self.0.str_alias);
            list.entry(&self.0.num_alias);
            for col in Row::columns(self.0.row) {
                list.entry(&Column::name(col));
                list.entry(&sqlx::TypeInfo::name(Column::type_info(col)));
            }
            list.finish()?;
            Ok(())
        }
    }

    impl<'r, R> fmt::Debug for DebugRow<RowPreAliased<'r, R>>
    where
        R: Row,
    {
        fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
            let mut list = f.debug_list();
            list.entry(&"pre_alias");
            list.entry(&self.0.alias);
            for col in Row::columns(self.0.row) {
                list.entry(&Column::name(col));
                list.entry(&sqlx::TypeInfo::name(Column::type_info(col)));
            }
            list.finish()?;
            Ok(())
        }
    }
    impl<'r, R> fmt::Debug for DebugRow<RowPostAliased<'r, R>>
    where
        R: Row,
    {
        fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
            let mut list = f.debug_list();
            list.entry(&"post_alias");
            list.entry(&self.0.alias);
            for col in Row::columns(self.0.row) {
                list.entry(&Column::name(col));
                list.entry(&sqlx::TypeInfo::name(Column::type_info(col)));
            }
            list.finish()?;
            Ok(())
        }
    }
}
pub mod into_infer_from_phantom {
    use std::marker::PhantomData;

    pub trait IntoInferFromPhantom<I> {
        fn into_pd(self, _: PhantomData<I>) -> I;
    }

    impl<F, I> IntoInferFromPhantom<I> for F
    where
        I: From<F>,
    {
        #[inline]
        fn into_pd(self, _: PhantomData<I>) -> I {
            self.into()
        }
    }
}

pub mod liquidify {

    #[macro_export]
    macro_rules! liquidify {
        ($extention:ty as $base:ty) => {
            fn cast_as_box<'box_lifetime, Type: $base + 'box_lifetime>(
                instance: Type,
            ) -> Box<dyn $extention + 'box_lifetime> {
                Box::new(instance) as Box<dyn $extention>
            }

            fn cast_as_rc<'rc_lifetime, Type: $base + 'rc_lifetime>(
                instance: Type,
            ) -> Rc<dyn $extention + 'rc_lifetime> {
                Rc::new(instance) as Rc<dyn $extention>
            }

            fn cast_as_arc<'arc_lifetime, Type: $base + 'arc_lifetime>(
                instance: Type,
            ) -> Arc<dyn $extention + 'arc_lifetime> {
                Arc::new(instance) as Arc<dyn $extention>
            }

            #[allow(unused)]
            #[allow(dead_code)]
            const _BUILD_TIME_CHECKS: () = {
                fn f1<T: $base>(_: T) {}
                fn f2<'any_lifetime>() {
                    let s: Box<dyn $extention + 'any_lifetime> = todo!();
                    f1(s);
                }
            };
        };
    }

    #[macro_export]
    /// sometimes you don't the option of being generic over lifetimes,
    /// especially if you are extending a trait that requires a "non flexible" lifetime.
    /// But sometimes that is good enough.
    macro_rules! liquidify_static {
        ($extention:ty as $base:ty) => {
            fn cast_as_box<Type: $base + 'static>(instance: Type) -> Box<dyn $extention> {
                Box::new(instance) as Box<dyn $extention>
            }

            fn cast_as_rc<Type: $base + 'static>(instance: Type) -> Rc<dyn $extention> {
                Rc::new(instance) as Rc<dyn $extention>
            }

            fn cast_as_arc<Type: $base + 'static>(instance: Type) -> Arc<dyn $extention> {
                Arc::new(instance) as Arc<dyn $extention>
            }

            #[allow(unused)]
            #[allow(dead_code)]
            const _BUILD_TIME_CHECKS: () = {
                fn f1<T: $base>(_: T) {}
                fn f2() {
                    let s: Box<dyn $extention> = todo!();
                    f1(s);
                }
            };
        };
    }
}

pub mod fix_executor {
    use futures::future::BoxFuture;
    use sqlx::{Database, Execute};

    /// this trait collects the implementation I'm interested in
    /// for the operation I implement
    /// + incoorporate "fix_executor" feature, until sqlx::Executor is fixed
    pub trait ExecutorTrait: Database {
        fn fetch_optional<'e, E: 'e + Execute<'e, Self>>(
            conn: &mut Self::Connection,
            execute: E,
        ) -> BoxFuture<'e, Result<Option<Self::Row>, sqlx::Error>>;
        fn fetch_all<'e, E: 'e + Execute<'e, Self>>(
            conn: &mut Self::Connection,
            execute: E,
        ) -> BoxFuture<'e, Result<Vec<Self::Row>, sqlx::Error>>;
        fn execute<'e, E: 'e + Execute<'e, Self>>(
            conn: &mut Self::Connection,
            execute: E,
        ) -> BoxFuture<'e, Result<<Self as Database>::QueryResult, sqlx::Error>>;
    }

    #[cfg(not(feature = "fix_executor"))]
    impl<S> ExecutorTrait for S
    where
        S: Database,
        for<'e> &'e mut S::Connection: sqlx::Executor<'e, Database = S>,
    {
        fn fetch_all<'e, E: 'e + Execute<'e, Self>>(
            conn: &mut Self::Connection,
            execute: E,
        ) -> BoxFuture<'e, Result<Vec<Self::Row>, sqlx::Error>> {
            // this unsafe code is not an issue
            // because the problem is in sqlx::Executor interface

            // Executor2 does the same thing, without unsafe code
            // therefore this will not produce any lifetime issues
            let break_executor = unsafe { &mut *(conn as *mut Self::Connection) };
            sqlx::Executor::fetch_all(break_executor, execute)
        }
        fn execute<'e, E: 'e + Execute<'e, Self>>(
            conn: &mut Self::Connection,
            execute: E,
        ) -> BoxFuture<'e, Result<<Self as Database>::QueryResult, sqlx::Error>> {
            // this unsafe code is not an issue
            // because the problem is in sqlx::Executor interface

            // Executor2 does the same thing, without unsafe code
            // therefore this will not produce any lifetime issues
            let break_executor = unsafe { &mut *(conn as *mut Self::Connection) };
            sqlx::Executor::execute(break_executor, execute)
        }
    }

    #[cfg(feature = "fix_executor")]
    impl<S> ExecutorTrait for S
    where
        S: Database,
        for<'e> &'e mut S::Connection: sqlx::executor_2::Executor2<Database = S>,
    {
        fn fetch_optional<'e, E: 'e + Execute<'e, Self>>(
            conn: &mut Self::Connection,
            execute: E,
        ) -> BoxFuture<'e, Result<Option<Self::Row>, sqlx::Error>> {
            let keep_conn_out = sqlx::executor_2::Executor2::fetch_optional(conn, execute);
            Box::pin(async move { keep_conn_out.await })
        }
        fn fetch_all<'e, E: 'e + Execute<'e, Self>>(
            conn: &mut Self::Connection,
            execute: E,
        ) -> BoxFuture<'e, Result<Vec<Self::Row>, sqlx::Error>> {
            let keep_conn_out = sqlx::executor_2::Executor2::fetch_all(conn, execute);
            Box::pin(async move { keep_conn_out.await })
        }
        fn execute<'e, E: 'e + Execute<'e, Self>>(
            conn: &mut Self::Connection,
            execute: E,
        ) -> BoxFuture<'e, Result<<Self as Database>::QueryResult, sqlx::Error>> {
            let keep_conn_out = sqlx::executor_2::Executor2::execute(conn, execute);
            Box::pin(async move { keep_conn_out.await })
        }
    }
}

pub mod debug_any {
    use core::fmt;
    use std::any::{Any, TypeId};

    pub trait DebugAny: Any {
        fn debug(&self, fmt: &mut fmt::Formatter<'_>) -> Result<(), fmt::Error>;
        fn type_id_2(&self) -> TypeId;
        fn type_name(&self) -> &str;
    }

    impl<T> DebugAny for T
    where
        T: ?Sized + 'static + fmt::Debug,
    {
        fn debug(&self, fmt: &mut fmt::Formatter<'_>) -> Result<(), fmt::Error> {
            fmt::Debug::fmt(self, fmt)
        }

        fn type_id_2(&self) -> TypeId {
            self.type_id()
        }

        fn type_name(&self) -> &str {
            std::any::type_name::<T>()
        }
    }

    impl<'q> fmt::Debug for dyn DebugAny + Send {
        fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
            self.debug(f)
        }
    }
}

pub mod clear_double_space {

    pub struct ClearDoubleSpace<Iterator> {
        iterator: Iterator,
        returning: String,
        space: bool,
    }

    impl<I: Iterator<Item = char>> ClearDoubleSpace<I> {
        pub fn new(iterator: I) -> Self {
            Self {
                iterator,
                returning: String::new(),
                space: false,
            }
        }
        pub fn consume(mut self) -> String {
            for each in self.iterator {
                if each == ' ' {
                    if self.space {
                        continue;
                    } else {
                        self.returning.push(each);
                    }
                    self.space = true;
                } else {
                    self.space = false;
                    self.returning.push(each);
                }
            }
            self.returning
        }
    }
}

pub mod utils_some_is_err {
    pub fn some_is_err<E>(optional: Option<E>) -> Result<(), E> {
        match optional {
            Some(e) => Err(e),
            None => Ok(()),
        }
    }

    pub trait SomeIsErr {
        type Error;
        fn some_is_err(self) -> Result<(), Self::Error>;
    }

    impl<E> SomeIsErr for Option<E> {
        type Error = E;
        fn some_is_err(self) -> Result<(), Self::Error> {
            match self {
                Some(e) => Err(e),
                None => Ok(()),
            }
        }
    }
}

// usefull old utils, they all in utils
// folder, I don't want to delete
// because I might come back for them!
// now the way I orgnize utils is by
// placing directly in lib.rs as
// `pub(self)` or `doc(hidden)`
// pub mod utils;
