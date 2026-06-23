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
pub mod extend_sqlite;
pub mod from_row;
// pub mod json_client;
pub mod json_value_cmp;
pub mod links;
pub mod on_migrate;
pub mod operations;
pub mod row_utils;
pub mod schema;
pub mod singleton;
pub mod sqlx_query_builder;
pub mod test_module;
pub mod tuple_trait;
pub mod update_mod;
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
        fn fetch_all_mapped<'e, E, R, F>(
            conn: &mut Self::Connection,
            execute: E,
            mapper: F,
        ) -> BoxFuture<'e, Result<Vec<R>, sqlx::Error>>
        where
            E: 'e + Execute<'e, Self>,
            F: Send + 'e + FnMut(Self::Row) -> R,
        {
            let keep_conn_out = Self::fetch_all(conn, execute);
            Box::pin(async move {
                let rows = keep_conn_out.await?;
                Ok(rows.into_iter().map(mapper).collect::<Vec<_>>())
            })
        }

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

pub mod sub_arc {
    use std::ops::{Deref, Range};
    use std::sync::Arc;

    pub type ArcSubStr = SubArc<str>;

    #[derive(Debug, PartialEq, Eq, Hash)]
    pub struct SubArc<T: ?Sized> {
        arc: Arc<T>,
        range: Range<usize>,
    }

    impl SubArc<str> {
        pub fn new(arc: Arc<str>, range: Range<usize>) -> Self {
            Self { arc, range }
        }

        /// The shared buffer this subslice borrows from (or owns after escape decoding).
        pub fn backing_arc(&self) -> &Arc<str> {
            &self.arc
        }

        /// Copy into a fresh `Arc<str>` so collection storage does not keep request JSON alive.
        pub fn detach(&self) -> Arc<str> {
            Arc::from(self.as_str())
        }
    }

    impl Clone for SubArc<str> {
        fn clone(&self) -> Self {
            Self::new(Arc::clone(&self.arc), self.range.clone())
        }
    }

    impl Deref for SubArc<str> {
        type Target = str;
        fn deref(&self) -> &Self::Target {
            &self.arc[self.range.clone()]
        }
    }

    impl AsRef<str> for SubArc<str> {
        fn as_ref(&self) -> &str {
            self.deref()
        }
    }

    impl PartialOrd for SubArc<str> {
        fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
            Some(self.as_str().cmp(other.as_str()))
        }
    }

    impl Ord for SubArc<str> {
        fn cmp(&self, other: &Self) -> std::cmp::Ordering {
            self.as_str().cmp(other.as_str())
        }
    }

    impl SubArc<str> {
        pub fn as_str(&self) -> &str {
            self.deref()
        }
    }

    #[cfg(test)]
    mod test {
        use super::*;

        #[test]
        fn main() {
            let arc: Arc<str> = Arc::from("hello world");
            let sub_arc = SubArc { arc, range: 0..5 };

            assert_eq!(&*sub_arc, "hello");
        }
    }

    #[cfg(test)]
    #[allow(unused)]
    mod alternaive_idea {
        /// this specification is more flexible and general,
        /// but it require one more generic to be infered.
        /// I'm not sure if this is worth it.
        use std::{
            marker::PhantomData,
            ops::{Deref, Range},
            sync::Arc,
        };

        pub struct SubArc<T: ?Sized, FS: FromSource<T>> {
            arc: Arc<FS::Source>,
            from_source: FS,
            _pd: PhantomData<T>,
        }

        impl<T: ?Sized, FS: FromSource<T>> Deref for SubArc<T, FS> {
            type Target = T;
            fn deref(&self) -> &Self::Target {
                self.from_source.clone().deref(&self.arc)
            }
        }

        impl<T: ?Sized, S> SubArc<T, fn(&S) -> &T> {
            fn new_using_fn(arc: Arc<S>, from_source: fn(&S) -> &T) -> Self {
                Self {
                    arc,
                    from_source,
                    _pd: PhantomData,
                }
            }
        }

        pub trait FromSource<T: ?Sized>: Clone {
            type Source: ?Sized;
            fn deref(self, s: &Self::Source) -> &T;
        }

        impl FromSource<str> for Range<usize> {
            type Source = str;
            fn deref(self, s: &str) -> &str {
                &s[self.clone()]
            }
        }

        impl<Source: ?Sized, T: ?Sized> FromSource<T> for fn(&Source) -> &T {
            type Source = Source;
            fn deref(self, s: &Self::Source) -> &T {
                self(s)
            }
        }

        pub struct Dictionary {
            f0: String,
            f1: usize,
        }

        #[test]
        fn main() {
            let arc: Arc<Dictionary> = Arc::new(Dictionary {
                f0: "hello".to_string(),
                f1: 1,
            });

            let sub_arc = SubArc::new_using_fn(arc.clone(), |d: &Dictionary| d.f0.as_str());

            assert_eq!(&*sub_arc, "hello");
        }
    }
}

#[allow(unused)]
#[warn(unused_must_use)]
pub mod constraints {
    use std::ops;

    pub struct Constrained<T, C> {
        pub value: T,
        constraints: C,
    }

    impl<T, C> Constrained<T, C>
    where
        C: Constrain<T>,
    {
        pub fn new(value: T, constraints: C) -> Result<Self, C::Err> {
            constraints.runtime_check(&value)?;
            Ok(Constrained { value, constraints })
        }
    }

    impl<T, C> ops::Deref for Constrained<T, C> {
        type Target = T;

        fn deref(&self) -> &Self::Target {
            &self.value
        }
    }

    impl<T, C> ops::DerefMut for Constrained<T, C> {
        fn deref_mut(&mut self) -> &mut Self::Target {
            &mut self.value
        }
    }

    pub trait Constrain<Value>: Sized {
        type Err;
        fn runtime_check(&self, value: &Value) -> Result<(), Self::Err>;
    }

    pub struct Lowercase;

    impl<T: AsRef<str>> Constrain<T> for Lowercase {
        type Err = ();

        fn runtime_check(&self, value: &T) -> Result<(), Self::Err> {
            let s = value.as_ref().chars();
            let mut string = String::new();

            todo!()
        }
    }
}

pub mod is_null {
    pub trait IsNull {
        fn is_null() -> bool;
    }

    impl<T> IsNull for Option<T> {
        fn is_null() -> bool {
            true
        }
    }

    #[cfg(feature = "nightly_rust_specialization")]
    impl<T> IsNull for T {
        default fn is_null() -> bool {
            false
        }
    }

    #[cfg(not(feature = "nightly_rust_specialization"))]
    mod impl_is_null_no_spectialization {
        use std::collections::HashMap;

        use super::IsNull;

        macro_rules! impl_no_gens {
            ($($ident:ident)*) => {
                $(impl IsNull for $ident {
                    fn is_null() -> bool {
                        false
                    }
                })*
            };
        }

        impl_no_gens!(i32 i64 f64 bool char String);

        macro_rules! impl_gens {
            ($ident:ident [$($gens:ident $(:$wheres:tt)?),*]) => {
                impl<$($gens,)*> IsNull for $ident<$($gens,)*>
                where $($gens:Sized $(+$wheres)? ),*
                {
                    fn is_null() -> bool {
                        false
                    }
                }
            };
        }

        impl_gens!(HashMap[K,V,S]);

        impl<T> IsNull for sqlx::types::Json<T> {
            fn is_null() -> bool {
                false
            }
        }
    }
}

#[cfg(all(test, feature = "trace"))]
mod track_sqlx_query;

mod gen_serde;

// usefull old utils, they all in utils
// folder, I don't want to delete
// because I might come back for them!
// now the way I orgnize utils is by
// placing directly in lib.rs as
// `pub(self)` or `doc(hidden)`
// pub mod utils;
