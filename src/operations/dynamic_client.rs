use std::marker::PhantomData;

use serde_json::Value;

use crate::build_tuple::BuildTuple;

pub struct DynamicClient<Collections> {
    c: PhantomData<Collections>,
}

impl Default for DynamicClient<()> {
    fn default() -> Self {
        DynamicClient {
            c: PhantomData::<()>,
        }
    }
}

impl<C> DynamicClient<C>
where
    C: BuildTuple,
{

    pub fn add_collection<C2>(self) -> DynamicClient<C::Bigger<C2>> {
        return DynamicClient { c: PhantomData };
    }
}

impl<C> DynamicClient<C> {
    pub fn select_one(_collection: &str) -> Value {
        todo!()
    }
}

