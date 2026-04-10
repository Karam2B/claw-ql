#![allow(non_camel_case_types)]
#![allow(unused)]
#![deny(unused_must_use)]
use sqlx::{AnyPool, Database, Pool, Sqlite, SqlitePool};
use std::marker::PhantomData;

pub mod build_tuple;
pub mod collections;
pub mod execute;
pub mod expressions;
// pub mod filters;
// #[cfg(feature = "http")]
// pub mod http;
// mod identity_management;
// #[cfg(feature = "inventory")]
// pub mod inventory;
// pub mod json_client;
// #[cfg(feature = "serde")]
// pub mod json_query;
// pub mod json_value_cmp;
pub mod links;
// pub mod migration;
// pub mod operations;
pub mod prelude;
pub mod query_builder;
// pub mod on_migrate;
// pub mod quick_query;
pub mod statements;
pub mod update_mod;
// pub mod ident;
// pub mod macros {
//     pub use claw_ql_macros::*;
// }
mod extend_sqlite;

pub use query_builder::*;
pub use serde_json::Value as JsonValue;
pub use sqlx;
use std::any::Any;

#[derive(Debug, Clone)]
pub struct Schema<C, L> {
    pub collections: C,
    pub links: L,
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
