use std::marker::PhantomData;

use crate::{
    QueryBuilder,
    filters::by_id_mod::by_id,
    json_client::{ErrorReporter, JsonCollection, axum_router_mod::HttpError},
    statements::{delete_st::DeleteSt, select_st::SelectSt, update_st::UpdateSt},
};

use claw_ql_macros::simple_enum;
use hyper::StatusCode;
#[cfg(feature = "unstable_id_trait")]
pub use id::*;
#[cfg(not(feature = "unstable_id_trait"))]
pub use no_id::*;
use serde::de::DeserializeOwned;

pub mod id {
    use claw_ql_macros::simple_enum;
    use serde::{Serialize, de::DeserializeOwned};
    use serde_json::{Value, from_value};
    use sqlx::{ColumnIndex, Database, Decode, Encode, Pool, Sqlite, prelude::Type};

    use crate::{
        Buildable, QueryBuilder,
        expressions::primary_key,
        json_client::add_collection::DynamicCollection,
        links::date_mod::date_spec,
        migration::OnMigrate,
        prelude::stmt::{CreateTableSt, InsertOneSt},
        statements::{create_table_st::header, select_st::SelectSt, update_st::UpdateSt},
    };
    pub trait CollectionBasic {
        fn table_name(&self) -> &str;
        fn table_name_lower_case(&self) -> &str;
        fn members(&self) -> Vec<String>;
    }
    pub trait CollectionHandler: CollectionBasic {
        type LinkedData;
    }

    pub trait HasHandler {
        type Handler: CollectionHandler;
    }

    pub trait Id {
        type SqlIdent;
        fn ident() -> Self::SqlIdent;
    }

    pub struct SingleIncremintalInt;

    impl Id for SingleIncremintalInt {
        type SqlIdent = &'static str;
        fn ident() -> &'static str {
            "id"
        }
    }

    trait OnMigrate2<S> {
        type Statement;
        fn migrate(&self) -> Self::Statement;
    }

    trait LiqOnMigrate<S: Database> {
        fn migrate(&self, pool: Pool<S>) -> Box<dyn Future<Output = ()>>;
    }

    pub trait StatementMayNeverBind {}

    impl<T, S> LiqOnMigrate<S> for T
    where
        S: Database + QueryBuilder,
        T: OnMigrate2<S>,
        T::Statement: Buildable<Database = S>,
        T::Statement: StatementMayNeverBind,
    {
        fn migrate(&self, pool: Pool<S>) -> Box<dyn Future<Output = ()>> {
            let str = OnMigrate2::migrate(self).build();
            todo!()
        }
    }

    impl OnMigrate2<Sqlite> for SingleIncremintalInt {
        type Statement = CreateTableSt<Sqlite>;
        fn migrate(&self) -> Self::Statement {
            let mut stmt = CreateTableSt::init(header::create, <Self as Id>::ident());
            stmt.column_def("id", crate::expressions::exports::primary_key::<Sqlite>());
            stmt
        }
    }

    pub struct AlterTableAddColumn;
    pub struct CreateTrigger;

    impl<F> OnMigrate2<Sqlite> for date_spec<F> {
        type Statement = (AlterTableAddColumn, AlterTableAddColumn, CreateTrigger);
        fn migrate(&self) -> Self::Statement {
            todo!()
        }
    }

    impl OnMigrate<Sqlite> for SingleIncremintalInt {
        fn custom_migrate_statements(&self) -> Vec<String> {
            let mut stmt = CreateTableSt::init(header::create, <Self as Id>::ident());
            stmt.column_def("id", crate::expressions::exports::primary_key::<Sqlite>());
            vec![Buildable::build(stmt).0]
        }
    }

    pub trait Collection<S>:
        Sized + Send + Sync + CollectionHandler<LinkedData = Self::Data>
    {
        type Partial;
        type Data;
        type Id;

        fn on_select(&self, stmt: &mut SelectSt<S>)
        where
            S: QueryBuilder;

        fn on_insert(&self, this: Self::Data, stmt: &mut InsertOneSt<S>)
        where
            S: sqlx::Database;
        fn on_update(&self, this: Self::Partial, stmt: &mut UpdateSt<S>)
        where
            S: QueryBuilder;

        fn from_row_noscope(&self, row: &S::Row) -> Self::Data
        where
            S: Database;
        fn from_row_scoped(&self, row: &S::Row) -> Self::Data
        where
            S: Database;
    }

    use serde_json::Error as SerdeJsonError;

    #[simple_enum]
    pub enum FailedToParse {
        SerdeJsonError,
        String,
    }

    impl From<&'_ str> for FailedToParse {
        fn from(value: &'_ str) -> Self {
            FailedToParse::String(value.to_string())
        }
    }

    pub trait JsonCollection<S>: Send + Sync + 'static {
        fn clone(&self) -> Box<dyn JsonCollection<S>>;
        fn table_name(&self) -> &str;
        fn table_name_lowercase(&self) -> &str;
        fn members(&self) -> Vec<String>;
        fn on_select(&self, stmt: &mut SelectSt<S>)
        where
            S: QueryBuilder;
        fn on_insert(&self, this: Value, stmt: &mut InsertOneSt<S>) -> Result<(), FailedToParse>
        where
            S: sqlx::Database;
        fn on_update(&self, this: Value, stmt: &mut UpdateSt<S>) -> Result<(), FailedToParse>
        where
            S: QueryBuilder;
        fn from_row_noscope(&self, row: &S::Row) -> Value
        where
            S: Database;
        fn from_row_scoped(&self, row: &S::Row) -> Value
        where
            S: Database;
    }

    impl<S: 'static> Clone for Box<dyn JsonCollection<S>> {
        fn clone(&self) -> Self {
            JsonCollection::<S>::clone(&**self)
        }
    }

    impl<S: 'static> CollectionBasic for Box<dyn JsonCollection<S>> {
        fn table_name(&self) -> &str {
            JsonCollection::<S>::table_name(&**self)
        }

        fn table_name_lower_case(&self) -> &str {
            JsonCollection::<S>::table_name_lowercase(&**self)
        }

        fn members(&self) -> Vec<String> {
            JsonCollection::<S>::members(&**self)
        }
    }

    impl<S: 'static> CollectionHandler for Box<dyn JsonCollection<S>> {
        type LinkedData = Value;
    }

    impl<S, T> JsonCollection<S> for T
    where
        T: Clone,
        S: QueryBuilder,
        T: Collection<S> + 'static,
        T::Data: Serialize + DeserializeOwned,
        T::Partial: DeserializeOwned,
    {
        fn clone(&self) -> Box<dyn JsonCollection<S>> {
            Box::new(Clone::clone(self))
        }

        #[inline]
        fn on_select(&self, stmt: &mut SelectSt<S>)
        where
            S: QueryBuilder,
        {
            Collection::<S>::on_select(self, stmt)
        }

        #[inline]
        fn on_insert(&self, this: Value, stmt: &mut InsertOneSt<S>) -> Result<(), FailedToParse>
        where
            S: sqlx::Database,
        {
            let input = from_value::<T::Data>(this)?;
            Collection::<S>::on_insert(self, input, stmt);
            Ok(())
        }

        #[inline]
        fn on_update(&self, this: Value, stmt: &mut UpdateSt<S>) -> Result<(), FailedToParse>
        where
            S: QueryBuilder,
        {
            let input = from_value::<T::Partial>(this)?;
            Collection::<S>::on_update(self, input, stmt);
            Ok(())
        }

        #[inline]
        fn from_row_scoped(&self, row: &S::Row) -> serde_json::Value
        where
            S: Database,
        {
            let row = Collection::<S>::from_row_scoped(self, row);
            serde_json::to_value(row)
                .expect("data integrity bug indicate the bug is within `claw_ql` code")
        }
        #[inline]
        fn from_row_noscope(&self, row: &S::Row) -> Value
        where
            S: Database,
        {
            let row = Collection::<S>::from_row_noscope(self, row);
            serde_json::to_value(row)
                .expect("data integrity bug indicate the bug is within `claw_ql` code")
        }

        fn table_name(&self) -> &str {
            CollectionBasic::table_name(self)
        }

        fn table_name_lowercase(&self) -> &str {
            CollectionBasic::table_name_lower_case(self)
        }

        fn members(&self) -> Vec<String> {
            CollectionBasic::members(self)
        }
    }

    impl<S> JsonCollection<S> for DynamicCollection<S>
    where
        for<'a> &'a str: ColumnIndex<S::Row>,
        S: QueryBuilder + Sync,
    {
        fn clone(&self) -> Box<dyn JsonCollection<S>> {
            Box::new(Clone::clone(self))
        }
        fn table_name(&self) -> &str {
            todo!()
            // &self.name
        }

        fn members(&self) -> Vec<String> {
            todo!()
            // self.fields.iter().map(|e| e.name.to_string()).collect()
        }

        fn on_select(&self, stmt: &mut crate::prelude::stmt::SelectSt<S>)
        where
            S: crate::QueryBuilder,
        {
            for field in self.fields.iter() {
                stmt.select(
                    crate::prelude::col(&field.name)
                        .table(&self.name)
                        .alias(&format!("{}_{}", self.table_name_lowercase(), field.name)),
                );
            }
        }

        fn on_insert(
            &self,
            this: serde_json::Value,
            stmt: &mut crate::prelude::stmt::InsertOneSt<S>,
        ) -> Result<(), FailedToParse>
        where
            S: sqlx::Database,
        {
            let this_obj = this.as_object().ok_or("failed to parse to object")?;
            for field in self.fields.iter() {
                field.type_info.on_insert(
                    this_obj
                        .get(&field.name)
                        .cloned()
                        .ok_or(format!("object doesn't contain keys {}", field.name))?,
                    stmt,
                    &field.name,
                )?;
            }
            todo!()
        }

        fn on_update(
            &self,
            this: serde_json::Value,
            stmt: &mut crate::prelude::macro_derive_collection::UpdateSt<S>,
        ) -> Result<(), FailedToParse>
        where
            S: crate::QueryBuilder,
        {
            todo!()
        }

        fn from_row_noscope(&self, row: &<S>::Row) -> serde_json::Value
        where
            S: Database,
        {
            use sqlx::Row;
            panic!("rows{:?}", row.columns());
            for field in self.fields.iter() {
                let typei = &field.type_info;
                let ret = field.type_info.from_row_optional(&field.name, row);
            }
            todo!()
        }

        #[track_caller]
        fn from_row_scoped(&self, row: &<S>::Row) -> serde_json::Value
        where
            S: Database,
        {
            use sqlx::Row;
            let table_name = &self.name;
            let mut map = serde_json::Map::default();
            for field in self.fields.iter() {
                let name = &field.name;
                let typei = &field.type_info;
                let ret = field
                    .type_info
                    .from_row_optional(&format!("{}_{name}", table_name), row);
                let inserted = map.insert(field.name.clone(), ret);
                if inserted.is_some() {
                    panic!("map should be empty")
                }
            }
            serde_json::to_value(map).unwrap()
        }

        fn table_name_lowercase(&self) -> &str {
            todo!()
        }
    }
}

pub mod no_id {
    use sqlx::Database;

    use crate::{
        QueryBuilder,
        json_client::JsonCollection,
        prelude::stmt::InsertOneSt,
        statements::{select_st::SelectSt, update_st::UpdateSt},
    };
    // todo: remove Default req to fulfill the next impl
    pub trait CollectionHandler: Send + Sync {
        fn table_name(&self) -> &str;
        fn table_name_lower_case(&self) -> &str;
        fn members(&self) -> Vec<String>;
        type LinkedData;
    }

    pub trait CollectionBasics {
        fn table_name(&self) -> &str;
        fn table_name_lower_case(&self) -> &str;
        fn members(&self) -> Vec<String>;
    }
    impl<T: Send + Sync + CollectionHandler> CollectionBasics for T {
        fn table_name(&self) -> &str {
            CollectionHandler::table_name(self)
        }

        fn table_name_lower_case(&self) -> &str {
            CollectionHandler::table_name_lower_case(self)
        }

        fn members(&self) -> Vec<String> {
            CollectionHandler::members(self)
        }
    }

    impl<S> CollectionBasics for &dyn JsonCollection<S> {
        fn table_name(&self) -> &str {
            todo!()
        }

        fn table_name_lower_case(&self) -> &str {
            todo!()
        }

        fn members(&self) -> Vec<String> {
            todo!()
        }
    }

    // impl<S: Send + Sync + QueryBuilder> CollectionHandler for &dyn JsonCollection<S> {
    //     fn table_name(&self) -> &str {
    //         self.table_name_js()
    //     }
    //     fn table_name_lower_case(&self) -> &'static str {
    //         todo!()
    //     }
    //     fn members(&self) -> Vec<String> {
    //         todo!()
    //     }
    //     type LinkedData = Box<dyn JsonCollection<S>>;
    // }

    pub trait HasHandler {
        type Handler: CollectionHandler;
    }

    pub trait Collection<S>:
        Sized + Send + Sync + CollectionHandler<LinkedData = Self::Data>
    {
        type Partial;
        type Data;
        fn on_select(&self, stmt: &mut SelectSt<S>)
        where
            S: QueryBuilder;

        fn on_insert(&self, this: Self::Data, stmt: &mut InsertOneSt<S>)
        where
            S: sqlx::Database;
        fn on_update(&self, this: Self::Partial, stmt: &mut UpdateSt<S>)
        where
            S: QueryBuilder;

        fn from_row_noscope(&self, row: &S::Row) -> Self::Data
        where
            S: Database;
        fn from_row_scoped(&self, row: &S::Row) -> Self::Data
        where
            S: Database;
    }
}

// this is only a concept: I don't know if it is ok to use Box as
// oppose of having three method one for update, select, and delete
mod has_where_clause_trati {
    use crate::{
        BindItem, QueryBuilder,
        prelude::stmt::SelectSt,
        statements::{delete_st::DeleteSt, update_st::UpdateSt},
    };

    use super::Filter;

    pub trait BindItemBoxed<S: QueryBuilder> {
        fn bind_item(
            self: Box<Self>,
            ctx: &mut S::Context1,
        ) -> Box<dyn FnOnce(&mut S::Context2) -> String>;
    }

    impl<S, T> BindItemBoxed<S> for T
    where
        S: QueryBuilder,
        T: BindItem<S> + 'static,
    {
        fn bind_item(
            self: Box<Self>,
            ctx: &mut S::Context1,
        ) -> Box<dyn FnOnce(&mut S::Context2) -> String> {
            let mov = <T as BindItem<S>>::bind_item(*self, ctx);
            Box::new(move |c| mov(c))
        }
    }

    impl<S> BindItem<S> for Box<dyn BindItemBoxed<S>>
    where
        S: QueryBuilder,
    {
        fn bind_item(
            self,
            ctx: &mut <S as QueryBuilder>::Context1,
        ) -> impl FnOnce(&mut <S as QueryBuilder>::Context2) -> String + 'static + use<S> {
            let mov = self.bind_item(ctx);
            move |ctx2| mov(ctx2)
        }
    }

    pub trait HasWhereClause<S> {
        fn delegate_where(&mut self, item: Box<dyn BindItemBoxed<S>>);
    }

    impl<S> HasWhereClause<S> for DeleteSt<S>
    where
        S: QueryBuilder,
    {
        fn delegate_where(&mut self, item: Box<dyn BindItemBoxed<S>>) {
            self.where_(item);
        }
    }

    impl<S> HasWhereClause<S> for UpdateSt<S>
    where
        S: QueryBuilder,
    {
        fn delegate_where(&mut self, item: Box<dyn BindItemBoxed<S>>) {
            self.where_(item);
        }
    }
    impl<S> HasWhereClause<S> for SelectSt<S>
    where
        S: QueryBuilder,
    {
        fn delegate_where(&mut self, item: Box<dyn BindItemBoxed<S>>) {
            self.where_(item);
        }
    }

    pub trait FilterGeneric<Q, C>: Sync + Send + LocalizeFilter {
        fn on_where(self, handler: &C, st: &mut dyn HasWhereClause<Q>)
        where
            Q: QueryBuilder;
    }

    // non-generic trait to solve conflicting impl
    pub trait LocalizeFilter {}

    impl<S, C, T> Filter<S, C> for T
    where
        T: FilterGeneric<S, C> + LocalizeFilter,
    {
        fn on_delete(self, handler: &C, st: &mut DeleteSt<S>)
        where
            S: QueryBuilder,
        {
            self.on_where(handler, st);
        }
        fn on_select(self, handler: &C, st: &mut SelectSt<S>)
        where
            S: QueryBuilder,
        {
            self.on_where(handler, st);
        }
        fn on_update(self, handler: &C, st: &mut UpdateSt<S>)
        where
            S: QueryBuilder,
        {
            self.on_where(handler, st);
        }
    }
}

// TODO: should we use this trait or FilterGeneric?
pub trait Filter<Q, C: ?Sized>: Sync + Send {
    fn on_delete(self, handler: &C, st: &mut DeleteSt<Q>)
    where
        Q: QueryBuilder;
    fn on_update(self, handler: &C, st: &mut UpdateSt<Q>)
    where
        Q: QueryBuilder;
    fn on_select(self, handler: &C, st: &mut SelectSt<Q>)
    where
        Q: QueryBuilder;
}

#[derive(Debug)]
#[cfg_attr(feature = "http", derive(serde::Serialize))]
pub struct FailedToParseBody(pub String);

impl HttpError for FailedToParseBody {
    fn status_code(&self) -> hyper::StatusCode {
        StatusCode::BAD_REQUEST
    }
}

#[derive(Debug)]
#[cfg_attr(feature = "http", derive(serde::Serialize))]
pub struct FilterIsNotApplicableForCollection;

impl HttpError for FilterIsNotApplicableForCollection {
    fn status_code(&self) -> hyper::StatusCode {
        StatusCode::BAD_REQUEST
    }
}

#[simple_enum]
#[derive(Debug)]
pub enum FilterError {
    FailedToParseBody,
    FilterIsNotApplicableForCollection,
}

pub trait LiqFilter<Q>: Send + Sync {
    fn on_select(
        &self,
        rest: serde_json::Value,
        collection: &dyn JsonCollection<Q>,
        st: &mut SelectSt<Q>,
    ) -> Result<(), FilterError>
    where
        Q: QueryBuilder;
}

impl<Q, T> LiqFilter<Q> for PhantomData<T>
where
    Q: QueryBuilder,
    T: Filter<Q, dyn JsonCollection<Q>> + Send + Sync + DeserializeOwned,
{
    fn on_select(
        &self,
        rest: serde_json::Value,
        handler: &dyn JsonCollection<Q>,
        st: &mut SelectSt<Q>,
    ) -> Result<(), FilterError> {
        let s = serde_json::from_value::<T>(rest).map_err(|e| FailedToParseBody(e.to_string()))?;
        s.on_select(handler, st);
        Ok(())
    }
}

// #[rustfmt::skip]
mod filters_tuple_impls {
    use super::Filter;
    use crate::{
        QueryBuilder,
        statements::{delete_st::DeleteSt, select_st::SelectSt, update_st::UpdateSt},
    };
    use paste::paste;

    macro_rules! implt {
        ($([$ty:ident, $part:literal],)*) => {
    #[allow(unused)]
    impl
        <S,C, $($ty,)* >
    Filter<S,C>
    for
        ($($ty,)*)
    where

        S: QueryBuilder,
        $($ty:  Filter<S, C>,)*
    {
        fn on_delete(self, h: &C, st: &mut DeleteSt<S>) {
            $(paste!(self.$part.on_delete(h, st));)*
        }
        fn on_update(self, h: &C, st: &mut UpdateSt<S>) {
            $(paste!(self.$part.on_update(h, st));)*
        }
        fn on_select(self, h: &C, st: &mut SelectSt<S>) {
            $(paste!(self.$part.on_select(h, st));)*
        }
    }
        }}

    implt!();
    #[allow(unused)]
    impl<S, C, R0> Filter<S, C> for (R0,)
    where
        S: QueryBuilder,
        R0: Filter<S, C>,
    {
        fn on_delete(self, h: &C, st: &mut DeleteSt<S>) {
            paste!(self.0.on_delete(h, st));
        }
        fn on_update(self, h: &C, st: &mut UpdateSt<S>) {
            paste!(self.0.on_update(h, st));
        }
        fn on_select(self, h: &C, st: &mut SelectSt<S>) {
            paste!(self.0.on_select(h, st));
        }
    }
    implt!([R0, 0], [R1, 1],);
}
