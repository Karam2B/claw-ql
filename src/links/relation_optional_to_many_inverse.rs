#[derive(Clone, Hash, PartialEq, Eq)]
pub struct OptionalToManyInverse<Id, F, T> {
    pub fk_unique_id: Id,
    pub from: F,
    pub to: T,
}
