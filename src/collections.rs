use sqlx::{Database, Executor};

use crate::{
    QueryBuilder,
    prelude::stmt::InsertOneSt,
    statements::{select_st::SelectSt, update_one_st::UpdateOneSt},
};

pub trait CollectionBasic {
    fn table_name(&self) -> &'static str;
    type LinkedData;
}

pub trait HasHandler {
    type Handler: Default;
}

//
pub trait Collection<S>: Sized + Send + Sync + CollectionBasic<LinkedData = Self::Data> {
    type Partial;
    type Data;
    fn on_select(&self, stmt: &mut SelectSt<S>)
    where
        S: QueryBuilder;

    fn on_insert(&self, this: Self::Data, stmt: &mut InsertOneSt<S>)
    where
        S: sqlx::Database;
    fn on_update(&self, this: Self::Partial, stmt: &mut UpdateOneSt<S>)
    where
        S: QueryBuilder;

    fn members(&self) -> Vec<String>;
    // fn members_scoped(&self) -> &'static [&'static str];
    // fn table_name(&self) -> &'static str;
    fn from_row_noscope(&self, row: &S::Row) -> Self::Data
    where
        S: Database;
    fn from_row_scoped(&self, row: &S::Row) -> Self::Data
    where
        S: Database;
}

pub trait OnMigrate<S> {
    fn custom_migration<'e>(
        &self,
        exec: impl for<'q> Executor<'q, Database = S> + Clone,
    ) -> impl Future<Output = ()>
    where
        S: QueryBuilder;
}

mod on_migrate_tuple_impls {
    use super::OnMigrate;
    use crate::QueryBuilder;
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
        BindItem, QueryBuilder, prelude::stmt::SelectSt, statements::update_one_st::UpdateOneSt,
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

    #[allow(unused)]
    pub fn on_where_concept<S>(_statement: &dyn HasWhereClause<S>) {}

    impl<S> HasWhereClause<S> for UpdateOneSt<S>
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

    // temperoraly trait to solve conflicting impl with std-types
    pub trait LocalizeFilter {}

    impl<S, C, T> Filter<S, C> for T
    where
        T: FilterGeneric<S, C> + LocalizeFilter,
    {
        fn on_select(self, handler: &C, st: &mut SelectSt<S>)
        where
            S: QueryBuilder,
        {
            self.on_where(handler, st);
        }
        fn on_update(self, handler: &C, st: &mut UpdateOneSt<S>)
        where
            S: QueryBuilder,
        {
            self.on_where(handler, st);
        }
    }
}

// TODO: should we use this trait of FilterGeneric? 
pub trait Filter<Q, C>: Sync + Send {
    fn on_update(self, handler: &C, st: &mut UpdateOneSt<Q>)
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
        QueryBuilder, statements::select_st::SelectSt, statements::update_one_st::UpdateOneSt,
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
        fn on_update(self, h: &C, st: &mut UpdateOneSt<S>) {
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
        fn on_update(self, h: &C, st: &mut UpdateOneSt<S>) {
            paste!(self.0.on_update(h, st));
        }
        fn on_select(self, h: &C, st: &mut SelectSt<S>) {
            paste!(self.0.on_select(h, st));
        }
    }
    implt!([R0, 0], [R1, 1],);
}
