pub mod relation_many_to_many;
pub mod relation_optional_to_many;
pub mod relation_optional_to_many_inverse;
pub mod timestamp;
pub mod update_links;

pub trait Link<Base> {
    type Spec;
    fn spec(self) -> Self::Spec;
}

#[derive(Clone)]
pub struct DefaultRelationKey;

impl AsRef<str> for DefaultRelationKey {
    fn as_ref(&self) -> &str {
        "_def"
    }
}

/// link back Link::Spec to Base
pub trait LinkedToBase {
    type Base;
}

pub trait LinkedViaId {}
pub trait LinkedViaIds {}
