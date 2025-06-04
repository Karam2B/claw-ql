use serde::Serialize;
use std::marker::PhantomData;
pub mod collections;
pub mod select_one;

pub trait LinkData<From> {
    type Spec;
    fn spec(self, from: From) -> Self::Spec
    where
        Self: Sized;
}

// struct general_link;
//
// impl JsonLinkData for general_link {
//     
//     fn spec(self, from: Value) -> Value {
//     }
// }
//
// impl<T> LinkData<T> for general_link {
//     type Spec = ();
//     fn spec(self, from: T) -> Self::Spec
//     where
//         Self: Sized,
//     {
//         let info = from.some_info();
//         info.using_the_info_to_gen_spec()
//     }
// }
// impl LinkData<todo> for category {
//     type Spec = ();
//     fn spec(self) -> Self::Spec {
//         ()
//     }
// }

pub struct Relation<From, To> {
    pub from: From,
    pub to: To,
}

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
