use crate::collections::CollectionBasic;

pub mod group_by;
pub mod relation;
pub mod relation_many_to_many;
pub mod relation_optional_to_many;
pub mod set_id;
pub mod set_new;

pub trait LinkData<From: CollectionBasic> {
    type Spec;
    fn spec(self, from: From) -> Self::Spec
    where
        Self: Sized;
}
