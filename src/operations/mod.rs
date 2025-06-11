use serde::Serialize;
pub mod collections;
pub mod update_one_op;
pub mod insert_one_op;
pub mod select_one_op;

#[derive(Clone, Debug, PartialEq, Serialize)]
pub struct LinkedOutput<C, L> {
    pub id: i64,
    pub attr: C,
    pub links: L,
}

#[derive(Clone, Debug, PartialEq, Serialize)]
pub struct CollectionOutput<C> {
    pub id: i64,
    pub attr: C,
}

#[derive(Clone, Debug, PartialEq, Serialize)]
pub struct IdOutput {
    pub id: i64,
}
impl<C, L> From<LinkedOutput<C, L>> for CollectionOutput<C> {
    fn from(value: LinkedOutput<C, L>) -> Self {
        CollectionOutput {
            id: value.id,
            attr: value.attr,
        }
    }
}

impl<C, L> From<LinkedOutput<C, L>> for IdOutput {
    fn from(value: LinkedOutput<C, L>) -> Self {
        IdOutput { id: value.id }
    }
}

impl<C> From<CollectionOutput<C>> for IdOutput {
    fn from(value: CollectionOutput<C>) -> Self {
        IdOutput { id: value.id }
    }
}
