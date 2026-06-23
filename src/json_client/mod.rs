#![allow(unused)]

/*
THESE ARE NOTE FOR AI AGENT TO HELP DURING THE REFACTOR, AI AGENT, PLEASE READ, AND DON'T MODIFY, HAVE TO BE DELETED AFTER THE REFACTOR.

depricate from pervious versions, (never use it here):
  1. never use serde and serde_json
  2. never use DatabaseForJsonClient

never be specify over Sqlite, be generic over `S: Database`

never modify client_interface, if there is any modification needed, summurize the changes and ask for approval.

always use StringClient in tests

while writing tests, try to reuse code from one I provided in `mod test_utiliteis`,
and don't make any modification to that module, if you believe there should be a modification to that module, summerize your changes and I will add that manually
and don't create your own utility functions, that is a function under #[cfg(test)] that is used inside tests, but it is not #[tokio::test] itself

in unit tests, use r#"here I can use ", lol"# always instead of adding escape for "

in unit tests, to to do full pretty_assertions::assert_eq between queries and what you expected should have ran



*/

mod to_bind;

pub use dynamic_collection::MigrateDynamicCollection;
pub use to_bind::ToBind;
pub type DynOptionalToMany<S> = crate::links::relation_optional_to_many::OptionalToMany<
    crate::links::DefaultRelationKey,
    std::sync::Arc<crate::json_client::dynamic_collection::DynamicCollection<S>>,
    std::sync::Arc<crate::json_client::dynamic_collection::DynamicCollection<S>>,
>;
pub type DynOptionalToManyInverse<S> =
    crate::links::relation_optional_to_many_inverse::OptionalToManyInverse<
        crate::links::DefaultRelationKey,
        std::sync::Arc<crate::json_client::dynamic_collection::DynamicCollection<S>>,
        std::sync::Arc<crate::json_client::dynamic_collection::DynamicCollection<S>>,
    >;
pub type DynManyToMany<S> = crate::links::relation_many_to_many::ManyToMany<
    crate::links::DefaultRelationKey,
    std::sync::Arc<crate::json_client::dynamic_collection::DynamicCollection<S>>,
    std::sync::Arc<crate::json_client::dynamic_collection::DynamicCollection<S>>,
>;
pub type DynTimestamp<S> = crate::links::timestamp::Timestamp<
    std::sync::Arc<crate::json_client::dynamic_collection::DynamicCollection<S>>,
>;

pub mod client_interface;
pub mod dynamic_collection;
mod gen_serde_impls;
mod op_add_collection;
mod op_add_link;
mod op_delete_one;
pub mod op_delete_one_trait_extension;
mod op_fetch_many;
pub mod op_fetch_many_trait_extension;
mod op_fetch_one;
pub mod op_fetch_one_trait_extension;
mod op_insert_many;
mod op_insert_one;
pub mod op_insert_one_trait_extension;
mod op_update_one;
pub mod op_update_one_trait_extension;
mod ops;
mod sqlx_executor;
mod string_client;
mod supported_filters;

#[cfg(test)]
mod test_utilities;
#[cfg(test)]
mod tests;
