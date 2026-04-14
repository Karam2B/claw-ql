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

    use crate::from_row::{post_alias, pre_alias, two_alias};

    pub struct DebugRow<T>(pub T);
    impl<'r, R> fmt::Debug for DebugRow<two_alias<'r, R>>
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

    impl<'r, R> fmt::Debug for DebugRow<pre_alias<'r, R>>
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
    impl<'r, R> fmt::Debug for DebugRow<post_alias<'r, R>>
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

// usefull old utils, they all in utils
// folder, I don't want to delete
// because I might come back for them!
// now the way I orgnize utils is by
// placing directly in lib.rs as
// `pub(self)` or `doc(hidden)`
// pub mod utils;
