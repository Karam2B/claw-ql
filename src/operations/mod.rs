use std::marker::PhantomData;

use serde::Serialize;

pub mod collections;
pub mod select_one;

pub trait LinkData<From> {
    type Spec;
    fn spec(self) -> Self::Spec;
}

pub struct Relation<To>(pub(crate) PhantomData<To>);

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
