use std::marker::PhantomData;

use serde::Serialize;
pub mod collections;
pub mod insert_one;
pub mod select_one;

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
