#![allow(unused)]
#![deny(unused_must_use)]
use sqlx::{AnyPool, Database, Pool, Sqlite, SqlitePool};
use std::marker::PhantomData;

pub mod build_tuple;
pub mod builder_pattern;
pub mod collections;
pub mod execute;
pub mod expressions;
pub mod filters;
#[cfg(feature = "http")]
pub mod http;
mod identity_management;
#[cfg(feature = "inventory")]
pub mod inventory;
pub mod json_client;
pub mod links;
pub mod migration;
pub mod operations;
pub mod prelude;
pub mod query_builder;
// pub mod quick_query;
pub mod statements;
pub mod update_mod;
pub mod macros {
    pub use claw_ql_macros::*;
}

pub use query_builder::*;
use std::any::Any;



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

impl ConnectInMemory for sqlx::Any {
    fn connect_in_memory() -> impl Future<Output = Pool<Self>> {
        async { AnyPool::connect("sqlite::memory:").await.unwrap() }
    }
}
impl ConnectInMemory for Sqlite {
    fn connect_in_memory() -> impl Future<Output = Pool<Self>> {
        async { SqlitePool::connect("sqlite::memory:").await.unwrap() }
    }
}

pub mod ident {
    use core::hash;
    use std::{
        collections::{HashMap, HashSet},
        hash::Hash,
    };



    use crate::{collections::CollectionBasic, links::relation::Relation};

    #[derive(Eq, PartialEq, Clone, Hash)]
    pub enum TypeIdent {
        Leaf(&'static str),
        Branch(&'static str, Vec<TypeIdent>),
    }

    impl TypeIdent {
        pub fn leaf(name: &'static str) -> TypeIdent {
            TypeIdent::Leaf(name)
        }
        pub fn branch(name: &'static str, to: Vec<TypeIdent>) -> Result<TypeIdent, ()> {
            let mut ve = vec![];
            let mut set = HashSet::new();
            for each in to {
                let name = match each {
                    TypeIdent::Leaf(s) => s,
                    TypeIdent::Branch(s, _) => s,
                };
                let false_if_already_exist = set.insert(name);
                if false_if_already_exist == false {
                    return Err(());
                }
                ve.push(each.clone());
            }
            Ok(TypeIdent::Branch(name, ve))
        }
    }

    pub struct Singlton {
        _priv: (),
    }

    static SINGLTON: Singlton = Singlton { _priv: () };

    pub trait AnyS<R> {
        fn type_id(&self, reg: &'static R) -> TypeIdent;
    }

    impl<F: CollectionBasic, T: CollectionBasic> AnyS<Singlton> for Relation<F, T> {
        fn type_id(&self, reg: &'static Singlton) -> TypeIdent {
            TypeIdent::branch(
                "relation",
                [
                    TypeIdent::leaf(self.from.table_name()),
                    TypeIdent::leaf(self.to.table_name()),
                ]
                .into_iter()
                .collect(),
            )
            .unwrap()

        }
    }
}