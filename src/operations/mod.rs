use serde::Serialize;
use std::marker::PhantomData;
pub mod collections;
pub mod select_one;

pub trait LinkData<From> {
    type Spec;
    fn spec(self) -> Self::Spec;
}

pub struct Relation<From, To> {
    pub from: PhantomData<From>,
    pub to: PhantomData<To>,
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
