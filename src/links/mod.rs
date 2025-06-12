use serde_json::Value;
use std::any::Any;

use crate::{
    any_set::AnySet,
    builder_pattern::json_client::{JsonCollection, SelectOneJsonFragment},
};

pub mod group_by;
pub mod set_id;
pub mod set_new;
pub mod relation;
pub mod relation_many_to_many;
pub mod relation_optional_to_many;

pub trait LinkData<From> {
    type Spec;
    /// should I change the reciever to `&self`, I'm requiring `Clone` in many parts due to this
    /// restriction !!!!
    fn spec(self, from: From) -> Self::Spec
    where
        Self: Sized;
}

#[allow(unused)]
pub trait DynamicLink<S> {
    fn init_entry() -> Self::Entry;
    type Entry: Any;
    fn on_register(&self, entry: &mut Self::Entry);
    fn on_finish(&self, build_ctx: &AnySet) -> Result<(), String>;

    fn json_entry() -> &'static str;
    fn get_entry<'a>(&self, ctx: &'a AnySet) -> &'a Self::Entry {
        ctx.get::<Self::Entry>()
            .expect("any set should always contain entry")
    }
    fn on_each_json_request(
        &self,
        base_col: &dyn JsonCollection<S>,
        input: Value,
        ctx: &AnySet,
    ) -> Option<Result<Box<dyn SelectOneJsonFragment<S>>, String>>;
}

// a version of DynamicLink that is trait-object compatible
pub trait DynamicLinkTraitObject<S>: Send + Sync {
    fn on_finish(&self, build_ctx: &AnySet) -> Result<(), String>;
    fn json_entry(&self) -> &'static str;
    fn on_each_json_request(
        &self,
        _base_col: &dyn JsonCollection<S>,
        input: Value,
        ctx: &AnySet,
    ) -> Option<Result<Box<dyn SelectOneJsonFragment<S>>, String>>;
}

impl<S, T> DynamicLinkTraitObject<S> for T
where
    T: DynamicLink<S, Entry: Any> + Send + Sync,
{
    fn on_finish(&self, build_ctx: &AnySet) -> Result<(), String> {
        <Self as DynamicLink<S>>::on_finish(self, build_ctx)
    }
    fn json_entry(&self) -> &'static str {
        T::json_entry()
    }
    fn on_each_json_request(
        &self,
        base_col: &dyn JsonCollection<S>,
        input: Value,
        ctx: &AnySet,
    ) -> Option<Result<Box<(dyn SelectOneJsonFragment<S> + 'static)>, String>> {
        <Self as DynamicLink<S>>::on_each_json_request(self, base_col, input, ctx)
    }
}
