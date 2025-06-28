use sqlx::{Database, Executor};

use crate::{
    QueryBuilder,
    prelude::stmt::InsertOneSt,
    statements::{delete_st::DeleteSt, select_st::SelectSt, update_st::UpdateSt},
};

#[cfg(feature = "experimental_id_trait")]
pub use id::*;
#[cfg(not(feature = "experimental_id_trait"))]
pub use no_id::*;

mod id {
    use std::marker::PhantomData;

    use sqlx::{Database, Decode, Encode, Executor, Sqlite, prelude::Type};

    use crate::{
        QueryBuilder,
        prelude::stmt::InsertOneSt,
        statements::{delete_st::DeleteSt, select_st::SelectSt, update_st::UpdateSt},
    };
    pub trait CollectionBasic: Sized + Send + Sync + Default + Clone + 'static {
        fn table_name(&self) -> &'static str;
        fn table_name_lower_case(&self) -> &'static str;
        fn members(&self) -> Vec<String>;
        type LinkedData;
    }

    pub trait HasHandler {
        type Handler: CollectionBasic;
    }

    pub trait Id<S> {
        type Data;

        fn on_migrate() -> &'static str;

        fn ident() -> &'static str;
    }

    pub struct SingleIncremintalInt;

    impl Id<Sqlite> for SingleIncremintalInt {
        type Data = i64;

        fn on_migrate() -> &'static str {
            "PRIMARY KEY AUTOINCREMENT"
        }

        fn ident() -> &'static str {
            "id"
        }
    }

    pub trait Collection<S>:
        Sized + Send + Sync + CollectionBasic<LinkedData = Self::Data>
    {
        type Partial;
        type Data;

        // a simplification would be
        // IdData: Id<S> + ...
        //
        // but that will force
        // impl Id<Sqlite> for i32
        //
        // this is restriction because sometime I want to
        // have different id implementaion for the same i32
        type IdData: Type<S> + for<'c> Decode<'c, S> + for<'c> Encode<'c, S>
        where
            S: Database;
        type Id: Id<S, Data = Self::IdData>
        where
            S: Database;
        // where
        //     <Self::Id as Id>::Data: Type<S> + for<'c> Decode<'c, S> + for<'c> Encode<'c, S>;
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

mod no_id {
    use sqlx::{Database, Executor};

    use crate::{
        QueryBuilder,
        prelude::stmt::InsertOneSt,
        statements::{delete_st::DeleteSt, select_st::SelectSt, update_st::UpdateSt},
    };
    pub trait CollectionBasic: Sized + Send + Sync + Default + Clone + 'static {
        fn table_name(&self) -> &'static str;
        fn table_name_lower_case(&self) -> &'static str;
        fn members(&self) -> Vec<String>;
        type LinkedData;
    }

    pub trait HasHandler {
        type Handler: CollectionBasic;
    }

    pub trait Collection<S>:
        Sized + Send + Sync + CollectionBasic<LinkedData = Self::Data>
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

mod on_migrate_tuple_impls {
    use crate::QueryBuilder;
    use crate::migration::OnMigrate;
    use paste::paste;
    use sqlx::{Database, Executor};

    macro_rules! implt {
        ($([$ty:ident, $part:literal]),*) => {
    #[allow(unused)]
    impl<S, $($ty,)*> OnMigrate<S> for ($($ty,)*)
    where
        S: QueryBuilder<Output = <S as Database>::Arguments<'static>>,
        $($ty: OnMigrate<S>,)*
    {
        fn custom_migration<'e>(
            &self,
            exec: impl for<'q> Executor<'q, Database = S> + Clone,
        ) -> impl Future<Output = ()>
        where
            S: QueryBuilder,
        {
            async move {$(
                paste!(self.$part).custom_migration(exec.clone()).await;
            )*}
        }
    }
        }}

    impl<S, R0> OnMigrate<S> for (R0,)
    where
        S: QueryBuilder<Output = <S as Database>::Arguments<'static>>,
        R0: OnMigrate<S>,
    {
        fn custom_migration<'e>(
            &self,
            exec: impl for<'q> Executor<'q, Database = S> + Clone,
        ) -> impl Future<Output = ()>
        where
            S: QueryBuilder,
        {
            async move {
                self.0.custom_migration(exec.clone()).await;
            }
        }
    }
    // implt!();
    implt!([R0, 0], [R1, 1]);
    implt!([R0, 0], [R1, 1], [R2, 2]);
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
pub trait Filter<Q, C>: Sync + Send {
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
