use std::marker::PhantomData;

use serde::Serialize;

pub mod collections;
pub mod select_one;
#[cfg(feature = "serde")]
pub mod dynamic_client;

pub trait LinkData<B> {
    type Spec;
    fn spec(self) -> Self::Spec;
}

pub struct Relation<T>(pub(crate) PhantomData<T>);

#[derive(Clone, Debug, PartialEq, Serialize)]
pub struct SimpleOutput<C> {
    pub id: i64,
    pub attr: C,
}

#[derive(Clone, Debug, PartialEq, Serialize)]
pub struct IdOutput<C> {
    pub id: i64,
    #[serde(skip)]
    pub _pd: PhantomData<C>,
}

// impl<From, To> LinkData<From> for Relation<To>
// where
//     From: Related<To>,
// {
//     type Worker =
//         RelationWorker<From::Spec, From, To>;
//     fn init(self) -> Self::Worker {
//         RelationWorker {
//             rel_spec: From::spec(),
//             _pd: PhantomData,
//         }
//     }
// }
