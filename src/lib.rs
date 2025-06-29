#![allow(unused)]
#![deny(unused_must_use)]
use std::marker::PhantomData;

use sqlx::{Database, Pool, Sqlite, SqlitePool};

#[cfg(feature = "http")]
pub mod http;
pub mod build_tuple;
pub mod builder_pattern;
pub mod collections;
pub mod execute;
pub mod expressions;
pub mod filters;
mod identity_management;
pub mod json_client;
pub mod links;
pub mod migration;
pub mod operations;
pub mod prelude;
pub mod quick_query;
pub mod statements;
pub mod update_mod;
pub mod macros {
    pub use claw_ql_macros::*;
}

pub trait QueryBuilder: Database {
    type Fragment;
    type Context1: Default + 'static;
    type Context2: From<Self::Context1>;

    fn build_sql_part_back(ctx: &mut Self::Context2, from: Self::Fragment) -> String;

    type Output;

    fn build_query(
        ctx1: Self::Context1,
        f: impl FnOnce(&mut Self::Context2) -> String,
    ) -> (String, Self::Output);

    fn handle_bind_item<T>(t: T, ctx: &mut Self::Context1) -> Self::Fragment
    where
        T: BindItem<Self> + 'static;

    fn handle_accept<T>(t: T, ctx: &mut Self::Context1) -> Self::Fragment
    where
        T: 'static + Send,
        Self: Accept<T>;
}

pub trait Buildable: Sized {
    type Database: QueryBuilder;
    fn build(self) -> (String, <Self::Database as QueryBuilder>::Output);
}

pub trait BuildableAsRef {
    type Database: QueryBuilder;

    fn build(&self) -> (&str, <Self::Database as QueryBuilder>::Output);
}

impl<St> Buildable for St
where
    St: BuildableAsRef,
{
    type Database = St::Database;
    fn build(self) -> (String, <Self::Database as QueryBuilder>::Output) {
        let (string, output) = <Self as BuildableAsRef>::build(&self);
        (string.to_string(), output)
    }
}

pub trait BindItem<S: QueryBuilder> {
    fn bind_item(
        self,
        ctx: &mut S::Context1,
    ) -> impl FnOnce(&mut S::Context2) -> String + 'static + use<Self, S>;
}

pub trait ColumPositionConstraint {}

#[rustfmt::skip]
mod impl_tuples {
    use crate::{BindItem, ColumPositionConstraint, QueryBuilder};
    use paste::paste;

        macro_rules! implt {
        ($([$ty:ident, $part:literal],)*) => {
    impl<S: QueryBuilder, $($ty,)*> BindItem<S> for ($($ty,)*)
    where
        $($ty: BindItem<S>,)*
    {
        fn bind_item(
            self,
            s: &mut <S as QueryBuilder>::Context1,
        ) -> impl FnOnce(&mut <S as QueryBuilder>::Context2) -> String + 'static + 
            use<$($ty,)* S>
        {
            let tuple = (
                $(paste!(self.$part.bind_item(s)),)*
            );
            |ctx| {
                let mut str = String::new();

                $(
                str.push_str(&paste!(tuple.$part(ctx)));
                str.push(' ');
                )*

                str.pop().unwrap();

                str
            }
        }
    }
        }} // end of macro

    impl<S: QueryBuilder> BindItem<S> for () {
        fn bind_item(
            self,
            _: &mut <S as QueryBuilder>::Context1,
        ) -> impl FnOnce(&mut <S as QueryBuilder>::Context2) -> String + 'static + use<S> {
            |_| "".to_string()
        }
    }

    implt!([R0, 0],);
    implt!([R0, 0], [R1, 1],);

    impl ColumPositionConstraint for () {}
    impl<T0> ColumPositionConstraint for (T0,) {}
    impl<T0, T1> ColumPositionConstraint for (T0, T1) {}
}

pub trait Accept<This>: QueryBuilder + Send {
    fn accept(
        this: This,
        ctx1: &mut Self::Context1,
    ) -> impl FnOnce(&mut Self::Context2) -> String + 'static + Send + use<Self, This>;
}

pub trait IntoMutArguments<'q, DB>
where
    Self: Sized,
    DB: Database,
{
    const LEN: usize;
    fn into_arguments(self, argument: &mut DB::Arguments<'q>);
}

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

pub mod any_set {
    use std::{
        any::{Any, TypeId},
        collections::HashMap,
    };

    #[derive(Default)]
    pub struct AnySet {
        inner: HashMap<TypeId, Box<dyn Any>>,
    }

    pub enum InsertOption<T> {
        Replaces(T),
        WasNew,
    }

    impl AnySet {
        pub fn set<T: Any>(&mut self, item: T) -> InsertOption<T> {
            let type_id = item.type_id();
            match self.inner.insert(type_id, Box::new(item)) {
                Some(replace) => InsertOption::Replaces(*replace.downcast::<T>().unwrap()),
                None => InsertOption::WasNew,
            }
        }
        pub fn get_mut<T: Any>(&mut self) -> Option<&mut T> {
            let type_id = TypeId::of::<T>();
            let get = self.inner.get_mut(&type_id);

            get.map(|e| e.as_mut().downcast_mut::<T>().unwrap())
        }
        pub fn get<T: Any>(&self) -> Option<&T> {
            let type_id = TypeId::of::<T>();
            let get = self.inner.get(&type_id);

            get.map(|e| e.as_ref().downcast_ref::<T>().unwrap())
        }
    }
}

pub trait ConnectInMemory: Database {
    fn connect_in_memory() -> impl Future<Output = Pool<Self>>;
}

impl ConnectInMemory for Sqlite {
    fn connect_in_memory() -> impl Future<Output = Pool<Self>> {
        async { SqlitePool::connect("sqlite::memory:").await.unwrap() }
    }
}
