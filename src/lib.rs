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
// pub mod json_client_channel;
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
        extentions::common_expressions::StrAliased,
        from_row::{
            FromRowAlias, FromRowData, FromRowError, RowPostAliased, RowPreAliased, RowTwoAliased,
        },
        query_builder::ManyBoxedExpressions,
    };
    use sqlx::Database;
    use std::any::Any;

    pub trait SelectItemsTraitObject<S, CastFromRowResult>: Send {
        fn str_alias_erase(&self, alias: &'static str) -> Box<dyn ManyBoxedExpressions<S> + Send>;
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

    pub struct ToImplSelectItemsNum<Se, CastFromRowResult> {
        pub num: usize,
        pub select_items: Se,
        pub cast_from_row_result: CastFromRowResult,
    }

    // impl<Se, C> StrAliased for ToImplSelectItems<Se, C>
    // where
    //     Se: Send + StrAliased,
    // {
    //     type StrAliased = Se::StrAliased;
    //     fn str_aliased(&self, alias: &'static str) -> Self::StrAliased {
    //         self.select_items.str_aliased(alias)
    //     }
    // }

    // impl<Se, C> StrAliased for ToImplSelectItemsNum<Se, C>
    // where
    //     Se: Send + StrAliased,
    // {
    //     type StrAliased = Se::StrAliased;
    //     fn str_aliased(&self, alias: &'static str) -> Self::StrAliased {
    //         let _ = alias;

    //         todo!("str_aliased for ToImplSelectItemsNum");
    //     }
    // }

    impl<Se, S> SelectItemsTraitObject<S, ()> for ToImplSelectItems<Se, ()>
    where
        Se: Send + StrAliased<StrAliased: 'static + Send + ManyBoxedExpressions<S>>,
        Se: for<'r> FromRowAlias<'r, S::Row>,
        Se: FromRowData<RData: Send + 'static>,
        S: Database,
    {
        fn str_alias_erase(&self, alias: &'static str) -> Box<dyn ManyBoxedExpressions<S> + Send> {
            Box::new(self.select_items.str_aliased(alias))
        }
        fn no_alias_2<'r>(&self, row: &'r S::Row) -> Result<Box<dyn Any + Send>, FromRowError> {
            Ok(Box::new(self.select_items.no_alias(row)?))
        }
        fn pre_alias_2<'r>(
            &self,
            row: RowPreAliased<'r, S::Row>,
        ) -> Result<Box<dyn Any + Send>, FromRowError> {
            let ret = self.select_items.pre_alias(row)?;
            println!(
                "EraseSelectItems::<NoErase>::pre_alias: \n{:?}",
                ret.type_id()
            );
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

    impl<S, Se> SelectItemsTraitObject<S, ()> for ToImplSelectItemsNum<Se, ()>
    where
        S: Database,
        Se: Send,
        Se: FromRowData<RData: 'static + Send>,
        Se: for<'r> FromRowAlias<'r, S::Row>,
    {
        fn str_alias_erase(&self, alias: &'static str) -> Box<dyn ManyBoxedExpressions<S> + Send> {
            todo!("ret alaias")
        }

        fn no_alias_2<'r>(&self, row: &'r <S>::Row) -> Result<Box<dyn Any + Send>, FromRowError>
        where
            S: Database,
        {
            let row = RowTwoAliased {
                row,
                str_alias: "",
                num_alias: Some(self.num),
            };

            Ok(Box::new(self.select_items.two_alias(row)?))
        }

        fn pre_alias_2<'r>(
            &self,
            row: RowPreAliased<'r, <S>::Row>,
        ) -> Result<Box<dyn Any + Send>, FromRowError>
        where
            S: Database,
            <S>::Row: sqlx::Row,
        {
            let row = RowTwoAliased {
                row: row.row,
                str_alias: row.alias,
                num_alias: Some(self.num),
            };

            Ok(Box::new(self.select_items.two_alias(row)?))
        }

        fn post_alias_2<'r>(
            &self,
            _: RowPostAliased<'r, <S>::Row>,
        ) -> Result<Box<dyn Any + Send>, FromRowError>
        where
            S: Database,
            <S>::Row: sqlx::Row,
        {
            panic!("bug: in the process of deprecating this method")
        }

        fn two_alias_2<'r>(
            &self,
            _: RowTwoAliased<'r, <S>::Row>,
        ) -> Result<Box<dyn Any + Send>, FromRowError>
        where
            S: Database,
            <S>::Row: sqlx::Row,
        {
            panic!("bug: there is some illegal nesting")
        }
    }

    impl<'r, S, C> StrAliased for Box<dyn SelectItemsTraitObject<S, C> + 'r> {
        type StrAliased = Box<dyn ManyBoxedExpressions<S> + Send>;

        fn str_aliased(&self, alias: &'static str) -> Self::StrAliased {
            self.str_alias_erase(alias)
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
}

pub mod temp_fetch_many_for_vec {
    use serde::Serialize;
    use sqlx::Database;

    use crate::from_row::{FromRowAlias, FromRowData};
    use crate::operations::OperationOutput;
    use crate::operations::boxed_operation::BoxedOperation;
    use crate::query_builder::ManyBoxedExpressions;
    use crate::select_items_trait_object::{ToImplSelectItems, ToImplSelectItemsNum};
    use crate::{database_extention::DatabaseExt, extentions::common_expressions::StrAliased};
    use crate::{
        operations::fetch_many::LinkFetchMany, select_items_trait_object::SelectItemsTraitObject,
    };
    use std::any::Any;
    use std::ops::{Deref, DerefMut};

    pub trait JsonLinkFetchMany<S> {
        fn select_items_expr(&self) -> Box<dyn SelectItemsTraitObject<S, ()>>;
        fn post_select_each_2(&self, item: &Box<dyn Any + Send>, poi: &mut Box<dyn Any + Send>);
        fn post_operation_input_init_2(&self) -> Box<dyn Any + Send>;
        fn post_select_2(&self, input: Box<dyn Any + Send>) -> Box<dyn BoxedOperation<S> + Send>;
        fn take_2(
            &self,
            item: Box<dyn Any + Send>,
            op: &mut Box<dyn Any + Send>,
        ) -> serde_json::Value;
        fn join_expr(&self) -> Box<dyn ManyBoxedExpressions<S> + Send>;
        fn wheres_expr(&self) -> Box<dyn ManyBoxedExpressions<S> + Send>;
    }

    impl<S, T> JsonLinkFetchMany<S> for T
    where
        T: Clone + Send + 'static,
        T::SelectItems: Send,
        T::SelectItems: FromRowData,
        S: DatabaseExt,
        T: LinkFetchMany,
        T::SelectItems: Send + StrAliased<StrAliased: 'static + Send + ManyBoxedExpressions<S>>,
        T::PostOperationInput: 'static + Send,
        T::PostOperation: Send + 'static + BoxedOperation<S>,
        T::PostOperation: OperationOutput,
        T::Output: Serialize,
        T::SelectItems: FromRowData<RData: Send + 'static>,
        T::SelectItems: for<'r> FromRowAlias<'r, S::Row>,
        T::Join: Send + 'static + ManyBoxedExpressions<S>,
        T::Wheres: Send + 'static + ManyBoxedExpressions<S>,
    {
        fn join_expr(&self) -> Box<dyn ManyBoxedExpressions<S> + Send> {
            Box::new(self.non_duplicating_join())
        }
        fn wheres_expr(&self) -> Box<dyn ManyBoxedExpressions<S> + Send> {
            Box::new(self.wheres())
        }
        fn take_2(
            &self,
            item: Box<dyn Any + Send>,
            op: &mut Box<dyn Any + Send>,
        ) -> serde_json::Value {
            let s = self.take(
                *item
                    .downcast::<<T::SelectItems as FromRowData>::RData>()
                    .unwrap(),
                op.downcast_mut::<<T::PostOperation as OperationOutput>::Output>()
                    .unwrap(),
            );

            serde_json::to_value(s).expect("bug: serializing should not fail")
        }
        fn select_items_expr(&self) -> Box<dyn SelectItemsTraitObject<S, ()>> {
            Box::new(ToImplSelectItems {
                select_items: self.non_aggregating_select_items(),
                cast_from_row_result: (),
            })
        }
        fn post_select_each_2(
            &self,
            item: &Box<dyn Any + Send>,
            mut poi: &mut Box<dyn Any + Send>,
        ) {
            let ite_down = item
                .deref()
                .downcast_ref::<<T::SelectItems as FromRowData>::RData>();

            let poi_down = poi.deref_mut().downcast_mut::<T::PostOperationInput>();

            self.post_select_each(ite_down.unwrap(), poi_down.unwrap())
        }

        fn post_operation_input_init_2(&self) -> Box<dyn Any + Send> {
            let ret = self.post_operation_input_init();

            println!(
                "JsonLinkFetchMany::post_operation_input_init_2: \n{:?}",
                ret.type_id(),
            );

            println!(
                "same as (): {:?}",
                std::any::TypeId::of::<()>() == ret.type_id()
            );

            Box::new(ret)
        }
        fn post_select_2(&self, input: Box<dyn Any + Send>) -> Box<dyn BoxedOperation<S> + Send> {
            Box::new(self.post_select(*input.downcast::<T::PostOperationInput>().unwrap()))
        }
    }

    impl<'r, S> LinkFetchMany for Box<dyn JsonLinkFetchMany<S> + Send + 'r>
    where
        Box<dyn SelectItemsTraitObject<S, ()>>: FromRowData<RData = Box<dyn Any + Send>>,
        Box<dyn BoxedOperation<S> + Send>: OperationOutput<Output = Box<dyn Any + Send>>,
    {
        type SelectItems = Box<dyn SelectItemsTraitObject<S, ()>>;

        fn non_aggregating_select_items(&self) -> Self::SelectItems {
            self.select_items_expr()
        }

        fn post_select_each(&self, item: &Box<dyn Any + Send>, poi: &mut Self::PostOperationInput)
        where
            Self::SelectItems: FromRowData,
        {
            self.post_select_each_2(item, poi)
        }

        fn take(
            &self,
            item: <Self::SelectItems as FromRowData>::RData,
            op: &mut <Self::PostOperation as OperationOutput>::Output,
        ) -> Self::Output
        where
            Self::SelectItems: FromRowData,
            Self::PostOperation: OperationOutput,
        {
            self.take_2(item, op)
        }

        type Join = Box<dyn ManyBoxedExpressions<S> + Send>;

        fn non_duplicating_join(&self) -> Self::Join {
            self.join_expr()
        }

        type Wheres = Box<dyn ManyBoxedExpressions<S> + Send>;

        fn wheres(&self) -> Self::Wheres {
            self.wheres_expr()
        }

        type Output = serde_json::Value;

        type PostOperationInput = Box<dyn Any + Send>;

        fn post_operation_input_init(&self) -> Self::PostOperationInput {
            let ret = self.post_operation_input_init_2();

            println!(
                "LinkFetchMany::<JsonLinkFetchMany>::post_operation_input_init: \n{:?}",
                ret.type_id()
            );

            println!(
                "same as (): {:?}",
                ret.type_id() == std::any::TypeId::of::<()>()
            );

            println!("downref to (): {:?}", ret.downcast_ref::<()>());

            ret
        }

        type PostOperation = Box<dyn BoxedOperation<S> + Send>;

        fn post_select(&self, input: Self::PostOperationInput) -> Self::PostOperation
        where
            Self::SelectItems: FromRowData,
        {
            self.post_select_2(input)
        }
    }

    impl<'r, S> LinkFetchMany for Vec<Box<dyn JsonLinkFetchMany<S> + Send + 'r>>
    where
        S: Database,
        Vec<Box<dyn SelectItemsTraitObject<S, ()>>>: FromRowData<RData = Vec<Box<dyn Any + Send>>>,
        Box<dyn BoxedOperation<S> + Send>: OperationOutput<Output = Box<dyn Any + Send>>,
    {
        type SelectItems = Vec<Box<dyn SelectItemsTraitObject<S, ()>>>;

        fn non_aggregating_select_items(&self) -> Self::SelectItems {
            let s = self
                .iter()
                .map(|each| each.select_items_expr())
                .collect::<Vec<_>>();

            s
        }

        fn post_select_each(
            &self,
            item: &Vec<Box<dyn Any + Send>>,
            poi: &mut Self::PostOperationInput,
        ) where
            Self::SelectItems: FromRowData,
        {
            // self.post_select_each_2(item, poi)
            todo!()
        }

        fn take(
            &self,
            item: <Self::SelectItems as FromRowData>::RData,
            op: &mut <Self::PostOperation as OperationOutput>::Output,
        ) -> Self::Output
        where
            Self::SelectItems: FromRowData,
            Self::PostOperation: OperationOutput,
        {
            // self.take_2(item, op)
            todo!()
        }

        type Join = Box<dyn ManyBoxedExpressions<S> + Send>;

        fn non_duplicating_join(&self) -> Self::Join {
            // self.join_expr()
            todo!()
        }

        type Wheres = Box<dyn ManyBoxedExpressions<S> + Send>;

        fn wheres(&self) -> Self::Wheres {
            // self.wheres_expr()
            todo!()
        }

        type Output = serde_json::Value;

        type PostOperationInput = Box<dyn Any + Send>;

        fn post_operation_input_init(&self) -> Self::PostOperationInput {
            // let ret = self.post_operation_input_init_2();
            // ret
            todo!()
        }

        type PostOperation = Box<dyn BoxedOperation<S> + Send>;

        fn post_select(&self, input: Self::PostOperationInput) -> Self::PostOperation
        where
            Self::SelectItems: FromRowData,
        {
            // self.post_select_2(input)
            todo!()
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

// usefull old utils, they all in utils
// folder, I don't want to delete
// because I might come back for them!
// now the way I orgnize utils is by
// placing directly in lib.rs as
// `pub(self)` or `doc(hidden)`
// pub mod utils;
