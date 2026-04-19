#![warn(unused_must_use)]

use std::collections::HashMap;

pub mod timestamp;
// pub mod group_by;
// pub mod relation_many_to_many;
pub mod relation_optional_to_many;
pub mod set_id_mod;
pub mod set_new_mod;
// pub mod relation_optional_to_many_inverse;
// pub mod set_id;
// pub mod set_new;

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

pub trait CollectionsStore {
    type Store;
}

pub struct Issue {
    pub desc: String,
}

#[cfg(feature = "inventory")]
const _: () = {
    // DynamicLink should be depricated because I'm not interested in creating DynamicJsonClient for not
    // but I will keep it for potention DynamicJsonClient
    use crate::inventory::*;
    todo!()
};

pub trait DynamicLink<DynamicBase> {
    type OnRequest;
    type OnRequestInput;
    type OnRequestError;

    fn on_request(
        &self,
        base: &DynamicBase,
        input: Self::OnRequestInput,
    ) -> Result<Self::OnRequest, Self::OnRequestError>;

    type CreateLinkOk;
    type CreateLinkInput;
    type CreateLinkError;

    fn create_link(
        &mut self,
        store: &HashMap<String, DynamicBase>,
        input: Self::CreateLinkInput,
    ) -> Result<Self::CreateLinkOk, Self::CreateLinkError>;

    type ModifyLinkOk;
    type ModifyLinkInput;
    type ModifyLinkError;

    fn modify_link(
        &mut self,
        store: &HashMap<String, DynamicBase>,
        input: Self::ModifyLinkInput,
    ) -> Result<Self::ModifyLinkOk, Self::ModifyLinkError>;
}
