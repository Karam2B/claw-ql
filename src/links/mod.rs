pub mod group_by;
pub mod relation;
pub mod relation_many_to_many;
pub mod relation_optional_to_many;
pub mod set_id;
pub mod set_new;

pub trait LinkData<From> {
    type Spec;
    /// should I change the reciever to `&self`, I'm requiring `Clone` in many parts due to this
    /// restriction !!!!
    fn spec(self, from: From) -> Self::Spec
    where
        Self: Sized;
}
