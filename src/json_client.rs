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

pub use crate::json_client_v0::dynamic_collection::impl_on_migrate::MigrateDynamicCollection;
pub use crate::json_client_v0::to_bind_trait::ToBind;
pub type DynOptionalToMany<S> = crate::links::relation_optional_to_many::OptionalToMany<
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

pub mod client_interface {
    use std::collections::BTreeMap;

    use crate::expressions::ColumnEqual;
    use crate::gen_serde::Serialize;
    use crate::gen_serde::json_format_side::PartialDeserialize;
    use crate::gen_serde::json_serialize_side::JsonAsString;
    use crate::json_client::dynamic_collection::CollectionToSerialize;
    use crate::operations::{CollectionOutput, LinkedOutput};
    use crate::sub_arc::ArcSubStr;

    //*******************
    //*
    //* SupportedType
    //*
    //*******************
    #[derive(Debug)]
    pub enum SupportedType {
        String,
        Boolean,
    }

    //*******************
    //*
    //* SupportedFilter
    //*
    //*******************
    #[derive(Debug)]
    pub enum SupportedFilter {
        ColEq(ColumnEqual<ArcSubStr, PartialDeserialize>),
    }

    //*******************
    //*
    //* AddCollection
    //*
    //*******************
    #[derive(Debug)]
    pub struct AddCollectionInput {
        pub name: ArcSubStr,
        pub fields: Vec<DynamicFieldInput>,
    }

    #[derive(Debug)]
    pub struct DynamicFieldInput {
        pub name: ArcSubStr,
        pub type_info: SupportedType,
        pub is_optional: bool,
    }

    pub type AddCollectionOutput = ();

    #[derive(Debug)]
    pub enum AddCollectionError {
        CollectionAlreadyExists,
        InvalidCollectionInput,
    }

    //*******************
    //*
    //* AddLink
    //*
    //*******************
    #[derive(Debug)]
    pub enum AddLinkInput {
        OptionalToMany { from: ArcSubStr, to: ArcSubStr },
        ManyToMany { from: ArcSubStr, to: ArcSubStr },
        Timestamp { collection: ArcSubStr },
    }

    pub type AddLinkOutput = ();

    #[derive(Debug)]
    pub enum AddLinkError {
        LinkAlreadyExists,
        CollectionNotFound,
    }

    //*******************
    //*
    //* InsertOne
    //*
    //*******************
    pub use crate::gen_serde::SerializedJson;
    pub use crate::operations::fetch_many::ManyOutput;

    #[derive(Debug)]
    pub enum SupportedInsertLink {
        SetId {
            to: ArcSubStr,
            id: i64,
        },
        SetNew {
            to: ArcSubStr,
            value: PartialDeserialize,
        },
    }

    #[derive(Debug)]
    pub struct InsertOneInput {
        pub base: ArcSubStr,
        pub data: PartialDeserialize,
        pub links: Vec<SupportedInsertLink>,
    }

    pub type InsertOneOutput =
        LinkedOutput<i64, CollectionToSerialize, Vec<Box<dyn Serialize<JsonAsString> + Send>>>;

    #[derive(Debug)]
    pub enum InsertOneError {
        CollectionNotFound,
        InvalidData,
        InvalidLink,
        LinkNotSetUpForThisBase,
    }

    //*******************
    //*
    //* FetchMany
    //*
    //*******************
    #[derive(Debug)]
    pub enum SupportedLinkFetchMany {
        OptionalToMany { to: ArcSubStr },
        ManyToMany { to: ArcSubStr },
        Timestamp,
    }

    #[derive(Debug)]
    pub struct FetchManyInput {
        pub base: ArcSubStr,
        pub filters: Vec<SupportedFilter>,
        pub links: Vec<SupportedLinkFetchMany>,
        pub pagination: Pagination,
    }

    #[derive(Debug)]
    pub struct Pagination {
        pub limit: i64,
        pub first_item: Option<FirstItem>,
        pub order_by: Vec<OrderBy>,
    }

    #[derive(Debug)]
    pub struct FirstItem {
        pub id: i64,
        pub data: BTreeMap<ArcSubStr, PartialDeserialize>,
    }

    #[derive(Debug)]
    pub struct OrderBy {
        pub col: ArcSubStr,
        pub direction: Direction,
    }

    #[derive(Debug)]
    pub enum Direction {
        Asc,
        Desc,
    }

    pub type FetchManyItem =
        LinkedOutput<i64, CollectionToSerialize, Vec<Box<dyn Serialize<JsonAsString> + Send>>>;

    pub type FetchManyOutput =
        ManyOutput<FetchManyItem, CollectionOutput<i64, CollectionToSerialize>>;

    #[derive(Debug)]
    pub enum FetchManyError {
        CollectionNotFound,
        InvalidData,
        LinkNotSetUpForThisBase,
        InvalidFilter,
        InvalidLink,
        InvalidOrderBy,
        InvalidFirstItem,
    }

    //*******************
    //*
    //* FetchOne
    //*
    //*******************
    #[derive(Debug)]
    pub enum SupportedLinkFetchOne {
        OptionalToMany { to: ArcSubStr },
        ManyToMany { to: ArcSubStr },
        Timestamp,
    }

    #[derive(Debug)]
    pub struct FetchOneInput {
        pub base: ArcSubStr,
        pub id: i64,
        pub filters: Vec<SupportedFilter>,
        pub links: Vec<SupportedLinkFetchOne>,
    }

    pub type FetchOneOutput =
        LinkedOutput<i64, CollectionToSerialize, Vec<Box<dyn Serialize<JsonAsString> + Send>>>;

    #[derive(Debug)]
    pub enum FetchOneError {
        CollectionNotFound,
        NotFound,
        InvalidFilter,
        InvalidLink,
    }

    //*******************
    //*
    //* UpdateOne
    //*
    //*******************
    #[derive(Debug)]
    pub enum SupportedUpdateLink {
        SetId {
            to: ArcSubStr,
            id: i64,
        },
        SetNew {
            to: ArcSubStr,
            value: PartialDeserialize,
        },
        SetNull {
            to: ArcSubStr,
        },
        RemoveId {
            to: ArcSubStr,
            id: i64,
        },
    }

    #[derive(Debug)]
    pub struct UpdateOneInput {
        pub base: ArcSubStr,
        pub id: i64,
        pub data: PartialDeserialize,
        pub links: Vec<SupportedUpdateLink>,
    }

    pub type UpdateOneOutput =
        LinkedOutput<i64, CollectionToSerialize, Vec<Box<dyn Serialize<JsonAsString> + Send>>>;

    #[derive(Debug)]
    pub enum UpdateOneError {
        CollectionNotFound,
        InvalidData,
        NotFound,
        InvalidLink,
    }

    //*******************
    //*
    //* DeleteOne
    //*
    //*******************
    #[derive(Debug)]
    pub enum SupportedDeleteLink {
        OptionalToMany { to: ArcSubStr },
        ManyToMany { to: ArcSubStr },
    }

    #[derive(Debug)]
    pub struct DeleteOneInput {
        pub base: ArcSubStr,
        pub id: i64,
        pub links: Vec<SupportedDeleteLink>,
    }

    pub type DeleteOneOutput =
        LinkedOutput<i64, CollectionToSerialize, Vec<Box<dyn Serialize<JsonAsString> + Send>>>;

    #[derive(Debug)]
    pub enum DeleteOneError {
        CollectionNotFound,
        NotFound,
        InvalidLink,
    }

    //*******************
    //*
    //* Client
    //*
    //*******************
    pub use super::ops::OperationError as ClientOperationError;
    pub use super::ops::OperationInput as ClientOperationInput;
    pub use super::ops::OperationOutput as ClientOperationOutput;

    pub struct Client {
        pub(crate) sender: tokio::sync::mpsc::UnboundedSender<(
            ClientOperationInput,
            oneshot::Sender<Result<ClientOperationOutput, ClientOperationError>>,
        )>,
    }
}
mod ops {
    use super::client_interface::*;
    use super::fetch_many_trait_extension::JsonLinkFetchMany;
    use super::fetch_one_trait_extension::JsonLinkFetchOne;

    macro_rules! ops {
        ($([$snake_case:ident, $pascal_case:ident]),*) => {
            paste::paste!{
                #[derive(Debug)]
                #[non_exhaustive]
                pub enum OperationOutput {
                    $(
                        $pascal_case ([<$pascal_case Output>]),
                    )*
                }

                #[derive(Debug)]
                #[non_exhaustive]
                pub enum OperationInput {
                    $(
                        $pascal_case ([<$pascal_case Input>]),
                    )*
                }

                #[derive(Debug)]
                #[non_exhaustive]
                pub enum OperationError {
                    $(
                        $pascal_case ([<$pascal_case Error>]),
                    )*
                }

                impl Client {
                    $(
                        pub fn
                            $snake_case(&self, input: [<$pascal_case Input>])
                        -> impl Future<Output = Result<[<$pascal_case Output>], [<$pascal_case Error>]>> {
                            let (tx, rx) = oneshot::async_channel::<Result<OperationOutput, OperationError>>();
                            self.sender.send((OperationInput::$pascal_case(input), tx)).unwrap();
                            async move {
                                let output = rx.await.expect("disconnected_channel");
                                let mapp = match output {
                                    Ok(OperationOutput::$pascal_case(e)) => Ok(e),
                                    Err(OperationError::$pascal_case(e)) => Err(e),
                                    _ => panic!("invalid mapping"),
                                };

                                return mapp;
                            }
                        }
                    )*
                }

                impl<S> $crate::json_client::sqlx_executor::SqlxExecutor<S>
                where
                    S: ::std::marker::Sync,
                    S: $crate::fix_executor::ExecutorTrait,
                    S: $crate::database_extention::DatabaseExt,
                    bool: for<'d> ::sqlx::Decode<'d, S> + ::sqlx::Type<S> + for<'q> ::sqlx::Encode<'q, S>,
                    std::string::String:
                        ::sqlx::Type<S> + for<'q> ::sqlx::Encode<'q, S> + for<'d> ::sqlx::Decode<'d, S>,
                    $crate::json_client::dynamic_collection::DynamicCollection<S>:
                        $crate::on_migrate::OnMigrate<Statements: $crate::query_builder::Expression<'static, S>>,
                    for<'a> S::Arguments<'a>: ::sqlx::IntoArguments<'a, S>,
                    for<'a> &'a str: ::sqlx::ColumnIndex<<S as ::sqlx::Database>::Row>,
                    i64: for<'r> ::sqlx::Decode<'r, S>
                        + for<'q> ::sqlx::Encode<'q, S>
                        + ::sqlx::Type<S>,
                    $crate::links::relation_optional_to_many::OptionalToMany<
                        $crate::links::DefaultRelationKey,
                        std::sync::Arc<$crate::json_client::dynamic_collection::DynamicCollection<S>>,
                        std::sync::Arc<$crate::json_client::dynamic_collection::DynamicCollection<S>>,
                    >: $crate::json_client::fetch_many_trait_extension::JsonLinkFetchMany<S>,
                    $crate::links::relation_many_to_many::ManyToMany<
                        $crate::links::DefaultRelationKey,
                        std::sync::Arc<$crate::json_client::dynamic_collection::DynamicCollection<S>>,
                        std::sync::Arc<$crate::json_client::dynamic_collection::DynamicCollection<S>>,
                    >: $crate::json_client::fetch_many_trait_extension::JsonLinkFetchMany<S>,
                    $crate::links::timestamp::Timestamp<
                        std::sync::Arc<$crate::json_client::dynamic_collection::DynamicCollection<S>>,
                    >: $crate::json_client::fetch_many_trait_extension::JsonLinkFetchMany<S>,
                    $crate::links::relation_optional_to_many::OptionalToMany<
                        $crate::links::DefaultRelationKey,
                        std::sync::Arc<$crate::json_client::dynamic_collection::DynamicCollection<S>>,
                        std::sync::Arc<$crate::json_client::dynamic_collection::DynamicCollection<S>>,
                    >: $crate::json_client::fetch_one_trait_extension::JsonLinkFetchOne<S>,
                    $crate::links::relation_many_to_many::ManyToMany<
                        $crate::links::DefaultRelationKey,
                        std::sync::Arc<$crate::json_client::dynamic_collection::DynamicCollection<S>>,
                        std::sync::Arc<$crate::json_client::dynamic_collection::DynamicCollection<S>>,
                    >: $crate::json_client::fetch_one_trait_extension::JsonLinkFetchOne<S>,
                    $crate::links::timestamp::Timestamp<
                        std::sync::Arc<$crate::json_client::dynamic_collection::DynamicCollection<S>>,
                    >: $crate::json_client::fetch_one_trait_extension::JsonLinkFetchOne<S>,
                    $crate::links::relation_optional_to_many::OptionalToMany<
                        $crate::links::DefaultRelationKey,
                        std::sync::Arc<$crate::json_client::dynamic_collection::DynamicCollection<S>>,
                        std::sync::Arc<$crate::json_client::dynamic_collection::DynamicCollection<S>>,
                    >: $crate::on_migrate::OnMigrate<
                        Statements: for<'q> $crate::query_builder::Expression<'q, S>,
                    >,
                    $crate::links::relation_many_to_many::ManyToMany<
                        $crate::links::DefaultRelationKey,
                        std::sync::Arc<$crate::json_client::dynamic_collection::DynamicCollection<S>>,
                        std::sync::Arc<$crate::json_client::dynamic_collection::DynamicCollection<S>>,
                    >: $crate::on_migrate::OnMigrate<
                        Statements: for<'q> $crate::query_builder::Expression<'q, S>,
                    >,
                    $crate::links::timestamp::Timestamp<
                        std::sync::Arc<$crate::json_client::dynamic_collection::DynamicCollection<S>>,
                    >: $crate::on_migrate::OnMigrate<
                        Statements: for<'q> $crate::query_builder::Expression<'q, S>,
                    >,
                    std::sync::Arc<$crate::json_client::dynamic_collection::DynamicCollection<S>>:
                        for<'r> $crate::from_row::FromRowAlias<
                            'r,
                            S::Row,
                            RData = $crate::json_client::dynamic_collection::CollectionToSerialize,
                        >,

                {
                    pub fn run(mut self) -> impl Future<Output = ::std::convert::Infallible> {
                        async move {
                            loop {
                                let operation = self.reciever.recv().await.unwrap();

                                paste::paste!{
                                match operation.0 {
                                    $(OperationInput::$pascal_case(input) => {
                                        let future = $crate::json_client::[<$snake_case _mod>]::[<$snake_case>](self.data.clone(), input);
                                        let operation_sender = operation.1;
                                        let dispatch = ::tracing::dispatcher::Dispatch::default();
                                        let span = ::tracing::Span::current();
                                        tokio::spawn(async move {
                                            let _dispatch =
                                                ::tracing::dispatcher::set_default(&dispatch);
                                            let _span = span.enter();
                                            operation_sender
                                                .send(
                                                    future
                                                        .await
                                                        .map(OperationOutput::$pascal_case)
                                                        .map_err(OperationError::$pascal_case),
                                                )
                                                .unwrap();
                                        });
                                    })*
                                }}
                            }
                        }
                    }
                }

                impl $crate::json_client::string_client::StringClient {
                    pub fn exec(&self, input: String) -> impl Future<Output = String> {
                        use crate::gen_serde::Serialize;
                        use crate::gen_serde::Deserialize;
                        use crate::gen_serde::json_format_side::JsonAsArcCursor;

                        async move {
                            let mut cursor = JsonAsArcCursor {
                                inner: input.as_str().into(),
                                start: 0,
                            };

                            let input = match $crate::json_client::string_client::StringClientInput::deserialize((), &mut cursor) {
                                Ok(input) => input,
                                Err(e) => return String::from(r#"{"error":"invalid_input"}"#),
                            };

                            match input.op.as_str() {
                                $(
                                    stringify!($snake_case) => {
                                        let body = match input.body.continue_deserialize() {
                                            Ok(body) => body,
                                            Err(e) => return String::from(r#"{"error":"invalid_body"}"#),
                                        };
                                        let output = match self.inner.$snake_case(body).await {
                                            Ok(output) => output,
                                            Err(e) => return format!("{{\"error\":\"{:?}\"}}", e),
                                        };

                                        let mut serialized = $crate::gen_serde::json_serialize_side::JsonAsString(String::new());
                                        $crate::json_client::string_client::StringClientOutput { output }.serialize(&mut serialized);

                                        return serialized.0;
                                    }
                                )*
                                _ => return String::from(r#"{"error":"unsupported_operation"}"#),
                            }
                        }
                    }
                }
            }
        };
    }

    ops!(
        [add_collection, AddCollection],
        [add_link, AddLink],
        [fetch_many, FetchMany],
        [fetch_one, FetchOne],
        [insert_one, InsertOne],
        [update_one, UpdateOne],
        [delete_one, DeleteOne]
    );
}

mod string_client {
    use crate::{
        gen_serde::{
            ObjectEncoding, Serialize, SerializedJson, json_format_side::PartialDeserialize,
            json_serialize_side::JsonAsString,
        },
        json_client::client_interface::Client,
        sub_arc::ArcSubStr,
    };

    pub struct StringClient {
        pub(super) inner: Client,
    }

    impl Client {
        pub fn into_string_client(self) -> StringClient {
            StringClient { inner: self }
        }
    }

    pub(super) struct StringClientInput {
        pub(super) op: ArcSubStr,
        pub(super) body: PartialDeserialize,
    }

    impl crate::gen_serde::DeserializeSpec for StringClientInput {
        type Handler = ();
    }

    pub(super) struct StringClientOutput<T> {
        pub(super) output: T,
    }

    impl<F, T> Serialize<F> for StringClientOutput<T>
    where
        F: ObjectEncoding,
        T: Serialize<F>,
        str: Serialize<F>,
    {
        fn serialize(&self, ctx: &mut F) {
            let mut object = ctx.serialize_start();
            ctx.serialize_pair(&mut object, "output", &self.output);
            ctx.serialize_end(object);
        }
    }

    impl<'de, S> crate::gen_serde::Deserialize<'de, S> for StringClientInput
    where
        S: crate::gen_serde::Deserializer<'de>,
        ArcSubStr: crate::gen_serde::Deserialize<'de, S>,
        PartialDeserialize: crate::gen_serde::Deserialize<'de, S>,
        S: crate::gen_serde::DeserializeMap<'de>,
        S: crate::gen_serde::KnownKey<&'static str>,
    {
        fn deserialize(_handler: Self::Handler, serialized: &mut S) -> Result<Self, S::Err> {
            let mut map = crate::gen_serde::DeserializeMap::start_map(serialized)?;
            let op = crate::gen_serde::DeserializeMap::deserialize_with_known_key(
                serialized,
                &mut map,
                "op",
                (),
            )?;
            let body = crate::gen_serde::DeserializeMap::deserialize_with_known_key(
                serialized,
                &mut map,
                "body",
                (),
            )?;
            crate::gen_serde::DeserializeMap::finish(serialized, map)?;
            Ok(StringClientInput { op, body })
        }
    }
}

mod gen_serde_impls {
    use std::collections::BTreeMap;

    use crate::expressions::ColumnEqual;
    use crate::gen_serde::json_format_side::PartialDeserialize;
    use crate::gen_serde::{
        Deserialize, DeserializeMap, DeserializeSeq, DeserializeSpec, Deserializer, KnownKey,
        UnknownKey,
    };
    use crate::json_client::client_interface::{
        AddCollectionInput, AddLinkInput, DeleteOneInput, Direction, DynamicFieldInput,
        FetchManyInput, FetchOneInput, FirstItem, InsertOneInput, OrderBy, Pagination,
        SupportedDeleteLink, SupportedFilter, SupportedInsertLink, SupportedLinkFetchMany,
        SupportedLinkFetchOne, SupportedType, SupportedUpdateLink, UpdateOneInput,
    };
    use crate::sub_arc::{ArcSubStr, SubArc};

    impl DeserializeSpec for SupportedType {
        type Handler = ();
    }

    impl<'de, S> Deserialize<'de, S> for SupportedType
    where
        S: Deserializer<'de>,
        String: Deserialize<'de, S>,
        S::Err: From<&'static str>,
    {
        fn deserialize(_handler: Self::Handler, serialized: &mut S) -> Result<Self, S::Err> {
            match String::deserialize((), serialized)?.as_str() {
                "String" => Ok(SupportedType::String),
                "Boolean" => Ok(SupportedType::Boolean),
                _other => Err(S::Err::from("unsupported SupportedType")),
            }
        }
    }

    impl DeserializeSpec for DynamicFieldInput {
        type Handler = ();
    }

    impl<'de, S> Deserialize<'de, S> for DynamicFieldInput
    where
        S: Deserializer<'de>,
        S: DeserializeMap<'de>,
        ArcSubStr: Deserialize<'de, S>,
        SupportedType: Deserialize<'de, S>,
        bool: Deserialize<'de, S>,
        S: KnownKey<&'static str>,
    {
        fn deserialize(_handler: Self::Handler, serialized: &mut S) -> Result<Self, S::Err> {
            let mut map = DeserializeMap::start_map(serialized)?;
            let name =
                DeserializeMap::deserialize_with_known_key(serialized, &mut map, "name", ())?;
            let type_info =
                DeserializeMap::deserialize_with_known_key(serialized, &mut map, "type_info", ())?;
            let is_optional = DeserializeMap::deserialize_with_known_key(
                serialized,
                &mut map,
                "is_optional",
                (),
            )?;
            DeserializeMap::finish(serialized, map)?;
            Ok(DynamicFieldInput {
                name,
                type_info,
                is_optional,
            })
        }
    }

    impl DeserializeSpec for AddCollectionInput {
        type Handler = ();
    }

    impl<'de, S> Deserialize<'de, S> for AddCollectionInput
    where
        S: Deserializer<'de>,
        S: DeserializeMap<'de>,
        ArcSubStr: Deserialize<'de, S>,
        Vec<DynamicFieldInput>: Deserialize<'de, S>,
        S: KnownKey<&'static str>,
    {
        fn deserialize(_handler: Self::Handler, serialized: &mut S) -> Result<Self, S::Err> {
            let mut map = DeserializeMap::start_map(serialized)?;
            let name =
                DeserializeMap::deserialize_with_known_key(serialized, &mut map, "name", ())?;
            let fields =
                DeserializeMap::deserialize_with_known_key(serialized, &mut map, "fields", ())?;
            DeserializeMap::finish(serialized, map)?;
            Ok(AddCollectionInput { name, fields })
        }
    }

    impl DeserializeSpec for InsertOneInput {
        type Handler = ();
    }

    impl<'de, S> Deserialize<'de, S> for InsertOneInput
    where
        S: Deserializer<'de>,
        S: DeserializeMap<'de>,
        S: DeserializeSeq<'de>,
        ArcSubStr: Deserialize<'de, S>,
        PartialDeserialize: Deserialize<'de, S>,
        Vec<SupportedInsertLink>: Deserialize<'de, S>,
        S: KnownKey<&'static str>,
        S::Err: From<&'static str>,
    {
        fn deserialize(_handler: Self::Handler, serialized: &mut S) -> Result<Self, S::Err> {
            let mut map = DeserializeMap::start_map(serialized)?;
            let base =
                DeserializeMap::deserialize_with_known_key(serialized, &mut map, "base", ())?;
            let data =
                DeserializeMap::deserialize_with_known_key(serialized, &mut map, "data", ())?;
            let links =
                DeserializeMap::deserialize_with_known_key(serialized, &mut map, "links", ())?;
            DeserializeMap::finish(serialized, map)?;
            Ok(InsertOneInput { base, data, links })
        }
    }

    impl DeserializeSpec for SupportedInsertLink {
        type Handler = ();
    }

    impl<'de, S> Deserialize<'de, S> for SupportedInsertLink
    where
        S: Deserializer<'de>,
        S: DeserializeMap<'de>,
        ArcSubStr: Deserialize<'de, S>,
        PartialDeserialize: Deserialize<'de, S>,
        i64: Deserialize<'de, S>,
        S: KnownKey<&'static str>,
        S::Err: From<&'static str>,
    {
        fn deserialize(_handler: Self::Handler, serialized: &mut S) -> Result<Self, S::Err> {
            let mut map = DeserializeMap::start_map(serialized)?;
            let ty: ArcSubStr =
                DeserializeMap::deserialize_with_known_key(serialized, &mut map, "ty", ())?;
            let out = match ty.as_str() {
                "set_id" => {
                    let to =
                        DeserializeMap::deserialize_with_known_key(serialized, &mut map, "to", ())?;
                    let id =
                        DeserializeMap::deserialize_with_known_key(serialized, &mut map, "id", ())?;
                    SupportedInsertLink::SetId { to, id }
                }
                "set_new" => {
                    let to =
                        DeserializeMap::deserialize_with_known_key(serialized, &mut map, "to", ())?;
                    let value = DeserializeMap::deserialize_with_known_key(
                        serialized,
                        &mut map,
                        "value",
                        (),
                    )?;
                    SupportedInsertLink::SetNew { to, value }
                }
                _ => return Err(S::Err::from("unsupported insert link ty")),
            };
            DeserializeMap::finish(serialized, map)?;
            Ok(out)
        }
    }

    impl DeserializeSpec for SupportedUpdateLink {
        type Handler = ();
    }

    impl<'de, S> Deserialize<'de, S> for SupportedUpdateLink
    where
        S: Deserializer<'de>,
        S: DeserializeMap<'de>,
        ArcSubStr: Deserialize<'de, S>,
        PartialDeserialize: Deserialize<'de, S>,
        i64: Deserialize<'de, S>,
        S: KnownKey<&'static str>,
        S::Err: From<&'static str>,
    {
        fn deserialize(_handler: Self::Handler, serialized: &mut S) -> Result<Self, S::Err> {
            let mut map = DeserializeMap::start_map(serialized)?;
            let ty: ArcSubStr =
                DeserializeMap::deserialize_with_known_key(serialized, &mut map, "ty", ())?;
            let out = match ty.as_str() {
                "set_id" => {
                    let to =
                        DeserializeMap::deserialize_with_known_key(serialized, &mut map, "to", ())?;
                    let id =
                        DeserializeMap::deserialize_with_known_key(serialized, &mut map, "id", ())?;
                    SupportedUpdateLink::SetId { to, id }
                }
                "set_new" => {
                    let to =
                        DeserializeMap::deserialize_with_known_key(serialized, &mut map, "to", ())?;
                    let value = DeserializeMap::deserialize_with_known_key(
                        serialized,
                        &mut map,
                        "value",
                        (),
                    )?;
                    SupportedUpdateLink::SetNew { to, value }
                }
                "set_null" => {
                    let to =
                        DeserializeMap::deserialize_with_known_key(serialized, &mut map, "to", ())?;
                    SupportedUpdateLink::SetNull { to }
                }
                "remove_id" => {
                    let to =
                        DeserializeMap::deserialize_with_known_key(serialized, &mut map, "to", ())?;
                    let id =
                        DeserializeMap::deserialize_with_known_key(serialized, &mut map, "id", ())?;
                    SupportedUpdateLink::RemoveId { to, id }
                }
                _ => return Err(S::Err::from("unsupported update link ty")),
            };
            DeserializeMap::finish(serialized, map)?;
            Ok(out)
        }
    }

    impl DeserializeSpec for SupportedLinkFetchMany {
        type Handler = ();
    }

    impl<'de, S> Deserialize<'de, S> for SupportedFilter
    where
        S: Deserializer<'de>,
        S: DeserializeMap<'de>,
        ArcSubStr: Deserialize<'de, S>,
        PartialDeserialize: Deserialize<'de, S>,
        S: KnownKey<&'static str>,
        S::Err: From<&'static str>,
    {
        fn deserialize(_handler: Self::Handler, serialized: &mut S) -> Result<Self, S::Err> {
            let mut map = DeserializeMap::start_map(serialized)?;
            let ty: ArcSubStr =
                DeserializeMap::deserialize_with_known_key(serialized, &mut map, "ty", ())?;
            let out = match ty.as_str() {
                "col_eq" => {
                    let col = DeserializeMap::deserialize_with_known_key(
                        serialized,
                        &mut map,
                        "col",
                        (),
                    )?;
                    let eq =
                        DeserializeMap::deserialize_with_known_key(serialized, &mut map, "eq", ())?;
                    SupportedFilter::ColEq(ColumnEqual { col, eq })
                }
                _ => return Err(S::Err::from("unsupported filter ty")),
            };
            DeserializeMap::finish(serialized, map)?;
            Ok(out)
        }
    }

    impl DeserializeSpec for SupportedFilter {
        type Handler = ();
    }

    impl<'de, S> Deserialize<'de, S> for SupportedLinkFetchMany
    where
        S: Deserializer<'de>,
        S: DeserializeMap<'de>,
        ArcSubStr: Deserialize<'de, S>,
        S: KnownKey<&'static str>,
        S::Err: From<&'static str>,
    {
        fn deserialize(_handler: Self::Handler, serialized: &mut S) -> Result<Self, S::Err> {
            let mut map = DeserializeMap::start_map(serialized)?;
            let ty: ArcSubStr =
                DeserializeMap::deserialize_with_known_key(serialized, &mut map, "ty", ())?;
            let out = match ty.as_str() {
                "optional_to_many" => {
                    let to =
                        DeserializeMap::deserialize_with_known_key(serialized, &mut map, "to", ())?;
                    SupportedLinkFetchMany::OptionalToMany { to }
                }
                "many_to_many" => {
                    let to =
                        DeserializeMap::deserialize_with_known_key(serialized, &mut map, "to", ())?;
                    SupportedLinkFetchMany::ManyToMany { to }
                }
                "timestamp" => SupportedLinkFetchMany::Timestamp,
                _ => return Err(S::Err::from("unsupported fetch link ty")),
            };
            DeserializeMap::finish(serialized, map)?;
            Ok(out)
        }
    }

    impl DeserializeSpec for AddLinkInput {
        type Handler = ();
    }

    impl<'de, S> Deserialize<'de, S> for AddLinkInput
    where
        S: Deserializer<'de>,
        S: DeserializeMap<'de>,
        ArcSubStr: Deserialize<'de, S>,
        S: KnownKey<&'static str>,
        S::Err: From<&'static str>,
    {
        fn deserialize(_handler: Self::Handler, serialized: &mut S) -> Result<Self, S::Err> {
            let mut map = DeserializeMap::start_map(serialized)?;
            let ty: ArcSubStr =
                DeserializeMap::deserialize_with_known_key(serialized, &mut map, "ty", ())?;
            let out = match ty.as_str() {
                "optional_to_many" => {
                    let from = DeserializeMap::deserialize_with_known_key(
                        serialized,
                        &mut map,
                        "from",
                        (),
                    )?;
                    let to =
                        DeserializeMap::deserialize_with_known_key(serialized, &mut map, "to", ())?;
                    AddLinkInput::OptionalToMany { from, to }
                }
                "many_to_many" => {
                    let from = DeserializeMap::deserialize_with_known_key(
                        serialized,
                        &mut map,
                        "from",
                        (),
                    )?;
                    let to =
                        DeserializeMap::deserialize_with_known_key(serialized, &mut map, "to", ())?;
                    AddLinkInput::ManyToMany { from, to }
                }
                "timestamp" => {
                    let collection = DeserializeMap::deserialize_with_known_key(
                        serialized,
                        &mut map,
                        "collection",
                        (),
                    )?;
                    AddLinkInput::Timestamp { collection }
                }
                _ => return Err(S::Err::from("unsupported add link ty")),
            };
            DeserializeMap::finish(serialized, map)?;
            Ok(out)
        }
    }

    impl DeserializeSpec for SupportedLinkFetchOne {
        type Handler = ();
    }

    impl<'de, S> Deserialize<'de, S> for SupportedLinkFetchOne
    where
        S: Deserializer<'de>,
        S: DeserializeMap<'de>,
        ArcSubStr: Deserialize<'de, S>,
        S: KnownKey<&'static str>,
        S::Err: From<&'static str>,
    {
        fn deserialize(_handler: Self::Handler, serialized: &mut S) -> Result<Self, S::Err> {
            let mut map = DeserializeMap::start_map(serialized)?;
            let ty: ArcSubStr =
                DeserializeMap::deserialize_with_known_key(serialized, &mut map, "ty", ())?;
            let out = match ty.as_str() {
                "optional_to_many" => {
                    let to =
                        DeserializeMap::deserialize_with_known_key(serialized, &mut map, "to", ())?;
                    SupportedLinkFetchOne::OptionalToMany { to }
                }
                "many_to_many" => {
                    let to =
                        DeserializeMap::deserialize_with_known_key(serialized, &mut map, "to", ())?;
                    SupportedLinkFetchOne::ManyToMany { to }
                }
                "timestamp" => SupportedLinkFetchOne::Timestamp,
                _ => return Err(S::Err::from("unsupported fetch one link ty")),
            };
            DeserializeMap::finish(serialized, map)?;
            Ok(out)
        }
    }

    impl DeserializeSpec for FetchOneInput {
        type Handler = ();
    }

    impl<'de, S> Deserialize<'de, S> for FetchOneInput
    where
        S: Deserializer<'de>,
        S: DeserializeMap<'de>,
        S: DeserializeSeq<'de>,
        ArcSubStr: Deserialize<'de, S>,
        i64: Deserialize<'de, S>,
        Vec<SupportedFilter>: Deserialize<'de, S>,
        Vec<SupportedLinkFetchOne>: Deserialize<'de, S>,
        S: KnownKey<&'static str>,
    {
        fn deserialize(_handler: Self::Handler, serialized: &mut S) -> Result<Self, S::Err> {
            let mut map = DeserializeMap::start_map(serialized)?;
            let base =
                DeserializeMap::deserialize_with_known_key(serialized, &mut map, "base", ())?;
            let id = DeserializeMap::deserialize_with_known_key(serialized, &mut map, "id", ())?;
            let filters =
                DeserializeMap::deserialize_with_known_key(serialized, &mut map, "filters", ())?;
            let links =
                DeserializeMap::deserialize_with_known_key(serialized, &mut map, "links", ())?;
            DeserializeMap::finish(serialized, map)?;
            Ok(FetchOneInput {
                base,
                id,
                filters,
                links,
            })
        }
    }

    impl DeserializeSpec for FetchManyInput {
        type Handler = ();
    }

    impl<'de, S> Deserialize<'de, S> for FetchManyInput
    where
        S: Deserializer<'de>,
        S: DeserializeMap<'de>,
        S: DeserializeSeq<'de>,
        ArcSubStr: Deserialize<'de, S>,
        Vec<SupportedLinkFetchMany>: Deserialize<'de, S>,
        Vec<SupportedFilter>: Deserialize<'de, S>,
        Pagination: Deserialize<'de, S>,
        S: KnownKey<&'static str>,
    {
        fn deserialize(_handler: Self::Handler, serialized: &mut S) -> Result<Self, S::Err> {
            let mut map = DeserializeMap::start_map(serialized)?;
            let base =
                DeserializeMap::deserialize_with_known_key(serialized, &mut map, "base", ())?;
            let filters =
                DeserializeMap::deserialize_with_known_key(serialized, &mut map, "filters", ())?;
            let links =
                DeserializeMap::deserialize_with_known_key(serialized, &mut map, "links", ())?;
            let pagination =
                DeserializeMap::deserialize_with_known_key(serialized, &mut map, "pagination", ())?;
            DeserializeMap::finish(serialized, map)?;
            Ok(FetchManyInput {
                base,
                filters,
                links,
                pagination,
            })
        }
    }

    impl DeserializeSpec for Pagination {
        type Handler = ();
    }

    impl<'de, S> Deserialize<'de, S> for Pagination
    where
        S: Deserializer<'de>,
        S: DeserializeMap<'de>,
        i64: Deserialize<'de, S>,
        Option<FirstItem>: Deserialize<'de, S>,
        Vec<OrderBy>: Deserialize<'de, S>,
        S: KnownKey<&'static str>,
    {
        fn deserialize(_handler: Self::Handler, serialized: &mut S) -> Result<Self, S::Err> {
            let mut map = DeserializeMap::start_map(serialized)?;
            let limit =
                DeserializeMap::deserialize_with_known_key(serialized, &mut map, "limit", ())?;
            let first_item =
                DeserializeMap::deserialize_with_known_key(serialized, &mut map, "first_item", ())?;
            let order_by =
                DeserializeMap::deserialize_with_known_key(serialized, &mut map, "order_by", ())?;
            DeserializeMap::finish(serialized, map)?;
            Ok(Pagination {
                limit,
                first_item,
                order_by,
            })
        }
    }

    impl DeserializeSpec for FirstItem {
        type Handler = ();
    }

    impl<'de, S> Deserialize<'de, S> for FirstItem
    where
        S: Deserializer<'de>,
        S: DeserializeMap<'de>,
        i64: Deserialize<'de, S>,
        BTreeMap<ArcSubStr, PartialDeserialize>: Deserialize<'de, S>,
        S: KnownKey<&'static str>,
    {
        fn deserialize(_handler: Self::Handler, serialized: &mut S) -> Result<Self, S::Err> {
            let mut map = DeserializeMap::start_map(serialized)?;
            let id = DeserializeMap::deserialize_with_known_key(serialized, &mut map, "id", ())?;
            let data =
                DeserializeMap::deserialize_with_known_key(serialized, &mut map, "data", ())?;
            DeserializeMap::finish(serialized, map)?;
            Ok(FirstItem { id, data })
        }
    }

    impl DeserializeSpec for BTreeMap<ArcSubStr, PartialDeserialize> {
        type Handler = ();
    }

    impl<'de, S> Deserialize<'de, S> for BTreeMap<ArcSubStr, PartialDeserialize>
    where
        S: Deserializer<'de>,
        S: DeserializeMap<'de>,
        ArcSubStr: UnknownKey<S> + Deserialize<'de, S>,
        PartialDeserialize: Deserialize<'de, S>,
    {
        fn deserialize(_handler: Self::Handler, serialized: &mut S) -> Result<Self, S::Err> {
            let mut map_access = DeserializeMap::start_map(serialized)?;
            let mut ret = BTreeMap::new();
            while DeserializeMap::map_has_next(serialized, &map_access) {
                let (key, value) = DeserializeMap::deserialize_with_unknown_key(
                    serialized,
                    &mut map_access,
                    (),
                    (),
                )?;
                ret.insert(key, value);
            }
            DeserializeMap::finish(serialized, map_access)?;
            Ok(ret)
        }
    }

    impl DeserializeSpec for OrderBy {
        type Handler = ();
    }

    impl<'de, S> Deserialize<'de, S> for OrderBy
    where
        S: Deserializer<'de>,
        S: DeserializeMap<'de>,
        ArcSubStr: Deserialize<'de, S>,
        Direction: Deserialize<'de, S>,
        S: KnownKey<&'static str>,
    {
        fn deserialize(_handler: Self::Handler, serialized: &mut S) -> Result<Self, S::Err> {
            let mut map = DeserializeMap::start_map(serialized)?;
            let col = DeserializeMap::deserialize_with_known_key(serialized, &mut map, "col", ())?;
            let direction =
                DeserializeMap::deserialize_with_known_key(serialized, &mut map, "direction", ())?;
            DeserializeMap::finish(serialized, map)?;
            Ok(OrderBy { col, direction })
        }
    }

    impl DeserializeSpec for Direction {
        type Handler = ();
    }

    impl<'de, S> Deserialize<'de, S> for Direction
    where
        S: Deserializer<'de>,
        String: Deserialize<'de, S>,
        S::Err: From<&'static str>,
    {
        fn deserialize(_handler: Self::Handler, serialized: &mut S) -> Result<Self, S::Err> {
            match String::deserialize((), serialized)?.as_str() {
                "asc" => Ok(Direction::Asc),
                "desc" => Ok(Direction::Desc),
                _ => Err(S::Err::from("unsupported order direction")),
            }
        }
    }

    impl DeserializeSpec for UpdateOneInput {
        type Handler = ();
    }

    impl<'de, S> Deserialize<'de, S> for UpdateOneInput
    where
        S: Deserializer<'de>,
        S: DeserializeMap<'de>,
        S: DeserializeSeq<'de>,
        ArcSubStr: Deserialize<'de, S>,
        PartialDeserialize: Deserialize<'de, S>,
        Vec<SupportedUpdateLink>: Deserialize<'de, S>,
        i64: Deserialize<'de, S>,
        S: KnownKey<&'static str>,
    {
        fn deserialize(_handler: Self::Handler, serialized: &mut S) -> Result<Self, S::Err> {
            let mut map = DeserializeMap::start_map(serialized)?;
            let base =
                DeserializeMap::deserialize_with_known_key(serialized, &mut map, "base", ())?;
            let id = DeserializeMap::deserialize_with_known_key(serialized, &mut map, "id", ())?;
            let data =
                DeserializeMap::deserialize_with_known_key(serialized, &mut map, "data", ())?;
            let links =
                DeserializeMap::deserialize_with_known_key(serialized, &mut map, "links", ())?;
            DeserializeMap::finish(serialized, map)?;
            Ok(UpdateOneInput {
                base,
                id,
                data,
                links,
            })
        }
    }

    impl DeserializeSpec for DeleteOneInput {
        type Handler = ();
    }

    impl DeserializeSpec for SupportedDeleteLink {
        type Handler = ();
    }

    impl<'de, S> Deserialize<'de, S> for SupportedDeleteLink
    where
        S: Deserializer<'de>,
        S: DeserializeMap<'de>,
        ArcSubStr: Deserialize<'de, S>,
        S: KnownKey<&'static str>,
        S::Err: From<&'static str>,
    {
        fn deserialize(_handler: Self::Handler, serialized: &mut S) -> Result<Self, S::Err> {
            let mut map = DeserializeMap::start_map(serialized)?;
            let ty: ArcSubStr =
                DeserializeMap::deserialize_with_known_key(serialized, &mut map, "ty", ())?;
            let out = match ty.as_str() {
                "optional_to_many" => {
                    let to =
                        DeserializeMap::deserialize_with_known_key(serialized, &mut map, "to", ())?;
                    SupportedDeleteLink::OptionalToMany { to }
                }
                "many_to_many" => {
                    let to =
                        DeserializeMap::deserialize_with_known_key(serialized, &mut map, "to", ())?;
                    SupportedDeleteLink::ManyToMany { to }
                }
                _ => return Err(S::Err::from("unsupported delete link ty")),
            };
            DeserializeMap::finish(serialized, map)?;
            Ok(out)
        }
    }

    impl<'de, S> Deserialize<'de, S> for DeleteOneInput
    where
        S: Deserializer<'de>,
        S: DeserializeMap<'de>,
        S: DeserializeSeq<'de>,
        ArcSubStr: Deserialize<'de, S>,
        Vec<SupportedDeleteLink>: Deserialize<'de, S>,
        i64: Deserialize<'de, S>,
        S: KnownKey<&'static str>,
    {
        fn deserialize(_handler: Self::Handler, serialized: &mut S) -> Result<Self, S::Err> {
            let mut map = DeserializeMap::start_map(serialized)?;
            let base =
                DeserializeMap::deserialize_with_known_key(serialized, &mut map, "base", ())?;
            let id = DeserializeMap::deserialize_with_known_key(serialized, &mut map, "id", ())?;
            let links =
                DeserializeMap::deserialize_with_known_key(serialized, &mut map, "links", ())?;
            DeserializeMap::finish(serialized, map)?;
            Ok(DeleteOneInput { base, id, links })
        }
    }
}

mod add_collection_mod {
    use std::{future::Future, sync::Arc};

    use sqlx::IntoArguments;

    use crate::{
        database_extention::DatabaseExt,
        fix_executor::ExecutorTrait,
        json_client::{
            client_interface::{AddCollectionError, AddCollectionInput, AddCollectionOutput},
            dynamic_collection::{DynamicCollection, FieldName},
            sqlx_executor::SqlxExecutorData,
        },
        on_migrate::OnMigrate,
        query_builder::{Expression, StatementBuilder},
    };

    pub fn add_collection<S>(
        this: Arc<SqlxExecutorData<S>>,
        input: AddCollectionInput,
    ) -> impl Future<Output = Result<AddCollectionOutput, AddCollectionError>> + 'static + Send + use<S>
    where
        S: Sync + DatabaseExt + ExecutorTrait,
        bool: for<'d> sqlx::Decode<'d, S> + sqlx::Type<S> + for<'q> sqlx::Encode<'q, S>,
        std::string::String:
            sqlx::Type<S> + for<'q> sqlx::Encode<'q, S> + for<'d> sqlx::Decode<'d, S>,
        DynamicCollection<S>: OnMigrate<Statements: Expression<'static, S>>,
        for<'a> &'a str: sqlx::ColumnIndex<<S as sqlx::Database>::Row>,
        for<'a> S::Arguments<'a>: IntoArguments<'a, S>,
    {
        async move {
            let dc = DynamicCollection::try_from(input)
                .map_err(|_| AddCollectionError::InvalidCollectionInput)?;

            let collection_key: Arc<str> = Arc::clone(&dc.collection_name.snake_case);

            {
                let collections = this.collections.read().await;
                if collections.get(collection_key.as_ref()).is_some() {
                    return Err(AddCollectionError::CollectionAlreadyExists);
                }
            }

            let mig = StatementBuilder::<S>::new_no_data(dc.statments()).expect("bug: {}");

            let mut conn = this.pool.acquire().await.expect("bug: {}");

            S::execute(&mut conn, mig.as_str())
                .await
                .expect("bug: migration should never fail");

            let mut collections = this.collections.write().await;
            let mut migration = this.migration.write().await;

            migration.push(mig);
            collections.insert(collection_key, tokio::sync::RwLock::new(Arc::new(dc)));

            Ok(())
        }
    }
}

mod add_link_mod {
    use std::sync::Arc;

    use crate::{
        database_extention::DatabaseExt,
        fix_executor::ExecutorTrait,
        json_client::{
            client_interface::{AddLinkError, AddLinkInput, AddLinkOutput},
            dynamic_collection::DynamicCollection,
            sqlx_executor::{FromTo, SqlxExecutorData},
        },
        links::{
            DefaultRelationKey, relation_many_to_many::ManyToMany,
            relation_optional_to_many::OptionalToMany, timestamp::Timestamp,
        },
        on_migrate::OnMigrate,
        query_builder::{Expression, StatementBuilder},
    };

    pub fn add_link<S>(
        this: Arc<SqlxExecutorData<S>>,
        input: AddLinkInput,
    ) -> impl Future<Output = Result<AddLinkOutput, AddLinkError>> + 'static + Send + use<S>
    where
        S: DatabaseExt + Sync + Send + ExecutorTrait,
        OptionalToMany<DefaultRelationKey, Arc<DynamicCollection<S>>, Arc<DynamicCollection<S>>>:
            OnMigrate<Statements: for<'q> Expression<'q, S>>,
        ManyToMany<DefaultRelationKey, Arc<DynamicCollection<S>>, Arc<DynamicCollection<S>>>:
            OnMigrate<Statements: for<'q> Expression<'q, S>>,
        Timestamp<Arc<DynamicCollection<S>>>: OnMigrate<Statements: for<'q> Expression<'q, S>>,
    {
        async move {
            match input {
                AddLinkInput::OptionalToMany { from, to } => {
                    {
                        let li_read = this.link_info.read().await;
                        if li_read.optional_to_many.contains(&FromTo {
                            from: from.detach(),
                            to: to.detach(),
                        }) {
                            return Err(AddLinkError::LinkAlreadyExists);
                        }
                    }

                    let collections = this.collections.read().await;
                    let from_col = collections
                        .get(from.as_str())
                        .ok_or(AddLinkError::CollectionNotFound)?
                        .read()
                        .await
                        .clone();
                    let to_col = collections
                        .get(to.as_str())
                        .ok_or(AddLinkError::CollectionNotFound)?
                        .read()
                        .await
                        .clone();
                    drop(collections);

                    let mig =
                        StatementBuilder::<S>::new_no_data(OnMigrate::statments(&OptionalToMany {
                            fk_unique_id: DefaultRelationKey,
                            from: from_col,
                            to: to_col,
                        }))
                        .expect("bug: {}");

                    let mut conn = this.pool.acquire().await.unwrap();
                    S::execute(&mut *conn, mig.as_str()).await.unwrap();

                    let mut migration = this.migration.write().await;
                    let mut li_write = this.link_info.write().await;
                    migration.push(mig);
                    li_write.optional_to_many.insert(FromTo {
                        from: from.detach(),
                        to: to.detach(),
                    });

                    Ok(())
                }
                AddLinkInput::ManyToMany { from, to } => {
                    {
                        let li_read = this.link_info.read().await;
                        if li_read.many_to_many.contains(&FromTo {
                            from: from.detach(),
                            to: to.detach(),
                        }) {
                            return Err(AddLinkError::LinkAlreadyExists);
                        }
                    }

                    let collections = this.collections.read().await;
                    let from_col = collections
                        .get(from.as_str())
                        .ok_or(AddLinkError::CollectionNotFound)?
                        .read()
                        .await
                        .clone();
                    let to_col = collections
                        .get(to.as_str())
                        .ok_or(AddLinkError::CollectionNotFound)?
                        .read()
                        .await
                        .clone();
                    drop(collections);

                    let mig =
                        StatementBuilder::<S>::new_no_data(OnMigrate::statments(&ManyToMany {
                            relation_key: DefaultRelationKey,
                            from: from_col,
                            to: to_col,
                        }))
                        .expect("bug: many_to_many migration contains bind parameters");

                    let mut conn = this.pool.acquire().await.unwrap();
                    S::execute(&mut *conn, mig.as_str()).await.unwrap();

                    let mut migration = this.migration.write().await;
                    let mut li_write = this.link_info.write().await;
                    migration.push(mig);
                    li_write.many_to_many.insert(FromTo {
                        from: from.detach(),
                        to: to.detach(),
                    });

                    Ok(())
                }
                AddLinkInput::Timestamp { collection } => {
                    {
                        let li_read = this.link_info.read().await;
                        if li_read.timestamped.contains(collection.as_str()) {
                            return Err(AddLinkError::LinkAlreadyExists);
                        }
                    }

                    let collections = this.collections.read().await;
                    let col = collections
                        .get(collection.as_str())
                        .ok_or(AddLinkError::CollectionNotFound)?
                        .read()
                        .await
                        .clone();
                    drop(collections);

                    let mig =
                        StatementBuilder::<S>::new_no_data(OnMigrate::statments(&Timestamp {
                            collection: col,
                        }))
                        .expect("bug: timestamp migration contains bind parameters");

                    let mut conn = this
                        .pool
                        .acquire()
                        .await
                        .expect("dev_ops: acquire connection");
                    S::execute(&mut *conn, mig.as_str())
                        .await
                        .expect("bug: timestamp migration failed");

                    let mut migration = this.migration.write().await;
                    let mut link_info = this.link_info.write().await;
                    migration.push(mig);
                    link_info.timestamped.insert(collection.detach());

                    Ok(())
                }
            }
        }
    }
}

mod update_one_mod {
    use std::sync::Arc;

    use sqlx::ColumnIndex;

    use crate::{
        collections::Collection,
        database_extention::DatabaseExt,
        expressions::ColumnEqual,
        extentions::common_expressions::Scoped,
        fix_executor::ExecutorTrait,
        from_row::FromRowAlias,
        gen_serde::{
            Deserialize, deserialize,
            json_format_side::{JsonAsArcCursor, JsonFormat},
        },
        json_client::{
            DynManyToMany, DynOptionalToMany,
            client_interface::{
                SupportedUpdateLink, UpdateOneError, UpdateOneInput, UpdateOneOutput,
            },
            dynamic_collection::{
                CollectionToSerialize, DynamicCollection, DynamicInsertInput, DynamicUpdateInput,
            },
            sqlx_executor::{FromTo, SqlxExecutorData},
            update_one_trait_extension::{JsonUpdateOneLink, JsonUpdateOneToConsume},
        },
        links::{
            DefaultRelationKey,
            relation_many_to_many::{ManyToMany, RemoveJunctionId, SetJunctionId},
            relation_optional_to_many::OptionalToMany,
            update_links::{SetId, SetNew},
        },
        operations::{
            Operation,
            update::{Update, UpdateLink, UpdateLinkData, UpdateLinkSplit},
        },
    };

    type DynCollection<S> = Arc<DynamicCollection<S>>;

    pub fn update_one<S>(
        this: Arc<SqlxExecutorData<S>>,
        input: UpdateOneInput,
    ) -> impl Future<Output = Result<UpdateOneOutput, UpdateOneError>> + 'static + Send + use<S>
    where
        i64: sqlx::Type<S> + for<'q> sqlx::Decode<'q, S> + for<'q> sqlx::Encode<'q, S>,
        for<'a> &'a str: sqlx::ColumnIndex<<S as sqlx::Database>::Row>,
        S: sqlx::Database + DatabaseExt + ExecutorTrait + Send + Sync + 'static,
        DynCollection<S>: for<'r> FromRowAlias<'r, S::Row, RData = CollectionToSerialize>,
        DynamicUpdateInput<S>: for<'d> Deserialize<'d, JsonAsArcCursor, Handler = DynCollection<S>>,
        DynamicInsertInput<S>: for<'d> Deserialize<'d, JsonAsArcCursor, Handler = DynCollection<S>>,
        SetId<DynOptionalToMany<S>, Option<i64>>: UpdateLinkSplit<
            Link: JsonUpdateOneLink<S>
                      + UpdateLink<
                InitSplitForPreOp: Send + 'static,
                InitSplitForWheres: Send + 'static,
                InitSplitForUpdateValues: Send + 'static,
                InitSplitPostOp: Send + 'static,
            >,
        >,
        SetJunctionId<DefaultRelationKey, DynCollection<S>, DynCollection<S>>: UpdateLinkSplit<
            Link: JsonUpdateOneLink<S>
                      + UpdateLink<
                InitSplitForPreOp: Send + 'static,
                InitSplitForWheres: Send + 'static,
                InitSplitForUpdateValues: Send + 'static,
                InitSplitPostOp: Send + 'static,
            >,
        >,
        RemoveJunctionId<DefaultRelationKey, DynCollection<S>, DynCollection<S>>: UpdateLinkSplit<
            Link: JsonUpdateOneLink<S>
                      + UpdateLink<
                InitSplitForPreOp: Send + 'static,
                InitSplitForWheres: Send + 'static,
                InitSplitForUpdateValues: Send + 'static,
                InitSplitPostOp: Send + 'static,
            >,
        >,
        SetNew<DynOptionalToMany<S>, DynamicInsertInput<S>>: UpdateLinkSplit<
            Link: JsonUpdateOneLink<S>
                      + UpdateLink<
                InitSplitForPreOp: Send + 'static,
                InitSplitForWheres: Send + 'static,
                InitSplitForUpdateValues: Send + 'static,
                InitSplitPostOp: Send + 'static,
            >,
        >,
        Vec<JsonUpdateOneToConsume<S>>:
            UpdateLinkSplit<Link = Vec<Box<dyn JsonUpdateOneLink<S> + Send>>>,
    {
        async move {
            let cols = this.collections.read().await;
            let base_gaurd = cols
                .get(input.base.as_str())
                .ok_or(UpdateOneError::CollectionNotFound)?
                .read()
                .await;
            let base = base_gaurd.clone();
            let mut all_gaurds = vec![base_gaurd];

            let data: DynamicUpdateInput<S> = deserialize(
                Arc::from(input.data.0.as_str()),
                Arc::clone(&base),
                JsonFormat,
            )
            .map_err(|_| UpdateOneError::InvalidData)?;

            if data.0.is_empty() && input.links.is_empty() {
                return Err(UpdateOneError::InvalidData);
            }

            let rel_guard = this.link_info.read().await;
            let mut links = Vec::<JsonUpdateOneToConsume<S>>::new();

            for link in input.links {
                match link {
                    SupportedUpdateLink::SetId { to, id } => {
                        let to_gaurd = cols
                            .get(to.as_str())
                            .ok_or(UpdateOneError::InvalidLink)?
                            .read()
                            .await;
                        let to = to_gaurd.clone();
                        all_gaurds.push(to_gaurd);

                        let forward = FromTo {
                            from: Arc::clone(&base.collection_name.snake_case),
                            to: Arc::clone(&to.collection_name.snake_case),
                        };

                        if rel_guard.many_to_many.contains(&forward) {
                            let (link, data) = SetJunctionId {
                                relation: ManyToMany {
                                    relation_key: DefaultRelationKey,
                                    from: base.clone(),
                                    to,
                                },
                                from_id: input.id,
                                to_id: id,
                            }
                            .init_split();
                            links.push(JsonUpdateOneToConsume {
                                link: Box::new(link),
                                data: UpdateLinkData {
                                    wheres: Box::new(data.wheres),
                                    update_values: Box::new(data.update_values),
                                    pre_op: Box::new(data.pre_op),
                                    post_op: Box::new(data.post_op),
                                },
                            });
                        } else if rel_guard.optional_to_many.contains(&forward) {
                            let (link, data) = SetId {
                                relation: OptionalToMany {
                                    fk_unique_id: DefaultRelationKey,
                                    from: base.clone(),
                                    to,
                                },
                                id: Some(id),
                            }
                            .init_split();
                            links.push(JsonUpdateOneToConsume {
                                link: Box::new(link),
                                data: UpdateLinkData {
                                    wheres: Box::new(data.wheres),
                                    update_values: Box::new(data.update_values),
                                    pre_op: Box::new(data.pre_op),
                                    post_op: Box::new(data.post_op),
                                },
                            });
                        } else {
                            return Err(UpdateOneError::InvalidLink);
                        }
                    }
                    SupportedUpdateLink::SetNew { to, value } => {
                        let to_gaurd = cols
                            .get(to.as_str())
                            .ok_or(UpdateOneError::InvalidLink)?
                            .read()
                            .await;
                        let to = to_gaurd.clone();
                        all_gaurds.push(to_gaurd);
                        let link_data: DynamicInsertInput<S> =
                            deserialize(Arc::from(value.0.as_str()), Arc::clone(&to), JsonFormat)
                                .map_err(|_| UpdateOneError::InvalidData)?;
                        let (link, data) = SetNew {
                            relation: OptionalToMany {
                                fk_unique_id: DefaultRelationKey,
                                from: base.clone(),
                                to,
                            },
                            data: link_data,
                        }
                        .init_split();
                        links.push(JsonUpdateOneToConsume {
                            link: Box::new(link),
                            data: UpdateLinkData {
                                wheres: Box::new(data.wheres),
                                update_values: Box::new(data.update_values),
                                pre_op: Box::new(data.pre_op),
                                post_op: Box::new(data.post_op),
                            },
                        })
                    }
                    SupportedUpdateLink::SetNull { to } => {
                        let to_gaurd = cols
                            .get(to.as_str())
                            .ok_or(UpdateOneError::InvalidLink)?
                            .read()
                            .await;
                        let to = to_gaurd.clone();
                        all_gaurds.push(to_gaurd);
                        let (link, data) = SetId {
                            relation: OptionalToMany {
                                fk_unique_id: DefaultRelationKey,
                                from: base.clone(),
                                to,
                            },
                            id: None,
                        }
                        .init_split();
                        links.push(JsonUpdateOneToConsume {
                            link: Box::new(link),
                            data: UpdateLinkData {
                                wheres: Box::new(data.wheres),
                                update_values: Box::new(data.update_values),
                                pre_op: Box::new(data.pre_op),
                                post_op: Box::new(data.post_op),
                            },
                        })
                    }
                    SupportedUpdateLink::RemoveId { to, id } => {
                        let to_gaurd = cols
                            .get(to.as_str())
                            .ok_or(UpdateOneError::InvalidLink)?
                            .read()
                            .await;
                        let to = to_gaurd.clone();
                        all_gaurds.push(to_gaurd);

                        let forward = FromTo {
                            from: Arc::clone(&base.collection_name.snake_case),
                            to: Arc::clone(&to.collection_name.snake_case),
                        };

                        if !rel_guard.many_to_many.contains(&forward) {
                            return Err(UpdateOneError::InvalidLink);
                        }

                        let (link, data) = RemoveJunctionId {
                            relation: ManyToMany {
                                relation_key: DefaultRelationKey,
                                from: base.clone(),
                                to,
                            },
                            from_id: input.id,
                            to_id: id,
                        }
                        .init_split();
                        links.push(JsonUpdateOneToConsume {
                            link: Box::new(link),
                            data: UpdateLinkData {
                                wheres: Box::new(data.wheres),
                                update_values: Box::new(data.update_values),
                                pre_op: Box::new(data.pre_op),
                                post_op: Box::new(data.post_op),
                            },
                        })
                    }
                }
            }

            let mut conn = this.pool.acquire().await.unwrap();

            let out = Operation::<S>::exec_operation(
                Update {
                    base: Arc::clone(&base),
                    partial: data,
                    wheres: ColumnEqual {
                        col: base.id().scoped(),
                        eq: input.id,
                    },
                    links,
                },
                &mut conn,
            )
            .await
            .expect("bug: update one failed");

            drop(all_gaurds);
            drop(rel_guard);
            drop(cols);

            let Some(updated) = out.into_iter().find(|row| row.id == input.id) else {
                return Err(UpdateOneError::NotFound);
            };

            Ok(UpdateOneOutput {
                id: updated.id,
                attributes: updated.attributes,
                links: updated.links,
            })
        }
    }
}

mod delete_one_mod {
    use std::sync::Arc;

    use sqlx::ColumnIndex;

    use crate::{
        collections::Collection,
        database_extention::DatabaseExt,
        expressions::ColumnEqual,
        extentions::common_expressions::Scoped,
        fix_executor::ExecutorTrait,
        from_row::FromRowAlias,
        json_client::{
            DynManyToMany, DynOptionalToMany,
            client_interface::{
                DeleteOneError, DeleteOneInput, DeleteOneOutput, SupportedDeleteLink,
            },
            delete_one_trait_extension::{JsonDeleteOneLink, JsonDeleteOneToConsume},
            dynamic_collection::{CollectionToSerialize, DynamicCollection},
            sqlx_executor::{FromTo, SqlxExecutorData},
        },
        links::{
            DefaultRelationKey,
            relation_many_to_many::{DeleteManyToManyLinked, ManyToMany},
            relation_optional_to_many::OptionalToMany,
        },
        operations::{
            Operation,
            delete::{Delete, DeleteLink, DeleteLinkSplit},
        },
    };

    type DynCollection<S> = Arc<DynamicCollection<S>>;

    pub fn delete_one<S>(
        this: Arc<SqlxExecutorData<S>>,
        input: DeleteOneInput,
    ) -> impl Future<Output = Result<DeleteOneOutput, DeleteOneError>> + 'static + Send + use<S>
    where
        i64: sqlx::Type<S> + for<'q> sqlx::Decode<'q, S> + for<'q> sqlx::Encode<'q, S>,
        for<'a> &'a str: sqlx::ColumnIndex<<S as sqlx::Database>::Row>,
        S: sqlx::Database + DatabaseExt + ExecutorTrait + Send + Sync + 'static,
        DynCollection<S>: for<'r> FromRowAlias<'r, S::Row, RData = CollectionToSerialize>,
        DynOptionalToMany<S>: DeleteLinkSplit<
                InitSplitForPreOp: Send + 'static,
                Link: JsonDeleteOneLink<S>
                          + DeleteLink<
                    InitSplitForWheres: Send + 'static,
                    PreOpSplitTake: Send + 'static,
                >,
            >,
        DeleteManyToManyLinked<DefaultRelationKey, DynCollection<S>, DynCollection<S>>:
            DeleteLinkSplit<
                    InitSplitForPreOp: Send + 'static,
                    Link: JsonDeleteOneLink<S>
                              + DeleteLink<
                        InitSplitForWheres: Send + 'static,
                        PreOpSplitTake: Send + 'static,
                    >,
                >,
        Vec<JsonDeleteOneToConsume<S>>: DeleteLinkSplit<
                Link = Vec<Box<dyn JsonDeleteOneLink<S> + Send>>,
                InitSplitForPreOp: Send
                                       + 'static
                                       + IntoIterator<Item = Box<dyn std::any::Any + Send>>,
            >,
    {
        async move {
            let cols = this.collections.read().await;
            let base_gaurd = cols
                .get(input.base.as_str())
                .ok_or(DeleteOneError::CollectionNotFound)?
                .read()
                .await;
            let base = base_gaurd.clone();
            let mut all_gaurds = vec![base_gaurd];

            let rel_guard = this.link_info.read().await;
            let mut links = Vec::<JsonDeleteOneToConsume<S>>::new();

            for link in input.links {
                match link {
                    SupportedDeleteLink::OptionalToMany { to } => {
                        let to_gaurd = cols
                            .get(to.as_str())
                            .ok_or(DeleteOneError::InvalidLink)?
                            .read()
                            .await;
                        let to = to_gaurd.clone();
                        all_gaurds.push(to_gaurd);
                        links.push(JsonDeleteOneToConsume::from_split(OptionalToMany {
                            fk_unique_id: DefaultRelationKey,
                            from: base.clone(),
                            to,
                        }))
                    }
                    SupportedDeleteLink::ManyToMany { to } => {
                        let to_gaurd = cols
                            .get(to.as_str())
                            .ok_or(DeleteOneError::InvalidLink)?
                            .read()
                            .await;
                        let to = to_gaurd.clone();
                        all_gaurds.push(to_gaurd);

                        let forward = FromTo {
                            from: Arc::clone(&base.collection_name.snake_case),
                            to: Arc::clone(&to.collection_name.snake_case),
                        };

                        if !rel_guard.many_to_many.contains(&forward) {
                            return Err(DeleteOneError::InvalidLink);
                        }

                        links.push(JsonDeleteOneToConsume::from_split(DeleteManyToManyLinked {
                            link: ManyToMany {
                                relation_key: DefaultRelationKey,
                                from: base.clone(),
                                to,
                            },
                            from_id: input.id,
                        }))
                    }
                }
            }

            let mut conn = this.pool.acquire().await.unwrap();

            let out = Operation::<S>::exec_operation(
                Delete {
                    base: Arc::clone(&base),
                    wheres: ColumnEqual {
                        col: base.id().scoped(),
                        eq: input.id,
                    },
                    links,
                },
                &mut conn,
            )
            .await;

            drop(all_gaurds);
            drop(rel_guard);
            drop(cols);

            let Some(deleted) = out.into_iter().find(|row| row.id == input.id) else {
                return Err(DeleteOneError::NotFound);
            };

            Ok(DeleteOneOutput {
                id: deleted.id,
                attributes: deleted.attributes,
                links: deleted.links,
            })
        }
    }
}

mod insert_one_mod {
    use std::sync::Arc;

    use sqlx::ColumnIndex;

    use crate::{
        collections::AutoGenerate,
        database_extention::DatabaseExt,
        fix_executor::ExecutorTrait,
        from_row::FromRowAlias,
        gen_serde::{
            Deserialize, deserialize,
            json_format_side::{JsonAsArcCursor, JsonFormat},
        },
        json_client::{
            DynManyToMany, DynOptionalToMany,
            client_interface::{
                InsertOneError, InsertOneInput, InsertOneOutput, SupportedInsertLink,
            },
            dynamic_collection::{CollectionToSerialize, DynamicCollection, DynamicInsertInput},
            insert_one_trait_extension::{JsonInsertOneLink, JsonInsertOneToConsume},
            sqlx_executor::{FromTo, SqlxExecutorData},
        },
        links::{
            DefaultRelationKey,
            relation_many_to_many::ManyToMany,
            relation_optional_to_many::OptionalToMany,
            update_links::{SetId, SetNew},
        },
        operations::{
            Operation,
            insert_one::{InsertLinkConsumeData, InsertOne, InsertOneLink},
        },
    };

    type DynCollection<S> = Arc<DynamicCollection<S>>;

    pub fn insert_one<S>(
        this: Arc<SqlxExecutorData<S>>,
        input: InsertOneInput,
    ) -> impl Future<Output = Result<InsertOneOutput, InsertOneError>> + 'static + Send + use<S>
    where
        i64: sqlx::Type<S> + for<'q> sqlx::Decode<'q, S> + for<'q> sqlx::Encode<'q, S>,
        for<'a> &'a str: sqlx::ColumnIndex<<S as sqlx::Database>::Row>,
        S: sqlx::Database + DatabaseExt + ExecutorTrait + Send + Sync + 'static,
        DynCollection<S>: for<'r> FromRowAlias<'r, S::Row, RData = CollectionToSerialize>,
        DynamicInsertInput<S>: for<'d> Deserialize<'d, JsonAsArcCursor, Handler = DynCollection<S>>,
        SetId<DynOptionalToMany<S>, i64>: InsertLinkConsumeData<
            Link: JsonInsertOneLink<S>
                      + InsertOneLink<InsertValuesData: Send, PreOpData: Send, PostOpData: Send>,
        >,
        SetId<DynManyToMany<S>, i64>: InsertLinkConsumeData<
            Link: JsonInsertOneLink<S>
                      + InsertOneLink<InsertValuesData: Send, PreOpData: Send, PostOpData: Send>,
        >,
        SetNew<DynOptionalToMany<S>, DynamicInsertInput<S>>: InsertLinkConsumeData<
            Link: JsonInsertOneLink<S>
                      + InsertOneLink<InsertValuesData: Send, PreOpData: Send, PostOpData: Send>,
        >,
        Vec<JsonInsertOneToConsume<S>>:
            InsertLinkConsumeData<Link = Vec<Box<dyn JsonInsertOneLink<S> + Send>>>,
    {
        async move {
            let cols = this.collections.read().await;
            let base_gaurd = cols
                .get(input.base.as_str())
                .ok_or(InsertOneError::CollectionNotFound)?
                .read()
                .await;
            let base = base_gaurd.clone();
            let mut all_gaurds = vec![base_gaurd];

            let data: DynamicInsertInput<S> = deserialize(
                Arc::from(input.data.0.as_str()),
                Arc::clone(&base),
                JsonFormat,
            )
            .map_err(|_| InsertOneError::InvalidData)?;

            let rel_guard = this.link_info.read().await;
            let mut links = Vec::<JsonInsertOneToConsume<S>>::new();

            for link in input.links {
                match link {
                    SupportedInsertLink::SetId { to, id } => {
                        let to_gaurd = cols
                            .get(to.as_str())
                            .ok_or(InsertOneError::InvalidLink)?
                            .read()
                            .await;
                        let to = to_gaurd.clone();
                        all_gaurds.push(to_gaurd);

                        let forward = FromTo {
                            from: Arc::clone(&base.collection_name.snake_case),
                            to: Arc::clone(&to.collection_name.snake_case),
                        };

                        if rel_guard.many_to_many.contains(&forward) {
                            links.push(JsonInsertOneToConsume::new(SetId {
                                relation: ManyToMany {
                                    relation_key: DefaultRelationKey,
                                    from: base.clone(),
                                    to,
                                },
                                id,
                            }))
                        } else if rel_guard.optional_to_many.contains(&forward) {
                            links.push(JsonInsertOneToConsume::new(SetId {
                                relation: OptionalToMany {
                                    fk_unique_id: DefaultRelationKey,
                                    from: base.clone(),
                                    to,
                                },
                                id,
                            }))
                        } else {
                            return Err(InsertOneError::InvalidLink);
                        }
                    }
                    SupportedInsertLink::SetNew { to, value } => {
                        let to_gaurd = cols
                            .get(to.as_str())
                            .ok_or(InsertOneError::InvalidLink)?
                            .read()
                            .await;
                        let to = to_gaurd.clone();
                        all_gaurds.push(to_gaurd);
                        let link_data: DynamicInsertInput<S> =
                            deserialize(Arc::from(value.0.as_str()), Arc::clone(&to), JsonFormat)
                                .map_err(|_| InsertOneError::InvalidData)?;
                        links.push(JsonInsertOneToConsume::new(SetNew {
                            relation: OptionalToMany {
                                fk_unique_id: DefaultRelationKey,
                                from: base.clone(),
                                to,
                            },
                            data: link_data,
                        }))
                    }
                }
            }

            let mut conn = this.pool.acquire().await.unwrap();

            let out = Operation::<S>::exec_operation(
                InsertOne {
                    id: AutoGenerate,
                    base,
                    data,
                    links,
                },
                &mut conn,
            )
            .await
            .expect("bug: insert one failed");
            drop(all_gaurds);
            drop(rel_guard);
            drop(cols);
            Ok(InsertOneOutput {
                id: out.id,
                attributes: out.attributes,
                links: out.links,
            })
        }
    }
}

pub mod insert_one_trait_extension {
    type TodoFix = std::convert::Infallible;
    impl crate::operations::OperationOutput for TodoFix {
        type Output = TodoFix;
    }
    impl crate::from_row::FromRowData for TodoFix {
        type RData = TodoFix;
    }
    use std::any::Any;

    use sqlx::Database;

    use crate::{
        database_extention::DatabaseExt,
        extentions::common_expressions::raw_from_row::RawFromRow,
        fix_executor::ExecutorTrait,
        from_row::FromRowData,
        gen_serde::{Serialize, json_serialize_side::JsonAsString},
        operations::{
            Operation, OperationOutput,
            boxed_operation::BoxedOperation,
            insert_one::{
                ConstraintViolation, InsertLinkConsumeData, InsertLinkData, InsertOneLink,
            },
        },
        query_builder::{ManyBoxedExpressions, functional_expr::ManyFlat},
    };

    /// [`FromRowAlias`] that decodes one sub-row per link.
    pub struct JsonInsertLinksFromRow<S>(pub Vec<Box<dyn RawFromRow<S> + Send>>);

    impl<S: Database> FromRowData for JsonInsertLinksFromRow<S> {
        type RData = Vec<Box<dyn Any + Send>>;
    }

    impl<'r, S: Database> crate::from_row::FromRowAlias<'r, S::Row> for JsonInsertLinksFromRow<S> {
        fn no_alias(&self, row: &'r S::Row) -> Result<Self::RData, crate::from_row::FromRowError> {
            self.0
                .iter()
                .map(|from_row: &Box<dyn RawFromRow<S> + Send>| {
                    RawFromRow::dyn_no_alias(&**from_row, row)
                })
                .collect()
        }

        fn pre_alias(
            &self,
            row: crate::from_row::RowPreAliased<'r, S::Row>,
        ) -> Result<Self::RData, crate::from_row::FromRowError>
        where
            S::Row: sqlx::Row,
        {
            self.0
                .iter()
                .map(|from_row: &Box<dyn RawFromRow<S> + Send>| {
                    RawFromRow::dyn_pre_alias(&**from_row, row.clone())
                })
                .collect()
        }

        fn post_alias(
            &self,
            _: crate::from_row::RowPostAliased<'r, S::Row>,
        ) -> Result<Self::RData, crate::from_row::FromRowError>
        where
            S::Row: sqlx::Row,
        {
            panic!("to be deprecated")
        }

        fn two_alias(
            &self,
            row: crate::from_row::RowTwoAliased<'r, S::Row>,
        ) -> Result<Self::RData, crate::from_row::FromRowError>
        where
            S::Row: sqlx::Row,
        {
            self.0
                .iter()
                .map(|from_row: &Box<dyn RawFromRow<S> + Send>| {
                    RawFromRow::dyn_two_alias(&**from_row, row.clone())
                })
                .collect()
        }
    }

    pub struct JsonInsertOneToConsume<S> {
        link: Box<dyn JsonInsertOneLink<S> + Send>,
        data: InsertLinkData<Box<dyn Any + Send>, Box<dyn Any + Send>, Box<dyn Any + Send>>,
    }

    impl<S> JsonInsertOneToConsume<S> {
        pub fn new<T>(pre_data: T) -> Self
        where
            S: sqlx::Database,
            T: InsertLinkConsumeData,
            T::Link: Send + 'static,
            <T::Link as InsertOneLink>::InsertValuesData: 'static + Send,
            <T::Link as InsertOneLink>::PreOpData: 'static + Send,
            <T::Link as InsertOneLink>::PostOpData: 'static + Send,
            T::Link: JsonInsertOneLink<S>,
        {
            let (link, data) = pre_data.consume_data();
            Self {
                link: Box::new(link),
                data: InsertLinkData {
                    insert_value_data: Box::new(data.insert_value_data),
                    pre_op_data: Box::new(data.pre_op_data),
                    post_op_data: Box::new(data.post_op_data),
                },
            }
        }
    }

    pub trait JsonInsertOneLink<S: Database>: Send + Sync + 'static {
        fn dyn_pre_op_init(&self, input: Box<dyn Any + Send>) -> Box<dyn BoxedOperation<S> + Send>;
        fn dyn_pre_op_split(
            &self,
            pre_op_output: Box<dyn Any + Send>,
        ) -> Result<
            (
                Box<dyn Any + Send>,
                Box<dyn Any + Send>,
                Box<dyn Any + Send>,
            ),
            ConstraintViolation,
        >;
        fn dyn_insert_names(&self) -> Box<dyn ManyBoxedExpressions<S> + Send>;
        fn dyn_insert_returning(&self) -> Box<dyn ManyBoxedExpressions<S> + Send>;
        fn dyn_insert_value(
            &self,
            from_data: Box<dyn Any + Send>,
            pre_op_output: Box<dyn Any + Send>,
        ) -> Box<dyn ManyBoxedExpressions<S> + Send>;
        fn dyn_from_row(&self) -> Box<dyn RawFromRow<S> + Send>;
        fn dyn_from_row_result(
            &self,
            from_data: Box<dyn Any + Send>,
            from_row: Box<dyn Any + Send>,
            pre_op_to_post_op: Box<dyn Any + Send>,
        ) -> (Box<dyn BoxedOperation<S> + Send>, Box<dyn Any + Send>);
        fn dyn_post_op_output(
            &self,
            poo: Box<dyn Any + Send>,
        ) -> Result<Box<dyn Any + Send>, ConstraintViolation>;
        fn dyn_take(
            self: Box<Self>,
            post_op_output: Box<dyn Any + Send>,
            insert_items: Box<dyn Any + Send>,
            pre_op_to_post_op: Box<dyn Any + Send>,
        ) -> Box<dyn Serialize<JsonAsString> + Send>;
    }

    impl<T, S> JsonInsertOneLink<S> for T
    where
        T: Send + Sync + 'static,
        S: sqlx::Database,
        T: InsertOneLink,
        T::PreOpData: 'static,
        T::PreOp: Operation<S> + 'static,
        T::PreOpToInsertValue: Send + 'static,
        T::PreOpToTake: Send + 'static,
        T::PreOpToPostOp: Send + 'static,
        T::InsertNames: Send + 'static + ManyBoxedExpressions<S>,
        T::InsertReturning: Send + 'static + ManyBoxedExpressions<S>,
        T::InsertValuesData: Send + 'static,
        T::InsertValues: Send + 'static + ManyBoxedExpressions<S>,
        T::FromRow: Send + 'static + RawFromRow<S>,
        T::PostOpData: Send + 'static,
        T::PostOp: Operation<S> + Send + 'static,
        T::TakeInput: Send + 'static,
        T::PostOpOutput: Send + 'static,
        T::Output: Send + 'static + Serialize<JsonAsString>,
    {
        fn dyn_pre_op_init(&self, input: Box<dyn Any + Send>) -> Box<dyn BoxedOperation<S> + Send> {
            // if std::any::TypeId::of::<T::PreOpData>()
            //     == std::any::TypeId::of::<Box<dyn Any + Send>>()
            // {
            //     let wrapper = (self as &dyn Any)
            //         .downcast_ref::<Box<dyn JsonInsertOneLink<S> + Send>>()
            //         .expect("PreOpData is Box<dyn Any>; self is Box<dyn JsonInsertOneLink>");
            //     return wrapper.as_ref().dyn_pre_op_init(input);
            // }
            let downcased = input.downcast::<T::PreOpData>().unwrap();
            Box::new(self.pre_operation_init(*downcased))
        }
        fn dyn_pre_op_split(
            &self,
            pre_op_output: Box<dyn Any + Send>,
        ) -> Result<
            (
                Box<dyn Any + Send>,
                Box<dyn Any + Send>,
                Box<dyn Any + Send>,
            ),
            ConstraintViolation,
        > {
            let downcasted_pre_op_output = pre_op_output
                .downcast::<<T::PreOp as OperationOutput>::Output>()
                .unwrap();

            let (insert_value, take, post_op) = self.pre_op_split(*downcasted_pre_op_output)?;
            Ok((Box::new(insert_value), Box::new(take), Box::new(post_op)))
        }
        fn dyn_insert_names(&self) -> Box<dyn ManyBoxedExpressions<S> + Send> {
            Box::new(self.insert_names())
        }
        fn dyn_insert_returning(&self) -> Box<dyn ManyBoxedExpressions<S> + Send> {
            Box::new(self.insert_returning())
        }
        fn dyn_insert_value(
            &self,
            from_data: Box<dyn Any + Send>,
            pre_op_output: Box<dyn Any + Send>,
        ) -> Box<dyn ManyBoxedExpressions<S> + Send> {
            let downcasted_from_data = from_data.downcast::<T::InsertValuesData>().unwrap();
            let downcasted_pre_op_output =
                pre_op_output.downcast::<T::PreOpToInsertValue>().unwrap();
            Box::new(self.insert_value(*downcasted_from_data, *downcasted_pre_op_output))
        }

        fn dyn_from_row(&self) -> Box<dyn RawFromRow<S> + Send> {
            Box::new(self.from_row())
        }
        fn dyn_from_row_result(
            &self,
            from_data: Box<dyn Any + Send>,
            from_row: Box<dyn Any + Send>,
            pre_op_to_post_op: Box<dyn Any + Send>,
        ) -> (Box<dyn BoxedOperation<S> + Send>, Box<dyn Any + Send>) {
            let downcasted_from_data = from_data.downcast::<T::PostOpData>().unwrap();
            let downcasted_from_row = from_row
                .downcast::<<T::FromRow as FromRowData>::RData>()
                .unwrap();
            let downcasted_pre_op_to_post_op =
                pre_op_to_post_op.downcast::<T::PreOpToPostOp>().unwrap();
            let (post_op, take_input) = self.from_row_result(
                *downcasted_from_data,
                *downcasted_from_row,
                *downcasted_pre_op_to_post_op,
            );

            (Box::new(post_op), Box::new(take_input))
        }
        fn dyn_post_op_output(
            &self,
            poo: Box<dyn Any + Send>,
        ) -> Result<Box<dyn Any + Send>, ConstraintViolation> {
            let downcasted_poo = poo
                .downcast::<<T::PostOp as OperationOutput>::Output>()
                .unwrap();
            Ok(Box::new(self.post_op_output(*downcasted_poo)?))
        }
        fn dyn_take(
            self: Box<Self>,
            post_op_output: Box<dyn Any + Send>,
            insert_items: Box<dyn Any + Send>,
            pre_op_to_post_op: Box<dyn Any + Send>,
        ) -> Box<dyn Serialize<JsonAsString> + Send> {
            let downcasted_post_op_output = post_op_output.downcast::<T::PostOpOutput>().unwrap();
            let downcasted_insert_items = insert_items.downcast::<T::TakeInput>().unwrap();
            let downcasted_pre_op_to_take = pre_op_to_post_op.downcast::<T::PreOpToTake>().unwrap();
            Box::new(self.take(
                *downcasted_post_op_output,
                *downcasted_insert_items,
                *downcasted_pre_op_to_take,
            ))
        }
    }

    impl<'b, S> InsertOneLink for Box<dyn JsonInsertOneLink<S> + Send + 'b>
    where
        S: Database,
    {
        type PreOp = Box<dyn BoxedOperation<S> + Send>;

        type PreOpData = Box<dyn Any + Send>;

        fn pre_operation_init(&self, input: Self::PreOpData) -> Self::PreOp {
            self.dyn_pre_op_init(input)
        }

        fn pre_op_split(
            &self,
            pre_op_output: <Self::PreOp as crate::operations::OperationOutput>::Output,
        ) -> Result<
            (
                Self::PreOpToInsertValue,
                Self::PreOpToTake,
                Self::PreOpToPostOp,
            ),
            crate::operations::insert_one::ConstraintViolation,
        > {
            self.dyn_pre_op_split(pre_op_output)
        }

        type PreOpToInsertValue = Box<dyn Any + Send>;

        type PreOpToTake = Box<dyn Any + Send>;

        type PreOpToPostOp = Box<dyn Any + Send>;

        type InsertNames = Box<dyn ManyBoxedExpressions<S> + Send>;

        fn insert_names(&self) -> Self::InsertNames {
            self.dyn_insert_names()
        }

        type InsertReturning = Box<dyn ManyBoxedExpressions<S> + Send>;

        fn insert_returning(&self) -> Self::InsertReturning {
            self.dyn_insert_returning()
        }

        type InsertValuesData = Box<dyn Any + Send>;

        type InsertValues = Box<dyn ManyBoxedExpressions<S> + Send>;

        fn insert_value(
            &self,
            from_data: Self::InsertValuesData,
            pre_op_output: Self::PreOpToInsertValue,
        ) -> Self::InsertValues {
            self.dyn_insert_value(from_data, pre_op_output)
        }

        type FromRow = Box<dyn RawFromRow<S> + Send>;

        fn from_row(&self) -> Self::FromRow {
            self.dyn_from_row()
        }

        type TakeInput = Box<dyn Any + Send>;

        type PostOp = Box<dyn BoxedOperation<S> + Send>;

        type PostOpData = Box<dyn Any + Send>;

        fn from_row_result(
            &self,
            from_data: Self::PostOpData,
            from_row: <Self::FromRow as crate::prelude::from_row_alias::FromRowData>::RData,
            pre_op_to_post_op: Self::PreOpToPostOp,
        ) -> (Self::PostOp, Self::TakeInput) {
            self.dyn_from_row_result(from_data, from_row, pre_op_to_post_op)
        }

        type PostOpOutput = Box<dyn Any + Send>;

        fn post_op_output(
            &self,
            poo: Box<dyn Any + Send>,
        ) -> Result<Self::PostOpOutput, crate::operations::insert_one::ConstraintViolation>
        {
            self.dyn_post_op_output(poo)
        }

        type Output = Box<dyn Serialize<JsonAsString> + Send>;

        fn take(
            self,
            post_op_output: Self::PostOpOutput,
            insert_items: Self::TakeInput,
            pre_op_to_post_op: Self::PreOpToTake,
        ) -> Self::Output {
            Box::new(self.dyn_take(post_op_output, insert_items, pre_op_to_post_op))
        }
    }

    impl<S> InsertLinkConsumeData for Vec<JsonInsertOneToConsume<S>>
    where
        S: Database + DatabaseExt + ExecutorTrait,
    {
        type Link = Vec<Box<dyn JsonInsertOneLink<S> + Send>>;

        fn consume_data(
            self,
        ) -> (
            Self::Link,
            InsertLinkData<
                <Self::Link as InsertOneLink>::PreOpData,
                <Self::Link as InsertOneLink>::InsertValuesData,
                <Self::Link as InsertOneLink>::PostOpData,
            >,
        ) {
            let mut links = Vec::with_capacity(self.len());
            let mut pre_op_data = Vec::with_capacity(self.len());
            let mut insert_value_data = Vec::with_capacity(self.len());
            let mut post_op_data = Vec::with_capacity(self.len());

            for item in self {
                links.push(item.link);
                pre_op_data.push(item.data.pre_op_data);
                insert_value_data.push(item.data.insert_value_data);
                post_op_data.push(item.data.post_op_data);
            }

            (
                links,
                InsertLinkData {
                    pre_op_data,
                    insert_value_data,
                    post_op_data,
                },
            )
        }
    }

    impl<'b, S> InsertOneLink for Vec<Box<dyn JsonInsertOneLink<S> + Send + 'b>>
    where
        S: Database + DatabaseExt + ExecutorTrait,
    {
        type PreOp = Vec<Box<dyn BoxedOperation<S> + Send>>;

        type PreOpData = Vec<Box<dyn Any + Send>>;

        fn pre_operation_init(&self, input: Self::PreOpData) -> Self::PreOp {
            self.iter()
                .zip(input)
                .map(|(link, data)| link.as_ref().dyn_pre_op_init(data))
                .collect()
        }

        fn pre_op_split(
            &self,
            pre_op_output: <Self::PreOp as OperationOutput>::Output,
        ) -> Result<
            (
                Self::PreOpToInsertValue,
                Self::PreOpToTake,
                Self::PreOpToPostOp,
            ),
            ConstraintViolation,
        > {
            let mut to_insert_value = Vec::with_capacity(self.len());
            let mut to_take = Vec::with_capacity(self.len());
            let mut to_post_op = Vec::with_capacity(self.len());

            for (link, output) in self.iter().zip(pre_op_output.into_iter()) {
                let (insert_value, take, post_op) = link.as_ref().dyn_pre_op_split(output)?;
                to_insert_value.push(insert_value);
                to_take.push(take);
                to_post_op.push(post_op);
            }

            Ok((to_insert_value, to_take, to_post_op))
        }

        type PreOpToInsertValue = Vec<Box<dyn Any + Send>>;

        type PreOpToTake = Vec<Box<dyn Any + Send>>;

        type PreOpToPostOp = Vec<Box<dyn Any + Send>>;

        type InsertNames = ManyFlat<Vec<Box<dyn ManyBoxedExpressions<S> + Send>>>;

        fn insert_names(&self) -> Self::InsertNames {
            ManyFlat(
                self.iter()
                    .map(|link| link.as_ref().dyn_insert_names())
                    .collect(),
            )
        }

        type InsertReturning = ManyFlat<Vec<Box<dyn ManyBoxedExpressions<S> + Send>>>;

        fn insert_returning(&self) -> Self::InsertReturning {
            ManyFlat(
                self.iter()
                    .map(|link| link.as_ref().dyn_insert_returning())
                    .collect(),
            )
        }

        type InsertValuesData = Vec<Box<dyn Any + Send>>;

        type InsertValues = ManyFlat<Vec<Box<dyn ManyBoxedExpressions<S> + Send>>>;

        fn insert_value(
            &self,
            from_data: Self::InsertValuesData,
            pre_op_output: Self::PreOpToInsertValue,
        ) -> Self::InsertValues {
            ManyFlat(
                self.iter()
                    .zip(from_data.into_iter().zip(pre_op_output))
                    .map(|(link, (from_data, pre_op))| {
                        link.as_ref().dyn_insert_value(from_data, pre_op)
                    })
                    .collect(),
            )
        }

        type FromRow = JsonInsertLinksFromRow<S>;

        fn from_row(&self) -> Self::FromRow {
            JsonInsertLinksFromRow(
                self.iter()
                    .map(|link| link.as_ref().dyn_from_row())
                    .collect(),
            )
        }

        type TakeInput = Vec<Box<dyn Any + Send>>;

        type PostOp = Vec<Box<dyn BoxedOperation<S> + Send>>;

        type PostOpData = Vec<Box<dyn Any + Send>>;

        fn from_row_result(
            &self,
            from_data: Self::PostOpData,
            from_row: <Self::FromRow as FromRowData>::RData,
            pre_op_to_post_op: Self::PreOpToPostOp,
        ) -> (Self::PostOp, Self::TakeInput) {
            let mut take_inputs = Vec::with_capacity(self.len());

            let post_ops = self
                .iter()
                .zip(from_data)
                .zip(from_row.into_iter().zip(pre_op_to_post_op))
                .map(|((link, from_data), (from_row, pre_op_to_post_op))| {
                    let (post_op, take_input) =
                        link.as_ref()
                            .dyn_from_row_result(from_data, from_row, pre_op_to_post_op);
                    take_inputs.push(take_input);
                    post_op
                })
                .collect();

            (post_ops, take_inputs)
        }

        type PostOpOutput = Vec<Box<dyn Any + Send>>;

        fn post_op_output(
            &self,
            poo: <Self::PostOp as OperationOutput>::Output,
        ) -> Result<Self::PostOpOutput, ConstraintViolation> {
            self.iter()
                .zip(poo.into_iter())
                .map(|(link, output)| link.as_ref().dyn_post_op_output(output))
                .collect()
        }

        type Output = Vec<Box<dyn Serialize<JsonAsString> + Send>>;

        fn take(
            self,
            post_op_output: Self::PostOpOutput,
            insert_items: Self::TakeInput,
            pre_op_to_take: Self::PreOpToTake,
        ) -> Self::Output {
            self.into_iter()
                .zip(
                    post_op_output
                        .into_iter()
                        .zip(insert_items.into_iter().zip(pre_op_to_take)),
                )
                .map(|(link, (post_op_output, (insert_items, pre_op_to_take)))| {
                    link.dyn_take(post_op_output, insert_items, pre_op_to_take)
                })
                .collect()
        }
    }
}

pub mod update_one_trait_extension {
    use std::any::Any;

    use sqlx::Database;

    use crate::{
        database_extention::DatabaseExt,
        extentions::common_expressions::raw_from_row::RawFromRow,
        fix_executor::ExecutorTrait,
        from_row::FromRowData,
        gen_serde::{Serialize, json_serialize_side::JsonAsString},
        operations::{
            Operation, OperationOutput,
            boxed_operation::BoxedOperation,
            insert_one::ConstraintViolation,
            update::{UpdateLink, UpdateLinkData, UpdateLinkSplit},
        },
        query_builder::{ManyBoxedExpressions, functional_expr::ManyFlat},
    };

    pub struct JsonUpdateLinksFromRow<S>(pub Vec<Box<dyn RawFromRow<S> + Send>>);

    impl<S: Database> FromRowData for JsonUpdateLinksFromRow<S> {
        type RData = Vec<Box<dyn Any + Send>>;
    }

    impl<'r, S: Database> crate::from_row::FromRowAlias<'r, S::Row> for JsonUpdateLinksFromRow<S> {
        fn no_alias(&self, row: &'r S::Row) -> Result<Self::RData, crate::from_row::FromRowError> {
            self.0
                .iter()
                .map(|from_row| RawFromRow::dyn_no_alias(&**from_row, row))
                .collect()
        }

        fn pre_alias(
            &self,
            row: crate::from_row::RowPreAliased<'r, S::Row>,
        ) -> Result<Self::RData, crate::from_row::FromRowError>
        where
            S::Row: sqlx::Row,
        {
            self.0
                .iter()
                .map(|from_row| RawFromRow::dyn_pre_alias(&**from_row, row.clone()))
                .collect()
        }

        fn post_alias(
            &self,
            _: crate::from_row::RowPostAliased<'r, S::Row>,
        ) -> Result<Self::RData, crate::from_row::FromRowError>
        where
            S::Row: sqlx::Row,
        {
            panic!("to be deprecated")
        }

        fn two_alias(
            &self,
            row: crate::from_row::RowTwoAliased<'r, S::Row>,
        ) -> Result<Self::RData, crate::from_row::FromRowError>
        where
            S::Row: sqlx::Row,
        {
            self.0
                .iter()
                .map(|from_row| RawFromRow::dyn_two_alias(&**from_row, row.clone()))
                .collect()
        }
    }

    pub struct JsonUpdateOneToConsume<S> {
        pub(crate) link: Box<dyn JsonUpdateOneLink<S> + Send>,
        pub(crate) data: UpdateLinkData<
            Box<dyn Any + Send>,
            Box<dyn Any + Send>,
            Box<dyn Any + Send>,
            Box<dyn Any + Send>,
        >,
    }

    impl<S> JsonUpdateOneToConsume<S> {
        pub fn new<T>(split: T) -> Self
        where
            S: sqlx::Database,
            T: UpdateLinkSplit,
            T::Link: JsonUpdateOneLink<S> + Send + 'static,
            <T::Link as UpdateLink>::InitSplitForWheres: Send + 'static,
            <T::Link as UpdateLink>::InitSplitForUpdateValues: Send + 'static,
            <T::Link as UpdateLink>::InitSplitForPreOp: Send + 'static,
            <T::Link as UpdateLink>::InitSplitPostOp: Send + 'static,
        {
            let (link, data) = split.init_split();
            Self {
                link: Box::new(link),
                data: UpdateLinkData {
                    wheres: Box::new(data.wheres),
                    update_values: Box::new(data.update_values),
                    pre_op: Box::new(data.pre_op),
                    post_op: Box::new(data.post_op),
                },
            }
        }
    }

    pub trait JsonUpdateOneLink<S: Database>: Send + Sync + 'static {
        fn dyn_pre_op(&self, input: Box<dyn Any + Send>) -> Box<dyn BoxedOperation<S> + Send>;
        fn dyn_split_pre_op(
            &self,
            pre_op_output: Box<dyn Any + Send>,
        ) -> Result<
            (
                Box<dyn ManyBoxedExpressions<S> + Send>,
                Box<dyn Any + Send>,
                Box<dyn Any + Send>,
                Box<dyn Any + Send>,
            ),
            ConstraintViolation,
        >;
        fn dyn_wheres(
            &self,
            wheres: Box<dyn Any + Send>,
        ) -> Box<dyn ManyBoxedExpressions<S> + Send>;
        fn dyn_update_names(&self) -> Box<dyn ManyBoxedExpressions<S> + Send>;
        fn dyn_update_values(
            &self,
            values: Box<dyn Any + Send>,
            pre_op_output: Box<dyn Any + Send>,
        ) -> Box<dyn ManyBoxedExpressions<S> + Send>;
        fn dyn_from_row(&self) -> Box<dyn RawFromRow<S> + Send>;
        fn dyn_post_op(
            &self,
            init_split: Box<dyn Any + Send>,
            pre_op_split: Box<dyn Any + Send>,
        ) -> Box<dyn BoxedOperation<S> + Send>;
        fn dyn_from_row_result(
            &self,
            row_data: &Box<dyn Any + Send>,
            post_op: &mut Box<dyn BoxedOperation<S> + Send>,
        );
        fn dyn_post_op_output(
            &self,
            poo: Box<dyn Any + Send>,
        ) -> Result<Box<dyn Any + Send>, ConstraintViolation>;
        fn dyn_take(
            &self,
            from_row: Box<dyn Any + Send>,
            post_op: &mut Box<dyn Any + Send>,
            pre_op_split_take: &mut Box<dyn Any + Send>,
        ) -> Box<dyn Serialize<JsonAsString> + Send>;
    }

    impl<T, S> JsonUpdateOneLink<S> for T
    where
        T: Send + Sync + 'static,
        S: sqlx::Database,
        T: UpdateLink,
        T::InitSplitForPreOp: 'static + Send,
        T::PreOp: Operation<S> + 'static,
        T::PreOpSplitWheres: Send + 'static + ManyBoxedExpressions<S>,
        T::PreOpSplitValues: Send + 'static,
        T::PreOpSplitPostOp: Send + 'static,
        T::PreOpSplitTake: Send + 'static,
        T::InitSplitForWheres: Send + 'static,
        T::UpdateWhere: Send + 'static + ManyBoxedExpressions<S>,
        T::UpdateNames: Send + 'static + ManyBoxedExpressions<S>,
        T::InitSplitForUpdateValues: Send + 'static,
        T::UpdateValues: Send + 'static + ManyBoxedExpressions<S>,
        T::FromRow: Send + 'static + RawFromRow<S>,
        T::InitSplitPostOp: Send + 'static,
        T::PostOp: Operation<S> + Send + 'static,
        T::PostOpOutput: Send + 'static,
        T::Output: Send + 'static + Serialize<JsonAsString>,
    {
        fn dyn_pre_op(&self, input: Box<dyn Any + Send>) -> Box<dyn BoxedOperation<S> + Send> {
            let downcasted = input.downcast::<T::InitSplitForPreOp>().unwrap();
            Box::new(self.pre_op(*downcasted))
        }

        fn dyn_split_pre_op(
            &self,
            pre_op_output: Box<dyn Any + Send>,
        ) -> Result<
            (
                Box<dyn ManyBoxedExpressions<S> + Send>,
                Box<dyn Any + Send>,
                Box<dyn Any + Send>,
                Box<dyn Any + Send>,
            ),
            ConstraintViolation,
        > {
            let downcasted_pre_op_output = pre_op_output
                .downcast::<<T::PreOp as OperationOutput>::Output>()
                .unwrap();
            let (wheres, values, post_op, take) = self.split_pre_op(*downcasted_pre_op_output)?;
            Ok((
                Box::new(wheres),
                Box::new(values),
                Box::new(post_op),
                Box::new(take),
            ))
        }

        fn dyn_wheres(
            &self,
            wheres: Box<dyn Any + Send>,
        ) -> Box<dyn ManyBoxedExpressions<S> + Send> {
            let downcasted = wheres.downcast::<T::InitSplitForWheres>().unwrap();
            Box::new(self.wheres(*downcasted))
        }

        fn dyn_update_names(&self) -> Box<dyn ManyBoxedExpressions<S> + Send> {
            Box::new(self.update_names())
        }

        fn dyn_update_values(
            &self,
            values: Box<dyn Any + Send>,
            pre_op_output: Box<dyn Any + Send>,
        ) -> Box<dyn ManyBoxedExpressions<S> + Send> {
            let downcasted_values = values.downcast::<T::InitSplitForUpdateValues>().unwrap();
            let downcasted_pre_op_output = pre_op_output.downcast::<T::PreOpSplitValues>().unwrap();
            Box::new(self.update_values(*downcasted_values, *downcasted_pre_op_output))
        }

        fn dyn_from_row(&self) -> Box<dyn RawFromRow<S> + Send> {
            Box::new(self.from_row())
        }

        fn dyn_post_op(
            &self,
            init_split: Box<dyn Any + Send>,
            pre_op_split: Box<dyn Any + Send>,
        ) -> Box<dyn BoxedOperation<S> + Send> {
            let downcasted_init = init_split.downcast::<T::InitSplitPostOp>().unwrap();
            let downcasted_pre = pre_op_split.downcast::<T::PreOpSplitPostOp>().unwrap();
            Box::new(self.post_op(*downcasted_init, *downcasted_pre))
        }

        fn dyn_from_row_result(
            &self,
            row_data: &Box<dyn Any + Send>,
            post_op: &mut Box<dyn BoxedOperation<S> + Send>,
        ) {
            let downcasted_row = row_data
                .downcast_ref::<<T::FromRow as FromRowData>::RData>()
                .unwrap();
            let downcasted_post_op = (post_op.as_mut() as &mut dyn Any)
                .downcast_mut::<T::PostOp>()
                .unwrap();
            self.from_row_result(downcasted_row, downcasted_post_op);
        }

        fn dyn_post_op_output(
            &self,
            poo: Box<dyn Any + Send>,
        ) -> Result<Box<dyn Any + Send>, ConstraintViolation> {
            let downcasted_poo = poo
                .downcast::<<T::PostOp as OperationOutput>::Output>()
                .unwrap();
            Ok(Box::new(self.post_op_output(*downcasted_poo)?))
        }

        fn dyn_take(
            &self,
            from_row: Box<dyn Any + Send>,
            post_op: &mut Box<dyn Any + Send>,
            pre_op_split_take: &mut Box<dyn Any + Send>,
        ) -> Box<dyn Serialize<JsonAsString> + Send> {
            let downcasted_from_row = from_row
                .downcast::<<T::FromRow as FromRowData>::RData>()
                .unwrap();
            let downcasted_post_op = post_op.downcast_mut::<T::PostOpOutput>().unwrap();
            let downcasted_take = pre_op_split_take
                .downcast_mut::<T::PreOpSplitTake>()
                .unwrap();
            Box::new(self.take(*downcasted_from_row, downcasted_post_op, downcasted_take))
        }
    }

    impl<'b, S> UpdateLink for Box<dyn JsonUpdateOneLink<S> + Send + 'b>
    where
        S: Database,
    {
        type InitSplitForPreOp = Box<dyn Any + Send>;

        type PreOp = Box<dyn BoxedOperation<S> + Send>;

        fn pre_op(&self, init_split_for_pre_op: Self::InitSplitForPreOp) -> Self::PreOp {
            self.dyn_pre_op(init_split_for_pre_op)
        }

        fn split_pre_op(
            &self,
            pre_op_output: <Self::PreOp as OperationOutput>::Output,
        ) -> Result<
            (
                Self::PreOpSplitWheres,
                Self::PreOpSplitValues,
                Self::PreOpSplitPostOp,
                Self::PreOpSplitTake,
            ),
            ConstraintViolation,
        > {
            self.dyn_split_pre_op(pre_op_output)
        }

        type PreOpSplitWheres = Box<dyn ManyBoxedExpressions<S> + Send>;

        type PreOpSplitValues = Box<dyn Any + Send>;

        type PreOpSplitPostOp = Box<dyn Any + Send>;

        type PreOpSplitTake = Box<dyn Any + Send>;

        type InitSplitForWheres = Box<dyn Any + Send>;

        type UpdateWhere = Box<dyn ManyBoxedExpressions<S> + Send>;

        fn wheres(&self, wheres: Self::InitSplitForWheres) -> Self::UpdateWhere {
            self.dyn_wheres(wheres)
        }

        type UpdateNames = Box<dyn ManyBoxedExpressions<S> + Send>;

        fn update_names(&self) -> Self::UpdateNames {
            self.dyn_update_names()
        }

        type InitSplitForUpdateValues = Box<dyn Any + Send>;

        type UpdateValues = Box<dyn ManyBoxedExpressions<S> + Send>;

        fn update_values(
            &self,
            values: Self::InitSplitForUpdateValues,
            pre_op_output: Self::PreOpSplitValues,
        ) -> Self::UpdateValues {
            self.dyn_update_values(values, pre_op_output)
        }

        type FromRow = Box<dyn RawFromRow<S> + Send>;

        fn from_row(&self) -> Self::FromRow {
            self.dyn_from_row()
        }

        type InitSplitPostOp = Box<dyn Any + Send>;

        type PostOp = Box<dyn BoxedOperation<S> + Send>;

        fn post_op(
            &self,
            from_init_split: Self::InitSplitPostOp,
            from_pre_op: Self::PreOpSplitPostOp,
        ) -> Self::PostOp {
            self.dyn_post_op(from_init_split, from_pre_op)
        }

        fn from_row_result(
            &self,
            row_data: &<Self::FromRow as FromRowData>::RData,
            post_op: &mut Self::PostOp,
        ) {
            self.dyn_from_row_result(row_data, post_op)
        }

        type PostOpOutput = Box<dyn Any + Send>;

        fn post_op_output(
            &self,
            poo: <Self::PostOp as OperationOutput>::Output,
        ) -> Result<Self::PostOpOutput, ConstraintViolation> {
            self.dyn_post_op_output(poo)
        }

        type Output = Box<dyn Serialize<JsonAsString> + Send>;

        fn take(
            &self,
            from_row: <Self::FromRow as FromRowData>::RData,
            post_op: &mut Self::PostOpOutput,
            pre_op_split_take: &mut Self::PreOpSplitTake,
        ) -> Self::Output {
            self.dyn_take(from_row, post_op, pre_op_split_take)
        }
    }

    impl<S> UpdateLinkSplit for Vec<JsonUpdateOneToConsume<S>>
    where
        S: Database + DatabaseExt + ExecutorTrait,
    {
        type Link = Vec<Box<dyn JsonUpdateOneLink<S> + Send>>;

        fn init_split(
            self,
        ) -> (
            Self::Link,
            UpdateLinkData<
                <Self::Link as UpdateLink>::InitSplitForWheres,
                <Self::Link as UpdateLink>::InitSplitForUpdateValues,
                <Self::Link as UpdateLink>::InitSplitForPreOp,
                <Self::Link as UpdateLink>::InitSplitPostOp,
            >,
        ) {
            let mut links = Vec::with_capacity(self.len());
            let mut wheres = Vec::with_capacity(self.len());
            let mut update_values = Vec::with_capacity(self.len());
            let mut pre_op = Vec::with_capacity(self.len());
            let mut post_op = Vec::with_capacity(self.len());

            for item in self {
                links.push(item.link);
                wheres.push(item.data.wheres);
                update_values.push(item.data.update_values);
                pre_op.push(item.data.pre_op);
                post_op.push(item.data.post_op);
            }

            (
                links,
                UpdateLinkData {
                    wheres,
                    update_values,
                    pre_op,
                    post_op,
                },
            )
        }
    }

    impl<'b, S> UpdateLink for Vec<Box<dyn JsonUpdateOneLink<S> + Send + 'b>>
    where
        S: Database + DatabaseExt + ExecutorTrait,
    {
        type InitSplitForPreOp = Vec<Box<dyn Any + Send>>;

        type PreOp = Vec<Box<dyn BoxedOperation<S> + Send>>;

        fn pre_op(&self, init_split_for_pre_op: Self::InitSplitForPreOp) -> Self::PreOp {
            self.iter()
                .zip(init_split_for_pre_op)
                .map(|(link, data)| link.as_ref().dyn_pre_op(data))
                .collect()
        }

        fn split_pre_op(
            &self,
            pre_op_output: <Self::PreOp as OperationOutput>::Output,
        ) -> Result<
            (
                Self::PreOpSplitWheres,
                Self::PreOpSplitValues,
                Self::PreOpSplitPostOp,
                Self::PreOpSplitTake,
            ),
            ConstraintViolation,
        > {
            let mut wheres = Vec::with_capacity(self.len());
            let mut values = Vec::with_capacity(self.len());
            let mut post_op = Vec::with_capacity(self.len());
            let mut take = Vec::with_capacity(self.len());

            for (link, output) in self.iter().zip(pre_op_output.into_iter()) {
                let (w, v, p, t) = link.as_ref().dyn_split_pre_op(output)?;
                wheres.push(w);
                values.push(v);
                post_op.push(p);
                take.push(t);
            }

            Ok((ManyFlat(wheres), values, post_op, take))
        }

        type PreOpSplitWheres = ManyFlat<Vec<Box<dyn ManyBoxedExpressions<S> + Send>>>;

        type PreOpSplitValues = Vec<Box<dyn Any + Send>>;

        type PreOpSplitPostOp = Vec<Box<dyn Any + Send>>;

        type PreOpSplitTake = Vec<Box<dyn Any + Send>>;

        type InitSplitForWheres = Vec<Box<dyn Any + Send>>;

        type UpdateWhere = ManyFlat<Vec<Box<dyn ManyBoxedExpressions<S> + Send>>>;

        fn wheres(&self, wheres: Self::InitSplitForWheres) -> Self::UpdateWhere {
            ManyFlat(
                self.iter()
                    .zip(wheres)
                    .map(|(link, data)| link.as_ref().dyn_wheres(data))
                    .collect(),
            )
        }

        type UpdateNames = ManyFlat<Vec<Box<dyn ManyBoxedExpressions<S> + Send>>>;

        fn update_names(&self) -> Self::UpdateNames {
            ManyFlat(
                self.iter()
                    .map(|link| link.as_ref().dyn_update_names())
                    .collect(),
            )
        }

        type InitSplitForUpdateValues = Vec<Box<dyn Any + Send>>;

        type UpdateValues = ManyFlat<Vec<Box<dyn ManyBoxedExpressions<S> + Send>>>;

        fn update_values(
            &self,
            values: Self::InitSplitForUpdateValues,
            pre_op_output: Self::PreOpSplitValues,
        ) -> Self::UpdateValues {
            ManyFlat(
                self.iter()
                    .zip(values.into_iter().zip(pre_op_output))
                    .map(|(link, (values, pre_op))| link.as_ref().dyn_update_values(values, pre_op))
                    .collect(),
            )
        }

        type FromRow = JsonUpdateLinksFromRow<S>;

        fn from_row(&self) -> Self::FromRow {
            JsonUpdateLinksFromRow(
                self.iter()
                    .map(|link| link.as_ref().dyn_from_row())
                    .collect(),
            )
        }

        type InitSplitPostOp = Vec<Box<dyn Any + Send>>;

        type PostOp = Vec<Box<dyn BoxedOperation<S> + Send>>;

        fn post_op(
            &self,
            from_init_split: Self::InitSplitPostOp,
            from_pre_op: Self::PreOpSplitPostOp,
        ) -> Self::PostOp {
            self.iter()
                .zip(from_init_split.into_iter().zip(from_pre_op))
                .map(|(link, (init, pre))| link.as_ref().dyn_post_op(init, pre))
                .collect()
        }

        fn from_row_result(
            &self,
            row_data: &<Self::FromRow as FromRowData>::RData,
            post_op: &mut Self::PostOp,
        ) {
            for (link, (row, post)) in self.iter().zip(row_data.iter().zip(post_op.iter_mut())) {
                link.as_ref().dyn_from_row_result(row, post);
            }
        }

        type PostOpOutput = Vec<Box<dyn Any + Send>>;

        fn post_op_output(
            &self,
            poo: <Self::PostOp as OperationOutput>::Output,
        ) -> Result<Self::PostOpOutput, ConstraintViolation> {
            self.iter()
                .zip(poo.into_iter())
                .map(|(link, output)| link.as_ref().dyn_post_op_output(output))
                .collect()
        }

        type Output = Vec<Box<dyn Serialize<JsonAsString> + Send>>;

        fn take(
            &self,
            from_row: <Self::FromRow as FromRowData>::RData,
            post_op: &mut Self::PostOpOutput,
            pre_op_split_take: &mut Self::PreOpSplitTake,
        ) -> Self::Output {
            self.iter()
                .zip(from_row)
                .zip(post_op.drain(..))
                .zip(pre_op_split_take.drain(..))
                .map(|(((link, from_row), mut post_op), mut take)| {
                    link.as_ref().dyn_take(from_row, &mut post_op, &mut take)
                })
                .collect()
        }
    }
}

pub mod delete_one_trait_extension {
    use std::any::Any;

    use sqlx::Database;

    type JsonDeleteOneInitSplitForPreOp = Vec<Box<dyn Any + Send>>;
    type JsonDeleteOneInitSplitForWheres = Vec<Box<dyn Any + Send>>;

    use crate::{
        database_extention::DatabaseExt,
        extentions::common_expressions::raw_from_row::RawFromRow,
        fix_executor::ExecutorTrait,
        from_row::FromRowData,
        gen_serde::{Serialize, json_serialize_side::JsonAsString},
        operations::{
            Operation, OperationOutput,
            boxed_operation::BoxedOperation,
            delete::{DeleteLink, DeleteLinkData, DeleteLinkPreOp, DeleteLinkSplit},
        },
        query_builder::{ManyBoxedExpressions, functional_expr::ManyFlat},
    };

    pub struct JsonDeleteLinksFromRow<S>(pub Vec<Box<dyn RawFromRow<S> + Send>>);

    unsafe impl<S: Database> Sync for JsonDeleteLinksFromRow<S> {}

    impl<S: Database> FromRowData for JsonDeleteLinksFromRow<S> {
        type RData = Vec<Box<dyn Any + Send>>;
    }

    impl<'r, S: Database> crate::from_row::FromRowAlias<'r, S::Row> for JsonDeleteLinksFromRow<S> {
        fn no_alias(&self, row: &'r S::Row) -> Result<Self::RData, crate::from_row::FromRowError> {
            self.0
                .iter()
                .map(|from_row| RawFromRow::dyn_no_alias(&**from_row, row))
                .collect()
        }

        fn pre_alias(
            &self,
            row: crate::from_row::RowPreAliased<'r, S::Row>,
        ) -> Result<Self::RData, crate::from_row::FromRowError>
        where
            S::Row: sqlx::Row,
        {
            self.0
                .iter()
                .map(|from_row| RawFromRow::dyn_pre_alias(&**from_row, row.clone()))
                .collect()
        }

        fn post_alias(
            &self,
            _: crate::from_row::RowPostAliased<'r, S::Row>,
        ) -> Result<Self::RData, crate::from_row::FromRowError>
        where
            S::Row: sqlx::Row,
        {
            panic!("to be deprecated")
        }

        fn two_alias(
            &self,
            row: crate::from_row::RowTwoAliased<'r, S::Row>,
        ) -> Result<Self::RData, crate::from_row::FromRowError>
        where
            S::Row: sqlx::Row,
        {
            self.0
                .iter()
                .map(|from_row| RawFromRow::dyn_two_alias(&**from_row, row.clone()))
                .collect()
        }
    }

    pub struct JsonDeleteOneToConsume<S> {
        pub(crate) link: Box<dyn JsonDeleteOneLink<S> + Send>,
        pub(crate) init_split_for_pre_op: Box<dyn Any + Send>,
        pub(crate) data: DeleteLinkData<Box<dyn Any + Send>>,
    }

    impl<S> JsonDeleteOneToConsume<S> {
        pub fn from_split<T>(split: T) -> Self
        where
            S: sqlx::Database,
            T: DeleteLinkSplit,
            T::Link: JsonDeleteOneLink<S> + Send + 'static,
            T::InitSplitForPreOp: Send + 'static,
            <T::Link as DeleteLink>::InitSplitForWheres: Send + 'static,
        {
            let (link, init_split_for_pre_op, data) = split.init_split();
            Self {
                link: Box::new(link),
                init_split_for_pre_op: Box::new(init_split_for_pre_op),
                data: DeleteLinkData {
                    wheres: Box::new(data.wheres),
                },
            }
        }
    }

    pub trait JsonDeleteOneLink<S: Database>: Send + Sync + 'static {
        fn dyn_pre_op(
            &self,
            init: Box<dyn Any + Send>,
            wheres: &dyn Any,
        ) -> Box<dyn BoxedOperation<S> + Send>;
        fn dyn_split_pre_op(
            &self,
            pre_op_output: Box<dyn Any + Send>,
        ) -> (Box<dyn Any + Send>, Box<dyn Any + Send>);
        fn dyn_wheres(
            &self,
            init_split_for_wheres: Box<dyn Any + Send>,
            pre_op_split_wheres: Box<dyn Any + Send>,
        ) -> Box<dyn ManyBoxedExpressions<S> + Send>;
        fn dyn_delete_return_expression(&self) -> Box<dyn ManyBoxedExpressions<S> + Send>;
        fn dyn_from_row(&self) -> Box<dyn RawFromRow<S> + Send>;
        fn dyn_take_once(
            &self,
            links: Box<dyn Any + Send>,
            pre_op_split_take: Box<dyn Any + Send>,
        ) -> Box<dyn Serialize<JsonAsString> + Send>;
        fn dyn_take_mut(
            &self,
            links: Box<dyn Any + Send>,
            pre_op_split_take: &mut Box<dyn Any + Send>,
        ) -> Box<dyn Serialize<JsonAsString> + Send>;
    }

    impl<T, S> JsonDeleteOneLink<S> for T
    where
        T: Send + Sync + 'static,
        S: Database,
        T: DeleteLink,
        T: DeleteLinkPreOp<()>,
        <T as DeleteLinkPreOp<()>>::InitSplitForPreOp: Send + 'static,
        <T as DeleteLinkPreOp<()>>::PreOp: Operation<S> + 'static,
        T::PreOpOutput: Send + 'static,
        T::PreOpSplitWheres: Send + 'static,
        T::PreOpSplitTake: Send + 'static,
        T::InitSplitForWheres: Send + 'static,
        T::Wheres: Send + ManyBoxedExpressions<S>,
        T::DeleteReturnExpression: Send + ManyBoxedExpressions<S>,
        T::DeleteReturnFromRow: Send + Sync + RawFromRow<S>,
        T::Output: Send + Serialize<JsonAsString>,
    {
        fn dyn_pre_op(
            &self,
            init: Box<dyn Any + Send>,
            _wheres: &dyn Any,
        ) -> Box<dyn BoxedOperation<S> + Send> {
            let downcasted_init = init
                .downcast::<<T as DeleteLinkPreOp<()>>::InitSplitForPreOp>()
                .unwrap();
            Box::new(self.pre_op(*downcasted_init, &()))
        }

        fn dyn_split_pre_op(
            &self,
            pre_op_output: Box<dyn Any + Send>,
        ) -> (Box<dyn Any + Send>, Box<dyn Any + Send>) {
            let downcasted = pre_op_output.downcast::<T::PreOpOutput>().unwrap();
            let (wheres, take) = self.split_pre_op(*downcasted);
            (Box::new(wheres), Box::new(take))
        }

        fn dyn_wheres(
            &self,
            init_split_for_wheres: Box<dyn Any + Send>,
            pre_op_split_wheres: Box<dyn Any + Send>,
        ) -> Box<dyn ManyBoxedExpressions<S> + Send> {
            let init = init_split_for_wheres
                .downcast::<T::InitSplitForWheres>()
                .unwrap();
            let pre = pre_op_split_wheres
                .downcast::<T::PreOpSplitWheres>()
                .unwrap();
            Box::new(self.wheres(*init, *pre))
        }

        fn dyn_delete_return_expression(&self) -> Box<dyn ManyBoxedExpressions<S> + Send> {
            Box::new(self.delete_return_expression())
        }

        fn dyn_from_row(&self) -> Box<dyn RawFromRow<S> + Send> {
            Box::new(self.from_row())
        }

        fn dyn_take_once(
            &self,
            links: Box<dyn Any + Send>,
            pre_op_split_take: Box<dyn Any + Send>,
        ) -> Box<dyn Serialize<JsonAsString> + Send> {
            let downcasted_links = links
                .downcast::<<T::DeleteReturnFromRow as FromRowData>::RData>()
                .unwrap();
            let downcasted_take = pre_op_split_take.downcast::<T::PreOpSplitTake>().unwrap();
            Box::new(self.take_once(*downcasted_links, *downcasted_take))
        }

        fn dyn_take_mut(
            &self,
            links: Box<dyn Any + Send>,
            pre_op_split_take: &mut Box<dyn Any + Send>,
        ) -> Box<dyn Serialize<JsonAsString> + Send> {
            let downcasted_links = links
                .downcast::<<T::DeleteReturnFromRow as FromRowData>::RData>()
                .unwrap();
            let downcasted_take = pre_op_split_take
                .downcast_mut::<T::PreOpSplitTake>()
                .unwrap();
            Box::new(self.take_mut(*downcasted_links, downcasted_take))
        }
    }

    impl<W, S> DeleteLinkPreOp<W> for Box<dyn JsonDeleteOneLink<S> + Send>
    where
        W: Clone + Send + 'static,
        S: Database,
    {
        type InitSplitForPreOp = Box<dyn Any + Send>;

        type PreOp = Box<dyn BoxedOperation<S> + Send>;

        fn pre_op(&self, init: Self::InitSplitForPreOp, wheres: &W) -> Self::PreOp {
            self.dyn_pre_op(init, wheres as &dyn Any)
        }
    }

    impl<S> DeleteLink for Box<dyn JsonDeleteOneLink<S> + Send>
    where
        S: Database,
    {
        type Output = Box<dyn Serialize<JsonAsString> + Send>;

        type PreOpOutput = Box<dyn Any + Send>;

        type PreOpSplitWheres = Box<dyn Any + Send>;

        type PreOpSplitTake = Box<dyn Any + Send>;

        fn split_pre_op(
            &self,
            pre_op_output: Self::PreOpOutput,
        ) -> (Self::PreOpSplitWheres, Self::PreOpSplitTake) {
            self.dyn_split_pre_op(pre_op_output)
        }

        type InitSplitForWheres = Box<dyn Any + Send>;

        type Wheres = Box<dyn ManyBoxedExpressions<S> + Send>;

        fn wheres(
            &self,
            init_split_for_wheres: Self::InitSplitForWheres,
            pre_op_split_wheres: Self::PreOpSplitWheres,
        ) -> Self::Wheres {
            self.dyn_wheres(init_split_for_wheres, pre_op_split_wheres)
        }

        type DeleteReturnExpression = Box<dyn ManyBoxedExpressions<S> + Send>;

        fn delete_return_expression(&self) -> Self::DeleteReturnExpression {
            self.dyn_delete_return_expression()
        }

        type DeleteReturnFromRow = Box<dyn RawFromRow<S> + Send>;

        fn from_row(&self) -> Self::DeleteReturnFromRow {
            self.dyn_from_row()
        }

        fn take_mut(
            &self,
            links: <Self::DeleteReturnFromRow as FromRowData>::RData,
            pre_op_split_take: &mut Self::PreOpSplitTake,
        ) -> Self::Output {
            self.dyn_take_mut(links, pre_op_split_take)
        }

        fn take_once(
            &self,
            links: <Self::DeleteReturnFromRow as FromRowData>::RData,
            pre_op_split_take: Self::PreOpSplitTake,
        ) -> Self::Output {
            self.dyn_take_once(links, pre_op_split_take)
        }
    }

    impl<S> DeleteLinkSplit for Vec<JsonDeleteOneToConsume<S>>
    where
        S: Database + DatabaseExt + ExecutorTrait,
    {
        type Link = Vec<Box<dyn JsonDeleteOneLink<S> + Send>>;

        type InitSplitForPreOp = JsonDeleteOneInitSplitForPreOp;

        fn init_split(
            self,
        ) -> (
            Self::Link,
            Self::InitSplitForPreOp,
            DeleteLinkData<<Self::Link as DeleteLink>::InitSplitForWheres>,
        ) {
            let mut links = Vec::with_capacity(self.len());
            let mut init_split_for_pre_op = Vec::with_capacity(self.len());
            let mut wheres = Vec::with_capacity(self.len());

            for item in self {
                links.push(item.link);
                init_split_for_pre_op.push(item.init_split_for_pre_op);
                wheres.push(item.data.wheres);
            }

            (links, init_split_for_pre_op, DeleteLinkData { wheres })
        }
    }

    impl<'b, W, S> DeleteLinkPreOp<W> for Vec<Box<dyn JsonDeleteOneLink<S> + Send + 'b>>
    where
        W: Clone + Send + 'static,
        S: Database + DatabaseExt + ExecutorTrait,
        Vec<JsonDeleteOneToConsume<S>>:
            DeleteLinkSplit<InitSplitForPreOp: IntoIterator<Item = Box<dyn Any + Send>> + Send>,
    {
        type InitSplitForPreOp =
            <Vec<JsonDeleteOneToConsume<S>> as DeleteLinkSplit>::InitSplitForPreOp;

        type PreOp = Vec<Box<dyn BoxedOperation<S> + Send>>;

        fn pre_op(&self, init: Self::InitSplitForPreOp, wheres: &W) -> Self::PreOp {
            self.iter()
                .zip(init)
                .map(|(link, data)| link.as_ref().dyn_pre_op(data, wheres as &dyn Any))
                .collect()
        }
    }

    impl<'b, S> DeleteLink for Vec<Box<dyn JsonDeleteOneLink<S> + Send + 'b>>
    where
        S: Database + DatabaseExt + ExecutorTrait,
    {
        type Output = Vec<Box<dyn Serialize<JsonAsString> + Send>>;

        type PreOpOutput = Vec<Box<dyn Any + Send>>;

        type PreOpSplitWheres = Vec<Box<dyn Any + Send>>;

        type PreOpSplitTake = Vec<Box<dyn Any + Send>>;

        fn split_pre_op(
            &self,
            pre_op_output: Self::PreOpOutput,
        ) -> (Self::PreOpSplitWheres, Self::PreOpSplitTake) {
            let mut wheres = Vec::with_capacity(self.len());
            let mut take = Vec::with_capacity(self.len());

            for (link, output) in self.iter().zip(pre_op_output.into_iter()) {
                let (w, t) = link.as_ref().dyn_split_pre_op(output);
                wheres.push(w);
                take.push(t);
            }

            (wheres, take)
        }

        type InitSplitForWheres = JsonDeleteOneInitSplitForWheres;

        type Wheres = ManyFlat<Vec<Box<dyn ManyBoxedExpressions<S> + Send>>>;

        fn wheres(
            &self,
            init_split_for_wheres: Self::InitSplitForWheres,
            pre_op_split_wheres: Self::PreOpSplitWheres,
        ) -> Self::Wheres {
            ManyFlat(
                self.iter()
                    .zip(init_split_for_wheres.into_iter().zip(pre_op_split_wheres))
                    .map(|(link, (init, pre))| link.as_ref().dyn_wheres(init, pre))
                    .collect(),
            )
        }

        type DeleteReturnExpression = ManyFlat<Vec<Box<dyn ManyBoxedExpressions<S> + Send>>>;

        fn delete_return_expression(&self) -> Self::DeleteReturnExpression {
            ManyFlat(
                self.iter()
                    .map(|link| link.as_ref().dyn_delete_return_expression())
                    .collect(),
            )
        }

        type DeleteReturnFromRow = JsonDeleteLinksFromRow<S>;

        fn from_row(&self) -> Self::DeleteReturnFromRow {
            JsonDeleteLinksFromRow(
                self.iter()
                    .map(|link| link.as_ref().dyn_from_row())
                    .collect(),
            )
        }

        fn take_mut(
            &self,
            links: <Self::DeleteReturnFromRow as FromRowData>::RData,
            pre_op_split_take: &mut Self::PreOpSplitTake,
        ) -> Self::Output {
            self.iter()
                .zip(links.into_iter().zip(pre_op_split_take.iter_mut()))
                .map(|(link, (row, take))| link.as_ref().dyn_take_mut(row, take))
                .collect()
        }

        fn take_once(
            &self,
            links: <Self::DeleteReturnFromRow as FromRowData>::RData,
            pre_op_split_take: Self::PreOpSplitTake,
        ) -> Self::Output {
            self.iter()
                .zip(links.into_iter().zip(pre_op_split_take))
                .map(|(link, (row, take))| link.as_ref().dyn_take_once(row, take))
                .collect()
        }
    }
}

mod supported_filters {
    use crate::{
        database_extention::DatabaseExt,
        expressions::ColumnEqual,
        json_client::{
            ToBind, client_interface::SupportedFilter, dynamic_collection::DynamicCollection,
        },
        query_builder::functional_expr::BoxedExpression,
        sub_arc::ArcSubStr,
    };

    pub fn parse_supported_filter<'q, S>(
        input: Vec<SupportedFilter>,
        base: &DynamicCollection<S>,
    ) -> Result<Vec<Box<dyn BoxedExpression<S> + Send>>, ()>
    where
        S: DatabaseExt,
        ColumnEqual<ArcSubStr, Box<dyn ToBind<S> + Send>>: BoxedExpression<S>,
    {
        let mut ret: Vec<Box<dyn BoxedExpression<S> + Send>> = vec![];
        for each in input {
            match each {
                SupportedFilter::ColEq(ColumnEqual { col, eq }) => {
                    if let Some(field) =
                        base.fields.iter().find(|f| f.name.as_str() == col.as_str())
                    {
                        if let Ok(bind) = (field.type_info.to_bind)(eq) {
                            ret.push(Box::new(ColumnEqual { col, eq: bind }));
                        } else {
                            return Err(());
                        }
                    } else {
                        return Err(());
                    }
                }
            }
        }

        Ok(ret)
    }
}

mod fetch_one_mod {
    use std::sync::Arc;

    use sqlx::ColumnIndex;

    use crate::{
        collections::Collection,
        database_extention::DatabaseExt,
        expressions::ColumnEqual,
        extentions::common_expressions::Scoped,
        fix_executor::ExecutorTrait,
        from_row::FromRowAlias,
        json_client::{
            DynManyToMany, DynOptionalToMany, DynTimestamp,
            client_interface::{
                FetchOneError, FetchOneInput, FetchOneOutput, SupportedLinkFetchOne,
            },
            dynamic_collection::{CollectionToSerialize, DynamicCollection},
            fetch_one_trait_extension::JsonLinkFetchOne,
            sqlx_executor::{FromTo, SqlxExecutorData},
            supported_filters::parse_supported_filter,
        },
        links::{
            DefaultRelationKey, relation_many_to_many::ManyToMany,
            relation_optional_to_many::OptionalToMany, timestamp::Timestamp,
        },
        operations::{Operation, fetch_one::FetchOne},
        query_builder::functional_expr::ManyFlat,
    };

    type DynCollection<S> = Arc<DynamicCollection<S>>;

    pub fn fetch_one<S>(
        this: Arc<SqlxExecutorData<S>>,
        input: FetchOneInput,
    ) -> impl Future<Output = Result<FetchOneOutput, FetchOneError>> + 'static + Send + use<S>
    where
        S: DatabaseExt + ExecutorTrait + Send + Sync,
        DynCollection<S>: for<'r> FromRowAlias<'r, S::Row, RData = CollectionToSerialize>,
        DynOptionalToMany<S>: JsonLinkFetchOne<S>,
        DynManyToMany<S>: JsonLinkFetchOne<S>,
        DynTimestamp<S>: JsonLinkFetchOne<S>,
        i64: sqlx::Type<S> + for<'q> sqlx::Decode<'q, S> + for<'q> sqlx::Encode<'q, S>,
        for<'a> &'a str: ColumnIndex<S::Row>,
    {
        async move {
            let cols = this.collections.read().await;
            let base_guard = cols
                .get(input.base.as_str())
                .ok_or(FetchOneError::CollectionNotFound)?
                .read()
                .await;
            let base = base_guard.clone();
            let rel_guard = this.link_info.read().await;
            let mut all_guards = vec![base_guard];

            let filter_exprs = parse_supported_filter(input.filters, &base)
                .map_err(|_| FetchOneError::InvalidFilter)?;

            let mut links = Vec::<Box<dyn JsonLinkFetchOne<S> + Send>>::new();

            for each in input.links {
                match each {
                    SupportedLinkFetchOne::OptionalToMany { to } => {
                        let to_guard = cols
                            .get(to.as_str())
                            .ok_or(FetchOneError::InvalidLink)?
                            .read()
                            .await;
                        let to = to_guard.clone();
                        all_guards.push(to_guard);

                        let forward = FromTo {
                            from: Arc::clone(&base.collection_name.snake_case),
                            to: Arc::clone(&to.collection_name.snake_case),
                        };

                        if rel_guard.optional_to_many.contains(&forward) {
                            links.push(Box::new(OptionalToMany {
                                fk_unique_id: DefaultRelationKey,
                                from: Arc::clone(&base),
                                to,
                            }));
                        } else {
                            return Err(FetchOneError::InvalidLink);
                        }
                    }
                    SupportedLinkFetchOne::ManyToMany { to } => {
                        let to_guard = cols
                            .get(to.as_str())
                            .ok_or(FetchOneError::InvalidLink)?
                            .read()
                            .await;
                        let to = to_guard.clone();
                        all_guards.push(to_guard);

                        let forward = FromTo {
                            from: Arc::clone(&base.collection_name.snake_case),
                            to: Arc::clone(&to.collection_name.snake_case),
                        };

                        if rel_guard.many_to_many.contains(&forward) {
                            links.push(Box::new(ManyToMany {
                                relation_key: DefaultRelationKey,
                                from: Arc::clone(&base),
                                to,
                            }));
                        } else {
                            return Err(FetchOneError::InvalidLink);
                        }
                    }
                    SupportedLinkFetchOne::Timestamp => {
                        if !rel_guard
                            .timestamped
                            .contains(base.collection_name.snake_case.as_ref())
                        {
                            return Err(FetchOneError::InvalidLink);
                        }
                        links.push(Box::new(Timestamp {
                            collection: Arc::clone(&base),
                        }));
                    }
                }
            }

            let wheres = ManyFlat((
                ColumnEqual {
                    col: base.id().scoped(),
                    eq: input.id,
                },
                ManyFlat(filter_exprs),
            ));

            let mut conn = this.pool.acquire().await.unwrap();

            let out = Operation::<S>::exec_operation(
                FetchOne {
                    base,
                    wheres,
                    links,
                },
                &mut conn,
            )
            .await;

            drop(rel_guard);
            drop(all_guards);
            drop(cols);

            out.ok_or(FetchOneError::NotFound)
        }
    }
}

mod fetch_many_mod {
    use sqlx::{ColumnIndex, Decode, Encode, Type};
    use tokio::sync::{RwLock, RwLockReadGuard};

    use crate::{
        database_extention::DatabaseExt,
        expressions::ColumnEqual,
        extentions::named_bind::NamedBind,
        fix_executor::ExecutorTrait,
        from_row::FromRowAlias,
        gen_serde::{Serialize, json_serialize_side::JsonAsString},
        json_client::{
            DynOptionalToMany, DynTimestamp, ToBind,
            client_interface::{
                FetchManyError, FetchManyInput, FetchManyOutput, FirstItem, InsertOneInput,
                InsertOneOutput, OrderBy, Pagination, SupportedInsertLink, SupportedLinkFetchMany,
            },
            dynamic_collection::{CollectionToSerialize, DynamicCollection, VTable},
            fetch_many_trait_extension::JsonLinkFetchMany,
            sqlx_executor::{FromTo, LinkInformations, SqlxExecutorData},
            supported_filters::parse_supported_filter,
        },
        links::{
            DefaultRelationKey, relation_many_to_many::ManyToMany,
            relation_optional_to_many::OptionalToMany, timestamp::Timestamp,
        },
        operations::{
            CollectionOutput, Operation,
            fetch_many::{FetchMany, ManyOutput},
        },
        query_builder::functional_expr::BoxedExpression,
        sub_arc::ArcSubStr,
    };
    use std::collections::{BTreeMap, HashMap};
    use std::sync::Arc;

    fn cursor_attributes_from_order_by(
        next: BTreeMap<String, Box<dyn Serialize<JsonAsString> + Send>>,
    ) -> CollectionToSerialize {
        CollectionToSerialize(
            next.into_iter()
                .map(|(key, value)| (Arc::<str>::from(key.as_str()), value))
                .collect(),
        )
    }

    pub fn fetch_many<S>(
        this: Arc<SqlxExecutorData<S>>,
        input: FetchManyInput,
    ) -> impl Future<Output = Result<FetchManyOutput, FetchManyError>> + 'static + Send + use<S>
    where
        S: DatabaseExt + ExecutorTrait + Send + Sync,
        Arc<DynamicCollection<S>>: for<'r> FromRowAlias<'r, S::Row, RData = CollectionToSerialize>,
        OptionalToMany<DefaultRelationKey, Arc<DynamicCollection<S>>, Arc<DynamicCollection<S>>>:
            JsonLinkFetchMany<S>,
        ManyToMany<DefaultRelationKey, Arc<DynamicCollection<S>>, Arc<DynamicCollection<S>>>:
            JsonLinkFetchMany<S>,
        Timestamp<Arc<DynamicCollection<S>>>: JsonLinkFetchMany<S>,
        i64: for<'q> Decode<'q, S> + for<'q> Encode<'q, S> + Type<S>,
        for<'s> &'s str: ColumnIndex<S::Row>,
        //connection
    {
        async move {
            let cols_gaurd = this.collections.read().await;
            let col_gaurd = cols_gaurd
                .get(input.base.as_str())
                .ok_or(FetchManyError::CollectionNotFound)?
                .read()
                .await;
            let rel_gaurd = this.link_info.read().await;

            let base = col_gaurd.clone();

            let mut all_gaurds = vec![col_gaurd];

            let wheres = parse_supported_filter(input.filters, &base)
                .map_err(|_| FetchManyError::InvalidFilter)?;

            let mut links = Vec::<Box<dyn JsonLinkFetchMany<S> + Send>>::new();

            for each in input.links {
                match each {
                    SupportedLinkFetchMany::OptionalToMany { to } => {
                        let to_collection_l = cols_gaurd
                            .get(to.as_str())
                            .ok_or(FetchManyError::InvalidLink)?
                            .read()
                            .await;

                        let to_collection = to_collection_l.clone();
                        all_gaurds.push(to_collection_l);

                        let forward = FromTo {
                            from: Arc::clone(&base.collection_name.snake_case),
                            to: Arc::clone(&to_collection.collection_name.snake_case),
                        };

                        if rel_gaurd.optional_to_many.contains(&forward) {
                            links.push(Box::new(OptionalToMany {
                                fk_unique_id: DefaultRelationKey,
                                from: Arc::clone(&base),
                                to: to_collection,
                            }));
                        } else {
                            return Err(FetchManyError::InvalidLink);
                        }
                    }
                    SupportedLinkFetchMany::ManyToMany { to } => {
                        let to_collection_l = cols_gaurd
                            .get(to.as_str())
                            .ok_or(FetchManyError::InvalidLink)?
                            .read()
                            .await;

                        let to_collection = to_collection_l.clone();
                        all_gaurds.push(to_collection_l);

                        let forward = FromTo {
                            from: Arc::clone(&base.collection_name.snake_case),
                            to: Arc::clone(&to_collection.collection_name.snake_case),
                        };

                        if rel_gaurd.many_to_many.contains(&forward) {
                            links.push(Box::new(ManyToMany {
                                relation_key: DefaultRelationKey,
                                from: Arc::clone(&base),
                                to: to_collection,
                            }));
                        } else {
                            return Err(FetchManyError::InvalidLink);
                        }
                    }
                    SupportedLinkFetchMany::Timestamp => {
                        if !rel_gaurd
                            .timestamped
                            .contains(base.collection_name.snake_case.as_ref())
                        {
                            return Err(FetchManyError::InvalidLink);
                        }
                        links.push(Box::new(Timestamp {
                            collection: Arc::clone(&base),
                        }));
                    }
                }
            }

            let limit = input.pagination.limit.clamp(0, 100);

            let order_by =
                dynamic_order_by_mod::process_order_by(&base, &input.pagination.order_by)
                    .ok_or(FetchManyError::InvalidOrderBy)?;

            let first_item = process_first_item(&base, &input.pagination.first_item)
                .map_err(|_| FetchManyError::InvalidFirstItem)?;
            let first_item = first_item.map(|item| (item.id, item.attributes));

            let mut conn = this.pool.acquire().await.unwrap();

            let s = FetchMany {
                base,
                wheres,
                links,
                limit,
                cursor_order_by: order_by,
                cursor_first_item: first_item,
            };

            let out = Operation::<S>::exec_operation(s, &mut conn).await;

            let next_item = out.next_item.map(|(id, next)| CollectionOutput {
                id,
                attributes: cursor_attributes_from_order_by(next),
            });

            drop(rel_gaurd);
            drop(all_gaurds);

            return Ok(ManyOutput {
                items: out.items,
                next_item,
            });
        }
    }

    pub mod dynamic_order_by_mod {
        use std::collections::BTreeMap;
        use std::sync::Arc;

        use sqlx::{ColumnIndex, Database, Row};

        use crate::{
            database_extention::DatabaseExt,
            extentions::common_expressions::Scoped,
            from_row::{FromRowAlias, FromRowData, FromRowError, from_row_v2::RowAliased},
            gen_serde::{Serialize, json_serialize_side::JsonAsString},
            json_client::{
                client_interface::{Direction, OrderBy},
                dynamic_collection::{DynamicCollection, VTable},
            },
            query_builder::{Expression, OpExpression, StatementBuilder},
        };

        pub struct DynamicOrderBy<S>
        where
            S: DatabaseExt,
        {
            table: Arc<str>,
            col: Arc<str>,
            sqlx_ident: VTable<S>,
            is_optional: bool,
            direction: Direction,
        }

        impl<S> Clone for DynamicOrderBy<S>
        where
            S: DatabaseExt,
        {
            fn clone(&self) -> Self {
                Self {
                    table: Arc::clone(&self.table),
                    col: Arc::clone(&self.col),
                    sqlx_ident: self.sqlx_ident.clone(),
                    is_optional: self.is_optional,
                    direction: match self.direction {
                        Direction::Asc => Direction::Asc,
                        Direction::Desc => Direction::Desc,
                    },
                }
            }
        }

        impl<S> Scoped for Vec<DynamicOrderBy<S>>
        where
            S: DatabaseExt,
        {
            type Scoped = Vec<DynamicOrderBy<S>>;

            fn scoped(&self) -> Self::Scoped {
                self.clone()
            }
        }

        impl<S> OpExpression for DynamicOrderBy<S> where S: DatabaseExt + Database {}

        impl<'q, S> Expression<'q, S> for DynamicOrderBy<S>
        where
            S: DatabaseExt,
        {
            fn expression(self, ctx: &mut StatementBuilder<'q, S>) {
                ctx.sanitize(self.table.as_ref());
                ctx.syntax(".");
                ctx.sanitize(self.col.as_ref());
                ctx.syntax(" ");
                match self.direction {
                    Direction::Asc => ctx.syntax("ASC"),
                    Direction::Desc => ctx.syntax("DESC"),
                }
            }
        }

        impl<S> FromRowData for DynamicOrderBy<S>
        where
            S: DatabaseExt,
        {
            type RData = (String, Box<dyn Serialize<JsonAsString> + Send>);
        }

        impl<'r, S> FromRowAlias<'r, S::Row> for DynamicOrderBy<S>
        where
            S: DatabaseExt + Database,
            for<'a> &'a str: ColumnIndex<S::Row>,
        {
            fn no_alias(&self, row: &'r S::Row) -> Result<Self::RData, FromRowError> {
                let value =
                    (self.sqlx_ident.decode_from_row)(self.is_optional, self.col.as_ref(), row)
                        .map_err(FromRowError::ColumnNotFound)?;

                Ok((self.col.to_string(), value))
            }

            fn pre_alias(
                &self,
                row: crate::from_row::RowPreAliased<'r, S::Row>,
            ) -> Result<Self::RData, FromRowError>
            where
                S::Row: Row,
            {
                let col_name = format!("{}{}", row.alias, self.col.as_ref());
                let value =
                    (self.sqlx_ident.decode_from_row)(self.is_optional, col_name.as_str(), row.row)
                        .map_err(FromRowError::ColumnNotFound)?;

                Ok((self.col.to_string(), value))
            }

            fn post_alias(
                &self,
                row: crate::from_row::RowPostAliased<'r, S::Row>,
            ) -> Result<Self::RData, FromRowError>
            where
                S::Row: Row,
            {
                self.no_alias(row.get_sqlx_row())
            }

            fn two_alias(
                &self,
                row: crate::from_row::RowTwoAliased<'r, S::Row>,
            ) -> Result<Self::RData, FromRowError>
            where
                S::Row: Row,
            {
                let col_name = format!(
                    "{}{}{}",
                    row.str_alias,
                    row.num_alias.map(|n| n.to_string()).unwrap_or_default(),
                    self.col.as_ref()
                );
                let value =
                    (self.sqlx_ident.decode_from_row)(self.is_optional, col_name.as_str(), row.row)
                        .map_err(FromRowError::ColumnNotFound)?;

                Ok((self.col.to_string(), value))
            }
        }

        impl<S> FromRowData for Vec<DynamicOrderBy<S>>
        where
            S: DatabaseExt,
        {
            type RData = BTreeMap<String, Box<dyn Serialize<JsonAsString> + Send>>;
        }

        impl<'r, S> FromRowAlias<'r, S::Row> for Vec<DynamicOrderBy<S>>
        where
            S: DatabaseExt + Database,
            for<'a> &'a str: ColumnIndex<S::Row>,
        {
            fn no_alias(&self, row: &'r S::Row) -> Result<Self::RData, FromRowError> {
                let mut ret = BTreeMap::new();
                for each in self {
                    let (col, value) = each.no_alias(row)?;
                    ret.insert(col, value);
                }
                Ok(ret)
            }

            fn pre_alias(
                &self,
                row: crate::from_row::RowPreAliased<'r, S::Row>,
            ) -> Result<Self::RData, FromRowError>
            where
                S::Row: Row,
            {
                let mut ret = BTreeMap::new();
                for each in self {
                    let (col, value) = each.pre_alias(row.clone())?;
                    ret.insert(col, value);
                }
                Ok(ret)
            }

            fn post_alias(
                &self,
                row: crate::from_row::RowPostAliased<'r, S::Row>,
            ) -> Result<Self::RData, FromRowError>
            where
                S::Row: Row,
            {
                self.no_alias(row.get_sqlx_row())
            }

            fn two_alias(
                &self,
                row: crate::from_row::RowTwoAliased<'r, S::Row>,
            ) -> Result<Self::RData, FromRowError>
            where
                S::Row: Row,
            {
                let mut ret = BTreeMap::new();
                for each in self {
                    let (col, value) = each.two_alias(row.clone())?;
                    ret.insert(col, value);
                }
                Ok(ret)
            }
        }

        pub fn process_order_by<S>(
            base: &DynamicCollection<S>,
            order_by: &[OrderBy],
        ) -> Option<Vec<DynamicOrderBy<S>>>
        where
            S: DatabaseExt,
        {
            let mut ret = vec![];

            for each in order_by {
                let found = base
                    .fields
                    .iter()
                    .find(|field| field.name.as_str() == each.col.as_str())?;

                ret.push(DynamicOrderBy {
                    table: Arc::clone(&base.collection_name.snake_case),
                    col: Arc::clone(&found.name.snake_case),
                    sqlx_ident: found.type_info.clone(),
                    is_optional: found.is_optional,
                    direction: match each.direction {
                        Direction::Asc => Direction::Asc,
                        Direction::Desc => Direction::Desc,
                    },
                });
            }

            Some(ret)
        }
    }

    fn process_first_item<S>(
        base: &DynamicCollection<S>,
        pagination: &Option<FirstItem>,
    ) -> Result<
        Option<
            CollectionOutput<i64, Vec<NamedBind<Arc<str>, Arc<str>, Box<dyn ToBind<S> + Send>>>>,
        >,
        (),
    >
    where
        S: DatabaseExt,
    {
        if let Some(first_item) = pagination {
            let mut attributes = vec![];

            for (key, value) in first_item.data.iter() {
                let found = base.fields.iter().find(|f| f.name.as_str() == key.as_str());
                let found = found.ok_or(())?;
                let bind = (found.type_info.to_bind)(value.clone()).map_err(|_| ())?;

                attributes.push(NamedBind {
                    table: Arc::clone(&base.collection_name.snake_case),
                    name: key.detach(),
                    value: bind,
                });
            }

            Ok(Some(CollectionOutput {
                id: first_item.id,
                attributes,
            }))
        } else {
            Ok(None)
        }
    }

    // fetch_many_link_trait lives in `pub mod fetch_many` below (gen_serde, no v1).
}

mod sqlx_executor {
    use crate::json_client::client_interface::{
        AddCollectionInput, Client, ClientOperationError, ClientOperationInput,
        ClientOperationOutput, SupportedType,
    };
    use crate::json_client::dynamic_collection::DynamicCollection;
    use crate::{
        database_extention::DatabaseExt, on_migrate::OnMigrate, query_builder::Expression,
    };
    use sqlx::{IntoArguments, Pool, Sqlite};
    use std::collections::HashSet;
    use std::{
        collections::HashMap,
        marker::PhantomData,
        sync::{Arc, Weak},
    };
    use tokio::sync::{RwLock as Trw, mpsc as tokio_mpsc};

    pub struct SqlxExecutor<S>
    where
        S: sqlx::Database + DatabaseExt,
    {
        pub(crate) reciever: tokio::sync::mpsc::UnboundedReceiver<(
            ClientOperationInput,
            oneshot::Sender<Result<ClientOperationOutput, ClientOperationError>>,
        )>,
        pub(crate) data: Arc<SqlxExecutorData<S>>,
    }

    pub(crate) struct SqlxExecutorData<S>
    where
        S: sqlx::Database + DatabaseExt,
    {
        pub(crate) collections: Trw<HashMap<Arc<str>, Trw<Arc<DynamicCollection<S>>>>>,
        pub(crate) migration: Trw<Vec<String>>,
        pub(crate) link_info: Trw<LinkInformations>,
        pub(crate) pool: Pool<S>,
        _s: PhantomData<S>,
    }
    #[derive(Default, Debug)]
    pub struct LinkInformations {
        pub optional_to_many: HashSet<FromTo>,
        pub many_to_many: HashSet<FromTo>,
        pub timestamped: HashSet<Arc<str>>,
    }

    #[derive(Debug, Hash, PartialEq, Eq)]
    pub struct FromTo {
        pub from: Arc<str>,
        pub to: Arc<str>,
    }

    impl Client {
        pub fn new_sqlx_db<S>(pool: Pool<S>) -> (Self, SqlxExecutor<S>)
        where
            S: DatabaseExt,
            bool: for<'d> sqlx::Decode<'d, S> + sqlx::Type<S> + for<'q> sqlx::Encode<'q, S>,
            std::string::String:
                sqlx::Type<S> + for<'q> sqlx::Encode<'q, S> + for<'d> sqlx::Decode<'d, S>,
            DynamicCollection<S>: OnMigrate<Statements: Expression<'static, S>>,
            for<'a> S::Arguments<'a>: IntoArguments<'a, S>,
        {
            let (sender, reciever) = tokio_mpsc::unbounded_channel::<(
                ClientOperationInput,
                oneshot::Sender<Result<ClientOperationOutput, ClientOperationError>>,
            )>();

            let data = Arc::new(SqlxExecutorData {
                collections: Trw::new(Default::default()),
                migration: Trw::new(Default::default()),
                link_info: Trw::new(Default::default()),
                pool,
                _s: PhantomData,
            });

            (Client { sender }, SqlxExecutor { reciever, data })
        }
    }
}

pub mod fetch_one_trait_extension {
    pub use super::fetch_many_trait_extension::JsonLinkFetchMany as JsonLinkFetchOne;
}

mod fetch_many_trait_extension {
    use core::fmt;
    use std::any::Any;
    use std::ops::{Deref, DerefMut};

    use sqlx::Database;
    use tracing::warn;

    use crate::{
        database_extention::DatabaseExt,
        extentions::common_expressions::Aliased,
        from_row::{FromRowAlias, FromRowData},
        gen_serde::{Serialize, SerializedJson, json_serialize_side::JsonAsString},
        operations::{OperationOutput, boxed_operation::BoxedOperation, fetch_many::LinkFetch},
        query_builder::ManyBoxedExpressions,
        select_items_trait_object::{SelectItemsTraitObject, ToImplSelectItems},
    };

    pub trait JsonLinkFetchMany<S> {
        fn select_items_expr(&self) -> Box<dyn SelectItemsTraitObject<S, ()>>;
        fn post_select_each_2(&self, item: &Box<dyn Any + Send>, poi: &mut Box<dyn Any + Send>);
        fn post_operation_input_init_2(&self) -> Box<dyn Any + Send>;
        fn post_select_2(&self, input: Box<dyn Any + Send>) -> Box<dyn BoxedOperation<S> + Send>;
        fn take_2(
            &self,
            item: Box<dyn Any + Send>,
            op: &mut Box<dyn Any + Send>,
        ) -> Box<dyn Serialize<JsonAsString> + Send>;
        fn join_expr(&self) -> Box<dyn ManyBoxedExpressions<S> + Send>;
        fn wheres_expr(&self) -> Box<dyn ManyBoxedExpressions<S> + Send>;
    }

    impl<S, T> JsonLinkFetchMany<S> for T
    where
        T: Clone + Send + 'static,
        T::SelectItems: Send,
        T::SelectItems: FromRowData,
        S: DatabaseExt,
        T: LinkFetch,
        T::SelectItems: Send
            + Aliased<
                NumAliased: 'static + Send + ManyBoxedExpressions<S>,
                Aliased: 'static + Send + ManyBoxedExpressions<S>,
            >,
        T::OpInput: 'static + Send,
        T::Op: Send + 'static + BoxedOperation<S>,
        T::Op: OperationOutput,
        T::Output: Serialize<JsonAsString>,
        T::SelectItems: FromRowData<RData: Send + 'static>,
        T::SelectItems: for<'r> FromRowAlias<'r, S::Row>,
        T::Join: Send + 'static + ManyBoxedExpressions<S>,
        T::Wheres: Send + 'static + ManyBoxedExpressions<S>,
        T::Output: Send + fmt::Debug,
    {
        fn join_expr(&self) -> Box<dyn ManyBoxedExpressions<S> + Send> {
            Box::new(self.non_duplicating_join_expressions())
        }

        fn wheres_expr(&self) -> Box<dyn ManyBoxedExpressions<S> + Send> {
            Box::new(self.where_expressions())
        }

        fn take_2(
            &self,
            item: Box<dyn Any + Send>,
            op: &mut Box<dyn Any + Send>,
        ) -> Box<dyn Serialize<JsonAsString> + Send> {
            let output = self.take_many(
                *item
                    .downcast::<<T::SelectItems as FromRowData>::RData>()
                    .unwrap(),
                op.downcast_mut::<<T::Op as OperationOutput>::Output>()
                    .unwrap(),
            );

            Box::new(output)
        }

        fn select_items_expr(&self) -> Box<dyn SelectItemsTraitObject<S, ()>> {
            Box::new(ToImplSelectItems {
                select_items: self.non_aggregating_select_items(),
                cast_from_row_result: (),
            })
        }

        fn post_select_each_2(&self, item: &Box<dyn Any + Send>, poi: &mut Box<dyn Any + Send>) {
            let item = item
                .deref()
                .downcast_ref::<<T::SelectItems as FromRowData>::RData>()
                .unwrap();
            let poi = poi.deref_mut().downcast_mut::<T::OpInput>().unwrap();

            self.operation_fix_on_many(item, poi)
        }

        fn post_operation_input_init_2(&self) -> Box<dyn Any + Send> {
            Box::new(self.operation_initialize_input())
        }

        fn post_select_2(&self, input: Box<dyn Any + Send>) -> Box<dyn BoxedOperation<S> + Send> {
            Box::new(self.operation_construct(*input.downcast::<T::OpInput>().unwrap()))
        }
    }

    impl<'r, S> LinkFetch for Box<dyn JsonLinkFetchMany<S> + Send + 'r>
    where
        Box<dyn SelectItemsTraitObject<S, ()>>: FromRowData<RData = Box<dyn Any + Send>>,
        Box<dyn BoxedOperation<S> + Send>: OperationOutput<Output = Box<dyn Any + Send>>,
    {
        type SelectItems = Box<dyn SelectItemsTraitObject<S, ()>>;

        fn non_aggregating_select_items(&self) -> Self::SelectItems {
            self.select_items_expr()
        }

        fn operation_fix_on_many(&self, item: &Box<dyn Any + Send>, poi: &mut Self::OpInput)
        where
            Self::SelectItems: FromRowData,
        {
            self.post_select_each_2(item, poi)
        }

        fn take_many(
            &self,
            item: <Self::SelectItems as FromRowData>::RData,
            op: &mut <Self::Op as OperationOutput>::Output,
        ) -> Self::Output
        where
            Self::SelectItems: FromRowData,
            Self::Op: OperationOutput,
        {
            self.take_2(item, op)
        }

        type Join = Box<dyn ManyBoxedExpressions<S> + Send>;

        fn non_duplicating_join_expressions(&self) -> Self::Join {
            self.join_expr()
        }

        type Wheres = Box<dyn ManyBoxedExpressions<S> + Send>;

        fn where_expressions(&self) -> Self::Wheres {
            self.wheres_expr()
        }

        type Output = Box<dyn Serialize<JsonAsString> + Send>;

        type OpInput = Box<dyn Any + Send>;

        fn operation_initialize_input(&self) -> Self::OpInput {
            self.post_operation_input_init_2()
        }

        type Op = Box<dyn BoxedOperation<S> + Send>;

        fn operation_construct(&self, input: Self::OpInput) -> Self::Op
        where
            Self::SelectItems: FromRowData,
        {
            self.post_select_2(input)
        }
    }

    impl<'r, S> LinkFetch for Vec<Box<dyn JsonLinkFetchMany<S> + Send + 'r>>
    where
        S: Database,
        Vec<Box<dyn SelectItemsTraitObject<S, ()>>>: FromRowData<RData = Vec<Box<dyn Any + Send>>>,
        Vec<Box<dyn BoxedOperation<S> + Send>>: OperationOutput<Output = Vec<Box<dyn Any + Send>>>,
    {
        type SelectItems = Vec<Box<dyn SelectItemsTraitObject<S, ()>>>;

        fn non_aggregating_select_items(&self) -> Self::SelectItems {
            self.iter().map(|each| each.select_items_expr()).collect()
        }

        fn operation_fix_on_many(
            &self,
            item: &Vec<Box<dyn Any + Send>>,
            poi: &mut Vec<Box<dyn Any + Send>>,
        ) where
            Self::SelectItems: FromRowData,
        {
            for (i, each) in self.iter().enumerate() {
                each.post_select_each_2(&item[i], &mut poi[i]);
            }
        }

        fn take_many(
            &self,
            item: Vec<Box<dyn Any + Send>>,
            op: &mut Vec<Box<dyn Any + Send>>,
        ) -> Self::Output
        where
            Self::SelectItems: FromRowData,
            Self::Op: OperationOutput,
        {
            self.iter()
                .zip(item)
                .zip(op.iter_mut())
                .map(|((each, item), op)| each.take_2(item, op))
                .collect()
        }

        type Join = Box<dyn ManyBoxedExpressions<S> + Send>;

        fn non_duplicating_join_expressions(&self) -> Self::Join {
            warn!("multiple links");
            Box::new(self.first().unwrap().join_expr())
        }

        type Wheres = ();

        fn where_expressions(&self) -> Self::Wheres {}

        type Output = Vec<Box<dyn Serialize<JsonAsString> + Send>>;

        type OpInput = Vec<Box<dyn Any + Send>>;

        fn operation_initialize_input(&self) -> Self::OpInput {
            self.iter()
                .map(|each| each.post_operation_input_init_2())
                .collect()
        }

        type Op = Vec<Box<dyn BoxedOperation<S> + Send>>;

        fn operation_construct(&self, input: Self::OpInput) -> Self::Op
        where
            Self::SelectItems: FromRowData,
        {
            self.iter()
                .zip(input)
                .map(|(each, input)| each.post_select_2(input))
                .collect()
        }
    }
}

pub mod dynamic_collection {
    use crate::{
        database_extention::DatabaseExt,
        gen_serde::{
            Deserialize, Serialize,
            json_format_side::{JsonAsArcCursor, PartialDeserialize},
            json_serialize_side::JsonAsString,
        },
        json_client::{
            ToBind,
            client_interface::{AddCollectionInput, SupportedType},
        },
        query_builder::{SyntaxAsType, functional_expr::BoxedExpression},
        sub_arc::ArcSubStr,
    };
    use convert_case::{Case, Casing};
    use core::fmt;
    use sqlx::{ColumnIndex, Decode, Encode, Row, Type};
    use std::ops::Not;
    use std::sync::Arc;
    use std::{any::TypeId, marker::PhantomData};

    /// Registry metadata: owned `Arc<str>` so request JSON (`ArcSubStr`) can be dropped after add.
    #[derive(Debug)]
    pub struct DynamicCollection<S>
    where
        S: sqlx::Database + DatabaseExt,
    {
        pub(crate) collection_name: CollectionName,
        pub(crate) fields: Vec<DynamicField<S>>,
    }

    #[derive(Debug)]
    pub(crate) struct DynamicField<S>
    where
        S: sqlx::Database + DatabaseExt,
    {
        pub(crate) name: FieldName,
        pub(crate) type_info: VTable<S>,
        pub(crate) is_optional: bool,
    }

    #[derive(Debug, Clone)]
    pub struct CollectionName {
        pub(crate) pascal_case: Arc<str>,
        pub(crate) snake_case: Arc<str>,
    }

    impl CollectionName {
        pub fn new<T: AsRef<str>>(value: &T) -> Result<Self, ()> {
            let name = value.as_ref();
            if name.is_empty()
                || name.is_case(Case::Snake).not()
                || name.starts_with("ct_")
                || name.starts_with("meta_")
            {
                return Err(());
            }

            Ok(Self {
                pascal_case: Arc::from(name.to_case(Case::Pascal)),
                snake_case: Arc::from(name),
            })
        }
    }

    impl PartialEq for CollectionName {
        fn eq(&self, other: &Self) -> bool {
            self.snake_case == other.snake_case
        }
    }

    impl Eq for CollectionName {}

    #[derive(Debug, Clone)]
    pub struct FieldName {
        pub(crate) snake_case: Arc<str>,
    }

    impl FieldName {
        pub fn new<T: AsRef<str>>(value: &T) -> Result<Self, ()> {
            let name = value.as_ref();
            if name.is_empty()
                || name.is_case(Case::Snake).not()
                || name.starts_with("id")
                || name.starts_with("fk_")
            {
                return Err(());
            }

            Ok(Self {
                snake_case: Arc::from(name),
            })
        }

        pub fn as_str(&self) -> &str {
            &self.snake_case
        }
    }

    impl AsRef<str> for FieldName {
        fn as_ref(&self) -> &str {
            &self.snake_case
        }
    }

    impl PartialEq for FieldName {
        fn eq(&self, other: &Self) -> bool {
            self.snake_case == other.snake_case
        }
    }

    impl Eq for FieldName {}

    type DynamicRowJsonValue = Result<Box<dyn Serialize<JsonAsString> + Send>, String>;

    macro_rules! vtable {
        (impl<T,S> $([$($wh:tt)*] ,)* {
            $(
                fn $fn_name:ident($($args:ident: $arg_type:ty),*) -> $return_type:ty {$body:tt}
            )*
        }) => {
            pub struct VTable<S>
            where S: DatabaseExt {
                $(
                    pub $fn_name: fn ($($args: $arg_type),*) -> $return_type,
                )*
                _s: PhantomData<S>,
            }

            impl<S> VTable<S>
                where S: DatabaseExt
            {
                pub fn new_as<T>() -> Self
                    where $( $($wh)*,)*
                {
                    Self {
                        $(
                            $fn_name: |$($args: $arg_type),*| -> $return_type {
                                $body
                            }
                        ,)*
                        _s: PhantomData,
                    }
                }
            }

            impl<S> Clone for VTable<S>
            where
                S: DatabaseExt,
            {
                fn clone(&self) -> Self {
                    Self {
                        $(
                            $fn_name: self.$fn_name,
                        )*
                        _s: PhantomData,
                    }
                }
            }
        };
    }

    vtable! {
        impl<T,S>
            [ T: Send + Sync + 'static ],
            [ for<'a> &'a str: ColumnIndex<S::Row> ],
            [ S: DatabaseExt + sqlx::Database ],
            [ T: for<'q> Decode<'q, S> + for<'q> Encode<'q, S> + Type<S> + Clone ],
            [ T: crate::expressions::is_null::IsNull ],
            [ T: Serialize<JsonAsString> ],
            [
                T: for<'de> Deserialize<'de, JsonAsArcCursor, Handler = ()>
            ],
        {
            fn type_name() -> &'static str {{
                std::any::type_name::<T>()
            }}
            fn type_id() -> TypeId {{
                TypeId::of::<T>()
            }}
            fn to_bind(partial: PartialDeserialize) -> Result<Box<dyn ToBind<S> + Send>, ()> {{
                partial
                    .continue_deserialize::<T>()
                    .map(|value| Box::new(value) as Box<dyn ToBind<S> + Send>)
                    .map_err(|_| ())
            }}
            fn type_expression() -> Box<dyn BoxedExpression<S> + Send> {{
                Box::new(SyntaxAsType::<T>(PhantomData))
            }}
            fn decode_from_row(is_optional: bool, name: &str, row: &S::Row) -> DynamicRowJsonValue {{
                let value: Option<T> = row
                    .try_get(name)
                    .map_err(|e| format!("decode {:?}: {e}", name))?;
                if !is_optional && value.is_none() {
                    return Err(format!("column {:?} is null", name));
                }
                Ok(Box::new(value) as Box<dyn Serialize<JsonAsString> + Send>)
            }}
            fn partial_to_row_value(is_optional: bool, partial: PartialDeserialize) -> DynamicRowJsonValue {{
                if is_optional && partial.0.as_str().trim() == "null" {
                    return Ok(Box::new(None::<T>) as Box<dyn Serialize<JsonAsString> + Send>);
                }
                partial
                    .continue_deserialize::<T>()
                    .map(|value| Box::new(Some(value)) as Box<dyn Serialize<JsonAsString> + Send>)
                    .map_err(|_| format!("invalid value for dynamic row field"))
            }}
        }
    }

    impl<S> fmt::Debug for VTable<S>
    where
        S: DatabaseExt,
    {
        fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
            write!(f, "VTable({})", (self.type_name)())
        }
    }

    impl<S> PartialEq for VTable<S>
    where
        S: DatabaseExt,
    {
        fn eq(&self, other: &Self) -> bool {
            (self.type_name)() == (other.type_name)()
        }
    }

    impl<S> Eq for VTable<S> where S: DatabaseExt {}

    impl<S> TryFrom<AddCollectionInput> for DynamicCollection<S>
    where
        S: sqlx::Database + DatabaseExt,
        String: for<'q> sqlx::Encode<'q, S> + sqlx::Type<S> + for<'d> sqlx::Decode<'d, S>,
        bool: for<'q> sqlx::Encode<'q, S> + sqlx::Type<S> + for<'d> sqlx::Decode<'d, S>,
        for<'a> &'a str: sqlx::ColumnIndex<S::Row>,
    {
        type Error = ();

        fn try_from(input: AddCollectionInput) -> Result<Self, Self::Error> {
            let collection_name = CollectionName::new(&input.name)?;
            let fields = input
                .fields
                .into_iter()
                .map(|f| {
                    Ok(DynamicField {
                        name: FieldName::new(&f.name)?,
                        type_info: match f.type_info {
                            SupportedType::String => VTable::new_as::<String>(),
                            SupportedType::Boolean => VTable::new_as::<bool>(),
                        },
                        is_optional: f.is_optional,
                    })
                })
                .collect::<Result<Vec<_>, _>>()?;

            Ok(Self {
                collection_name,
                fields,
            })
        }
    }

    mod dynamic_insert_binds {
        use std::{collections::HashMap, sync::Arc};

        use crate::{
            database_extention::DatabaseExt,
            gen_serde::{
                Deserialize, DeserializeMap, DeserializeMapOrSeq, DeserializeSpec, Deserializer,
                KnownKey, MapOrSeq, UnknownKey, json_format_side::PartialDeserialize,
            },
            json_client::{
                ToBind,
                dynamic_collection::{DynamicCollection, DynamicField},
            },
            sub_arc::ArcSubStr,
        };

        use super::DynamicInsertInput;

        impl<S> DeserializeSpec for DynamicInsertInput<S>
        where
            S: sqlx::Database + DatabaseExt,
        {
            type Handler = Arc<DynamicCollection<S>>;
        }

        impl<'de, D, S> Deserialize<'de, D> for DynamicInsertInput<S>
        where
            S: sqlx::Database + DatabaseExt,
            String: for<'q> sqlx::Encode<'q, S> + sqlx::Type<S>,
            bool: for<'q> sqlx::Encode<'q, S> + sqlx::Type<S>,
            D: Deserializer<'de>,
            D: DeserializeMapOrSeq<'de>,
            D: DeserializeMap<'de>,
            ArcSubStr: Deserialize<'de, D> + UnknownKey<D>,
            PartialDeserialize: Deserialize<'de, D>,
            <D as Deserializer<'de>>::Err: From<String>,
        {
            fn deserialize(
                collection: Self::Handler,
                serialized: &mut D,
            ) -> Result<Self, <D as Deserializer<'de>>::Err> {
                match serialized.start_map_or_seq()? {
                    MapOrSeq::Map(mut map_access) => {
                        let mut out: DynamicInsertInput<S> = HashMap::new();

                        while serialized.map_has_next(&map_access) {
                            let (key, partial): (ArcSubStr, PartialDeserialize) =
                                DeserializeMap::deserialize_with_unknown_key(
                                    serialized,
                                    &mut map_access,
                                    (),
                                    (),
                                )?;

                            let field = collection
                                .fields
                                .iter()
                                .find(|field| field.name.as_ref() == key.as_str())
                                .ok_or_else(|| {
                                    format!("unknown field {:?}", key.as_str()).into()
                                })?;

                            let bind = if partial.0.as_str().trim() == "null" {
                                if !field.is_optional {
                                    return Err(
                                        format!("field {:?} cannot be null", key.as_str()).into()
                                    );
                                }
                                Box::new(()) as Box<dyn ToBind<S> + Send>
                            } else {
                                (field.type_info.to_bind)(partial).map_err(|_| {
                                    format!("invalid value for {:?}", key.as_str()).into()
                                })?
                            };

                            let key_str = key.as_str().to_string();
                            if out.insert(key, bind).is_some() {
                                return Err(format!("duplicate field {:?}", key_str).into());
                            }
                        }

                        DeserializeMapOrSeq::finish_map(serialized, map_access)?;

                        for field in &collection.fields {
                            if !field.is_optional
                                && !out.keys().any(|key| key.as_str() == field.name.as_ref())
                            {
                                return Err(format!(
                                    "missing required field {:?}",
                                    field.name.as_ref()
                                )
                                .into());
                            }
                        }

                        Ok(out)
                    }
                    MapOrSeq::Seq(_) => {
                        Err("expected JSON object for insert data".to_string().into())
                    }
                }
            }
        }

        pub struct DynamicUpdateInput<S>(pub HashMap<ArcSubStr, Box<dyn ToBind<S> + Send>>);

        impl<S> DeserializeSpec for DynamicUpdateInput<S>
        where
            S: sqlx::Database + DatabaseExt,
        {
            type Handler = Arc<DynamicCollection<S>>;
        }

        impl<'de, D, S> Deserialize<'de, D> for DynamicUpdateInput<S>
        where
            S: sqlx::Database + DatabaseExt,
            String: for<'q> sqlx::Encode<'q, S> + sqlx::Type<S>,
            bool: for<'q> sqlx::Encode<'q, S> + sqlx::Type<S>,
            D: Deserializer<'de>,
            D: DeserializeMapOrSeq<'de>,
            D: DeserializeMap<'de>,
            ArcSubStr: Deserialize<'de, D> + UnknownKey<D>,
            PartialDeserialize: Deserialize<'de, D>,
            <D as Deserializer<'de>>::Err: From<String>,
        {
            fn deserialize(
                collection: Self::Handler,
                serialized: &mut D,
            ) -> Result<Self, <D as Deserializer<'de>>::Err> {
                match serialized.start_map_or_seq()? {
                    MapOrSeq::Map(mut map_access) => {
                        let mut out: DynamicUpdateInput<S> = DynamicUpdateInput(HashMap::new());

                        while serialized.map_has_next(&map_access) {
                            let (key, partial): (ArcSubStr, PartialDeserialize) =
                                DeserializeMap::deserialize_with_unknown_key(
                                    serialized,
                                    &mut map_access,
                                    (),
                                    (),
                                )?;

                            let field = collection
                                .fields
                                .iter()
                                .find(|field| field.name.as_ref() == key.as_str())
                                .ok_or_else(|| {
                                    format!("unknown field {:?}", key.as_str()).into()
                                })?;

                            let bind = if partial.0.as_str().trim() == "null" {
                                if !field.is_optional {
                                    return Err(
                                        format!("field {:?} cannot be null", key.as_str()).into()
                                    );
                                }
                                Box::new(()) as Box<dyn ToBind<S> + Send>
                            } else {
                                (field.type_info.to_bind)(partial).map_err(|_| {
                                    format!("invalid value for {:?}", key.as_str()).into()
                                })?
                            };

                            let key_str = key.as_str().to_string();
                            if out.0.insert(key, bind).is_some() {
                                return Err(format!("duplicate field {:?}", key_str).into());
                            }
                        }

                        DeserializeMapOrSeq::finish_map(serialized, map_access)?;

                        Ok(out)
                    }
                    MapOrSeq::Seq(_) => {
                        Err("expected JSON object for update data".to_string().into())
                    }
                }
            }
        }
    }

    pub(crate) mod collection_impls {
        use std::{
            collections::HashMap,
            fmt,
            ops::{Deref, DerefMut},
            sync::Arc,
        };

        use crate::{
            collections::{Collection, SingleIncremintalInt},
            database_extention::DatabaseExt,
            gen_serde::{
                Deserialize, Serialize, deserialize,
                json_format_side::{JsonAsArcCursor, JsonFormat, PartialDeserialize},
                json_serialize_side::JsonAsString,
            },
            json_client::ToBind,
            sub_arc::ArcSubStr,
        };

        use super::DynamicCollection;

        pub(crate) type DynamicInput<S> = HashMap<ArcSubStr, Box<dyn ToBind<S> + Send>>;

        #[derive(Default)]
        pub struct CollectionToSerialize(
            pub(crate) HashMap<std::sync::Arc<str>, Box<dyn Serialize<JsonAsString> + Send>>,
        );

        impl Deref for CollectionToSerialize {
            type Target = HashMap<std::sync::Arc<str>, Box<dyn Serialize<JsonAsString> + Send>>;

            fn deref(&self) -> &Self::Target {
                &self.0
            }
        }

        impl DerefMut for CollectionToSerialize {
            fn deref_mut(&mut self) -> &mut Self::Target {
                &mut self.0
            }
        }

        impl fmt::Debug for CollectionToSerialize {
            fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
                f.debug_map().entries(self.0.iter()).finish()
            }
        }

        impl PartialEq for CollectionToSerialize {
            fn eq(&self, other: &Self) -> bool {
                self.0.len() == other.0.len()
                    && self.0.iter().all(|(key, value)| {
                        other.0.get(key).is_some_and(|other| {
                            let left = value.as_ref() as &dyn Serialize<JsonAsString>;
                            let right = other.as_ref() as &dyn Serialize<JsonAsString>;
                            left == right
                        })
                    })
            }
        }

        impl Eq for CollectionToSerialize {}

        impl Clone for CollectionToSerialize {
            fn clone(&self) -> Self {
                use crate::gen_serde::{SerializedJson, json_serialize_side::JsonAsString};

                Self(
                    self.0
                        .iter()
                        .map(|(key, value)| {
                            let mut serialized = JsonAsString(String::new());
                            value.serialize(&mut serialized);
                            (
                                Arc::clone(key),
                                Box::new(SerializedJson(Arc::from(serialized.0.as_str())))
                                    as Box<dyn Serialize<JsonAsString> + Send>,
                            )
                        })
                        .collect(),
                )
            }
        }

        impl Serialize<JsonAsString> for CollectionToSerialize {
            fn serialize(&self, ctx: &mut JsonAsString) {
                let mut keys: Vec<_> = self.0.keys().collect();
                keys.sort();
                ctx.0.push('{');
                for (i, key) in keys.iter().enumerate() {
                    if i > 0 {
                        ctx.0.push(',');
                    }
                    Serialize::serialize(key.as_ref(), ctx);
                    ctx.0.push(':');
                    Serialize::serialize(self.0.get(*key).unwrap().as_ref(), ctx);
                }
                ctx.0.push('}');
            }
        }

        impl Serialize<JsonAsString> for crate::operations::CollectionOutput<i64, CollectionToSerialize> {
            fn serialize(&self, ctx: &mut JsonAsString) {
                ctx.0.push('{');
                Serialize::serialize("id", ctx);
                ctx.0.push(':');
                self.id.serialize(ctx);
                ctx.0.push(',');
                Serialize::serialize("attributes", ctx);
                ctx.0.push(':');
                self.attributes.serialize(ctx);
                ctx.0.push('}');
            }
        }

        pub(crate) fn decode_json_scalar<T>(value: &Box<dyn Serialize<JsonAsString> + Send>) -> T
        where
            T: for<'de> Deserialize<'de, JsonAsArcCursor, Handler = ()>,
        {
            let mut json = JsonAsString::default();
            Serialize::serialize(value.as_ref(), &mut json);
            deserialize(Arc::from(json.0), (), JsonFormat).expect("decode json scalar")
        }

        pub(crate) fn dynamic_row_from_json<S>(
            json: std::sync::Arc<str>,
            dc: &std::sync::Arc<DynamicCollection<S>>,
        ) -> CollectionToSerialize
        where
            S: sqlx::Database + DatabaseExt,
            String: for<'q> sqlx::Encode<'q, S> + sqlx::Type<S>,
            bool: for<'q> sqlx::Encode<'q, S> + sqlx::Type<S>,
        {
            use crate::gen_serde::{
                Deserialize, DeserializeMap, DeserializeMapOrSeq, FromSerialized, MapOrSeq,
                json_format_side::PartialDeserialize,
            };
            use crate::sub_arc::ArcSubStr;

            let mut row = CollectionToSerialize::default();
            let mut cursor = json.start();

            match cursor.start_map_or_seq().expect("expected JSON object") {
                MapOrSeq::Map(mut map) => {
                    while cursor.map_has_next(&mut map) {
                        let (key, partial): (ArcSubStr, PartialDeserialize) =
                            DeserializeMap::deserialize_with_unknown_key(
                                &mut cursor,
                                &mut map,
                                (),
                                (),
                            )
                            .expect("decode json field");

                        let Some(field) = dc
                            .fields
                            .iter()
                            .find(|field| field.name.as_ref() == key.as_str())
                        else {
                            continue;
                        };

                        let value =
                            (field.type_info.partial_to_row_value)(field.is_optional, partial)
                                .expect("decode json field value");

                        row.insert(Arc::clone(&field.name.snake_case), value);
                    }
                    DeserializeMapOrSeq::finish_map(&mut cursor, map).expect("finish json object");
                }
                MapOrSeq::Seq(_) => panic!("expected JSON object"),
            }

            row
        }

        pub(crate) fn serialize_dynamic_row(row: &CollectionToSerialize, keys: &[&str]) -> String {
            let mut json = JsonAsString::default();
            json.0.push('{');
            for (i, key) in keys.iter().enumerate() {
                if i > 0 {
                    json.0.push(',');
                }
                Serialize::serialize(*key, &mut json);
                json.0.push(':');
                let value = row
                    .get(*key)
                    .unwrap_or_else(|| panic!("missing field {key:?}"));
                Serialize::serialize(value.as_ref(), &mut json);
            }
            json.0.push('}');
            json.into()
        }

        impl<S> Collection for Arc<DynamicCollection<S>>
        where
            S: sqlx::Database + DatabaseExt,
        {
            fn table_name(&self) -> &str {
                &self.collection_name.pascal_case
            }

            fn table_name_lower_case(&self) -> &str {
                &self.collection_name.snake_case
            }

            type InputData = DynamicInput<S>;
            type UpdateData = DynamicInput<S>;
            type OutputData = CollectionToSerialize;
            type Id = SingleIncremintalInt<Arc<str>>;

            fn id(&self) -> Self::Id {
                SingleIncremintalInt(Arc::clone(&self.collection_name.pascal_case))
            }
        }
    }

    pub(crate) use collection_impls::{CollectionToSerialize, DynamicInput as DynamicInsertInput};
    pub(crate) use collection_impls::{
        decode_json_scalar, dynamic_row_from_json, serialize_dynamic_row,
    };
    pub(crate) use dynamic_insert_binds::DynamicUpdateInput;

    pub(crate) mod common_expression_impls {
        use std::{ops::Not, sync::Arc};

        use crate::{
            database_extention::DatabaseExt,
            extentions::common_expressions::{
                Aliased, Identifier, OnInsert, TableNameExpression, V0OnUpdate,
            },
            from_row::{FromRowAlias, FromRowData, FromRowError, from_row_v2::RowAliased},
            json_client::{
                ToBind,
                dynamic_collection::{
                    CollectionToSerialize, DynamicCollection, DynamicField, DynamicInsertInput,
                    DynamicUpdateInput,
                },
            },
            query_builder::{Expression, IsOpExpression, ManyExpressions, StatementBuilder},
            sub_arc::ArcSubStr,
        };
        use sqlx::{ColumnIndex, Database, Row};

        pub struct ToBindSetMany<S> {
            pub vec: Vec<Box<dyn ToBind<S> + Send>>,
        }

        impl<S> IsOpExpression for ToBindSetMany<S> {
            fn is_op(&self) -> bool {
                self.vec.is_empty().not()
            }
        }

        impl<'q, S> ManyExpressions<'q, S> for ToBindSetMany<S>
        where
            S: DatabaseExt,
        {
            fn expression(
                mut self,
                start: &'static str,
                join: &'static str,
                ctx: &mut StatementBuilder<'q, S>,
            ) where
                S: DatabaseExt,
            {
                if self.vec.is_empty() {
                    return;
                }

                ctx.syntax(start);
                let last = self.vec.pop();

                for each in self.vec {
                    ctx.bind(each);
                    ctx.syntax(join);
                }

                if let Some(value) = last {
                    ctx.bind(value);
                }
            }
        }

        pub struct StoredMemberNames {
            pub cols: Vec<Arc<str>>,
        }

        impl IsOpExpression for StoredMemberNames {
            fn is_op(&self) -> bool {
                self.cols.is_empty().not()
            }
        }

        impl<'q, S> ManyExpressions<'q, S> for StoredMemberNames
        where
            S: DatabaseExt,
        {
            fn expression(
                mut self,
                start: &'static str,
                join: &'static str,
                ctx: &mut StatementBuilder<'q, S>,
            ) where
                S: DatabaseExt,
            {
                if self.cols.is_empty() {
                    return;
                }

                ctx.syntax(start);
                let last = self.cols.pop();

                for col in self.cols {
                    Expression::expression(col, ctx);
                    ctx.syntax(join);
                }

                if let Some(col) = last {
                    Expression::expression(col, ctx);
                }
            }
        }

        impl<S> TableNameExpression for DynamicCollection<S>
        where
            S: sqlx::Database + DatabaseExt,
        {
            type TableNameExpression = Arc<str>;
            type LowerCaseTableNameExpression = Arc<str>;

            fn table_name_expression(&self) -> Self::TableNameExpression {
                Arc::clone(&self.collection_name.pascal_case)
            }

            fn lower_case_table_name_expression(&self) -> Self::LowerCaseTableNameExpression {
                Arc::clone(&self.collection_name.snake_case)
            }
        }

        impl<S> Identifier for DynamicCollection<S>
        where
            S: sqlx::Database + DatabaseExt,
        {
            type Identifier = StoredMemberNames;

            fn identifier(&self) -> Self::Identifier {
                StoredMemberNames {
                    cols: self
                        .fields
                        .iter()
                        .map(|field| Arc::clone(&field.name.snake_case))
                        .collect(),
                }
            }
        }

        impl<S> crate::extentions::Members for DynamicCollection<S>
        where
            S: sqlx::Database + DatabaseExt,
        {
            fn members_names(&self) -> Vec<String> {
                self.fields
                    .iter()
                    .map(|field| field.name.snake_case.to_string())
                    .collect()
            }
        }

        impl<S> OnInsert for DynamicCollection<S>
        where
            S: sqlx::Database + DatabaseExt,
        {
            type InsertInput = DynamicInsertInput<S>;
            type InsertExpression = ToBindSetMany<S>;

            fn on_insert(&self, input: Self::InsertInput) -> Self::InsertExpression {
                let mut vec = Vec::with_capacity(self.fields.len());
                for field in &self.fields {
                    let bind = input
                        .iter()
                        .find(|(key, _)| key.as_str() == field.name.as_ref())
                        .map(|(_, value)| value.clone_to_box())
                        .unwrap_or_else(|| Box::new(()) as Box<dyn ToBind<S> + Send>);
                    vec.push(bind);
                }
                ToBindSetMany { vec }
            }
        }

        impl<S> FromRowData for DynamicCollection<S>
        where
            S: sqlx::Database + DatabaseExt,
        {
            type RData = CollectionToSerialize;
        }

        impl<'r, S> FromRowAlias<'r, S::Row> for DynamicCollection<S>
        where
            S: DatabaseExt + sqlx::Database,
            for<'a> &'a str: ColumnIndex<S::Row>,
            String: for<'d> sqlx::Decode<'d, S> + sqlx::Type<S>,
            bool: for<'d> sqlx::Decode<'d, S> + sqlx::Type<S>,
        {
            fn no_alias(&self, row: &'r S::Row) -> Result<Self::RData, FromRowError> {
                let mut map = CollectionToSerialize::default();
                for field in &self.fields {
                    let value = (field.type_info.decode_from_row)(
                        field.is_optional,
                        field.name.as_ref(),
                        row,
                    )
                    .map_err(|message| FromRowError::ColumnNotFound(message))?;
                    map.insert(Arc::clone(&field.name.snake_case), value);
                }
                Ok(map)
            }

            fn pre_alias(
                &self,
                row: crate::from_row::RowPreAliased<'r, S::Row>,
            ) -> Result<Self::RData, FromRowError>
            where
                S::Row: Row,
            {
                let mut map = CollectionToSerialize::default();
                for field in &self.fields {
                    let col_name = format!("{}{}", row.alias, field.name.as_ref());
                    let value = (field.type_info.decode_from_row)(
                        field.is_optional,
                        col_name.as_str(),
                        row.row,
                    )
                    .map_err(|message| FromRowError::ColumnNotFound(message))?;
                    map.insert(Arc::clone(&field.name.snake_case), value);
                }
                Ok(map)
            }

            fn post_alias(
                &self,
                row: crate::from_row::RowPostAliased<'r, S::Row>,
            ) -> Result<Self::RData, FromRowError>
            where
                S::Row: Row,
            {
                self.no_alias(row.get_sqlx_row())
            }

            fn two_alias(
                &self,
                row: crate::from_row::RowTwoAliased<'r, S::Row>,
            ) -> Result<Self::RData, FromRowError>
            where
                S::Row: Row,
            {
                let mut map = CollectionToSerialize::default();
                for field in &self.fields {
                    let col_name = format!(
                        "{}{}{}",
                        row.str_alias,
                        row.num_alias.map(|n| n.to_string()).unwrap_or_default(),
                        field.name.as_ref()
                    );
                    let value = (field.type_info.decode_from_row)(
                        field.is_optional,
                        col_name.as_str(),
                        row.row,
                    )
                    .map_err(FromRowError::ColumnNotFound)?;
                    map.insert(Arc::clone(&field.name.snake_case), value);
                }
                Ok(map)
            }
        }

        pub struct DynamicAliasedMembers {
            pub table: Arc<str>,
            pub cols: Vec<Arc<str>>,
            pub alias: &'static str,
            pub num: Option<usize>,
        }

        impl IsOpExpression for DynamicAliasedMembers {
            fn is_op(&self) -> bool {
                self.cols.is_empty().not()
            }
        }

        impl<'q, S> ManyExpressions<'q, S> for DynamicAliasedMembers
        where
            S: DatabaseExt,
        {
            fn expression(
                mut self,
                start: &'static str,
                join: &'static str,
                ctx: &mut StatementBuilder<'q, S>,
            ) where
                S: DatabaseExt,
            {
                if self.cols.is_empty() {
                    return;
                }

                ctx.syntax(start);
                let last = self.cols.pop();

                for col in &self.cols {
                    ctx.sanitize(self.table.as_ref());
                    ctx.syntax(".");
                    ctx.sanitize(col.as_ref());
                    ctx.syntax(" AS ");
                    match self.num {
                        None => ctx.sanitize_strings((self.alias, col.as_ref())),
                        Some(num) => ctx.sanitize_strings((self.alias, num, col.as_ref())),
                    }
                    ctx.syntax(join);
                }

                if let Some(col) = last {
                    ctx.sanitize(self.table.as_ref());
                    ctx.syntax(".");
                    ctx.sanitize(col.as_ref());
                    ctx.syntax(" AS ");
                    match self.num {
                        None => ctx.sanitize_strings((self.alias, col.as_ref())),
                        Some(num) => ctx.sanitize_strings((self.alias, num, col.as_ref())),
                    }
                }
            }
        }

        impl<S> Aliased for DynamicCollection<S>
        where
            S: sqlx::Database + DatabaseExt,
        {
            type Aliased = DynamicAliasedMembers;
            type NumAliased = DynamicAliasedMembers;

            fn aliased(&self, alias: &'static str) -> Self::Aliased {
                DynamicAliasedMembers {
                    table: Arc::clone(&self.collection_name.pascal_case),
                    cols: self
                        .fields
                        .iter()
                        .map(|field| Arc::clone(&field.name.snake_case))
                        .collect(),
                    alias,
                    num: None,
                }
            }

            fn num_aliased(&self, num: usize, alias: &'static str) -> Self::NumAliased {
                DynamicAliasedMembers {
                    table: Arc::clone(&self.collection_name.pascal_case),
                    cols: self
                        .fields
                        .iter()
                        .map(|field| Arc::clone(&field.name.snake_case))
                        .collect(),
                    alias,
                    num: Some(num),
                }
            }
        }

        pub struct DynamicUpdateSet<S> {
            pub sets: Vec<(Arc<str>, Box<dyn ToBind<S> + Send>)>,
        }

        impl<S> IsOpExpression for DynamicUpdateSet<S> {
            fn is_op(&self) -> bool {
                self.sets.is_empty().not()
            }
        }

        impl<'q, S> ManyExpressions<'q, S> for DynamicUpdateSet<S>
        where
            S: DatabaseExt,
        {
            fn expression(
                mut self,
                start: &'static str,
                join: &'static str,
                ctx: &mut StatementBuilder<'q, S>,
            ) where
                S: DatabaseExt,
            {
                if self.sets.is_empty() {
                    return;
                }

                ctx.syntax(start);
                let last = self.sets.pop();

                for (name, bind) in self.sets {
                    ctx.sanitize(name.as_ref());
                    ctx.syntax(" = ");
                    ctx.bind(bind);
                    ctx.syntax(join);
                }

                if let Some((name, bind)) = last {
                    ctx.sanitize(name.as_ref());
                    ctx.syntax(" = ");
                    ctx.bind(bind);
                }
            }
        }

        impl<S> crate::extentions::common_expressions::V0OnUpdate for DynamicCollection<S>
        where
            S: sqlx::Database + DatabaseExt,
        {
            type UpdateInput = DynamicUpdateInput<S>;
            type UpdateExpression = DynamicUpdateSet<S>;

            fn on_update(self, input: Self::UpdateInput) -> Self::UpdateExpression {
                DynamicUpdateSet {
                    sets: input
                        .0
                        .into_iter()
                        .map(|(key, value)| (ArcSubStr::detach(&key), value))
                        .collect(),
                }
            }
        }
    }

    mod arc_collection_impls {
        use std::sync::Arc;

        use crate::{
            database_extention::DatabaseExt,
            extentions::common_expressions::{
                Aliased, Identifier, OnInsert, TableNameExpression, V0OnUpdate,
            },
            from_row::{FromRowAlias, FromRowData, FromRowError},
            json_client::dynamic_collection::{
                CollectionToSerialize, DynamicCollection, DynamicInsertInput, DynamicUpdateInput,
                common_expression_impls::{DynamicUpdateSet, ToBindSetMany},
            },
            sub_arc::ArcSubStr,
        };
        use sqlx::{ColumnIndex, Database, Row};

        impl<S> TableNameExpression for Arc<DynamicCollection<S>>
        where
            S: sqlx::Database + DatabaseExt,
        {
            type TableNameExpression = Arc<str>;
            type LowerCaseTableNameExpression = Arc<str>;

            fn table_name_expression(&self) -> Self::TableNameExpression {
                TableNameExpression::table_name_expression(self.as_ref())
            }

            fn lower_case_table_name_expression(&self) -> Self::LowerCaseTableNameExpression {
                TableNameExpression::lower_case_table_name_expression(self.as_ref())
            }
        }

        impl<S> Identifier for Arc<DynamicCollection<S>>
        where
            S: sqlx::Database + DatabaseExt,
        {
            type Identifier = super::common_expression_impls::StoredMemberNames;

            fn identifier(&self) -> Self::Identifier {
                Identifier::identifier(self.as_ref())
            }
        }

        impl<S> crate::extentions::Members for Arc<DynamicCollection<S>>
        where
            S: sqlx::Database + DatabaseExt,
        {
            fn members_names(&self) -> Vec<String> {
                crate::extentions::Members::members_names(self.as_ref())
            }
        }

        impl<S> OnInsert for Arc<DynamicCollection<S>>
        where
            S: sqlx::Database + DatabaseExt,
        {
            type InsertInput = DynamicInsertInput<S>;
            type InsertExpression = ToBindSetMany<S>;

            fn on_insert(&self, input: Self::InsertInput) -> Self::InsertExpression {
                OnInsert::on_insert(self.as_ref(), input)
            }
        }

        impl<S> Aliased for Arc<DynamicCollection<S>>
        where
            S: sqlx::Database + DatabaseExt,
        {
            type Aliased = super::common_expression_impls::DynamicAliasedMembers;
            type NumAliased = super::common_expression_impls::DynamicAliasedMembers;

            fn aliased(&self, alias: &'static str) -> Self::Aliased {
                Aliased::aliased(self.as_ref(), alias)
            }

            fn num_aliased(&self, num: usize, alias: &'static str) -> Self::NumAliased {
                Aliased::num_aliased(self.as_ref(), num, alias)
            }
        }

        impl<S> V0OnUpdate for Arc<DynamicCollection<S>>
        where
            S: sqlx::Database + DatabaseExt,
            DynamicCollection<S>: V0OnUpdate,
        {
            type UpdateInput = DynamicUpdateInput<S>;
            type UpdateExpression = DynamicUpdateSet<S>;

            fn on_update(self, input: Self::UpdateInput) -> Self::UpdateExpression {
                DynamicUpdateSet {
                    sets: input
                        .0
                        .into_iter()
                        .map(|(key, value)| (ArcSubStr::detach(&key), value))
                        .collect(),
                }
            }
        }

        impl<S> FromRowData for Arc<DynamicCollection<S>>
        where
            S: sqlx::Database + DatabaseExt,
        {
            type RData = CollectionToSerialize;
        }

        impl<'r, S> FromRowAlias<'r, S::Row> for Arc<DynamicCollection<S>>
        where
            S: DatabaseExt + sqlx::Database,
            DynamicCollection<S>: FromRowData<RData = CollectionToSerialize>,
            DynamicCollection<S>: FromRowAlias<'r, S::Row, RData = CollectionToSerialize>,
        {
            fn no_alias(&self, row: &'r S::Row) -> Result<Self::RData, FromRowError> {
                self.as_ref().no_alias(row)
            }

            fn pre_alias(
                &self,
                row: crate::from_row::RowPreAliased<'r, S::Row>,
            ) -> Result<Self::RData, FromRowError>
            where
                S::Row: Row,
            {
                self.as_ref().pre_alias(row)
            }

            fn post_alias(
                &self,
                row: crate::from_row::RowPostAliased<'r, S::Row>,
            ) -> Result<Self::RData, FromRowError>
            where
                S::Row: Row,
            {
                self.as_ref().post_alias(row)
            }

            fn two_alias(
                &self,
                row: crate::from_row::RowTwoAliased<'r, S::Row>,
            ) -> Result<Self::RData, FromRowError>
            where
                S::Row: Row,
            {
                self.as_ref().two_alias(row)
            }
        }
    }

    mod std_impls {
        use crate::database_extention::DatabaseExt;

        use super::{DynamicCollection, DynamicField};

        impl<S: DatabaseExt> PartialEq for DynamicCollection<S> {
            fn eq(&self, other: &Self) -> bool {
                self.collection_name == other.collection_name && self.fields == other.fields
            }
        }

        impl<S: DatabaseExt> Eq for DynamicCollection<S> {}

        impl<S: DatabaseExt> Clone for DynamicField<S> {
            fn clone(&self) -> Self {
                Self {
                    name: self.name.clone(),
                    type_info: self.type_info.clone(),
                    is_optional: self.is_optional,
                }
            }
        }

        impl<S: DatabaseExt> PartialEq for DynamicField<S> {
            fn eq(&self, other: &Self) -> bool {
                self.name == other.name
                    && self.type_info == other.type_info
                    && self.is_optional == other.is_optional
            }
        }

        impl<S: DatabaseExt> Eq for DynamicField<S> {}
    }

    mod impl_on_migrate {
        use std::ops::Not;
        use std::sync::Arc;

        use super::DynamicCollection;
        use crate::{
            database_extention::DatabaseExt,
            json_client::dynamic_collection::DynamicField,
            on_migrate::OnMigrate,
            query_builder::{
                Expression, OpExpression, StatementBuilder,
                essential_syntax::{CLOSE_PARANTHESIS, OPEN_PARANTHESIS},
            },
        };
        use sqlx::ColumnIndex;

        pub struct MigrateDynamicCollection<S: DatabaseExt> {
            upper_case_name: Arc<str>,
            fields: Vec<DynamicField<S>>,
        }

        impl<S> OnMigrate for DynamicCollection<S>
        where
            S: DatabaseExt,
            for<'a> &'a str: ColumnIndex<S::Row>,
            bool: sqlx::Type<S>,
            std::string::String: sqlx::Type<S>,
        {
            type Statements = MigrateDynamicCollection<S>;

            fn statments(&self) -> Self::Statements {
                MigrateDynamicCollection {
                    upper_case_name: Arc::clone(&self.collection_name.pascal_case),
                    fields: self.fields.clone(),
                }
            }
        }

        impl<S: DatabaseExt> OpExpression for MigrateDynamicCollection<S> {}

        impl<'q, S> Expression<'q, S> for MigrateDynamicCollection<S>
        where
            S: DatabaseExt,
            S::IdExpression: Expression<'q, S>,
        {
            fn expression(self, ctx: &mut StatementBuilder<'q, S>) {
                ctx.syntax("CREATE TABLE ");
                ctx.sanitize(self.upper_case_name.as_ref());
                ctx.syntax(" ");
                ctx.syntax(OPEN_PARANTHESIS);

                S::id_on_create_table_expression().expression(ctx);

                for field in self.fields.into_iter() {
                    ctx.syntax(&", ");
                    ctx.sanitize(field.name.as_str());
                    ctx.syntax(" ");
                    (field.type_info.type_expression)().expression(ctx);
                    if field.is_optional.not() {
                        ctx.syntax(&" NOT NULL");
                    }
                }
                ctx.syntax(CLOSE_PARANTHESIS);
                ctx.syntax(";");
            }
        }
    }

    macro_rules! default_executor {
        ($([$name:ident, $upper_case:ident]),*) => {};
    }
}

#[cfg(test)]
mod tests {
    use sqlx::Sqlite;

    use crate::{
        connect_in_memory::ConnectInMemory, json_client::client_interface::Client,
        track_sqlx_query::watch_sqlx_calls,
    };

    use super::test_utilities::{
        add_category_collection, add_todo_collection, clear_timestams, setup_todo_collection,
        setup_todo_with_category_link, todo_is_one_to_many_with_category, todo_is_timestamped,
    };

    #[tokio::test(flavor = "current_thread")]
    async fn test_insert_one() {
        watch_sqlx_calls(async |scope, cache| {
            let pool = Sqlite::in_memory_pool().await;
            let (client, ex) = Client::new_sqlx_db(pool);
            let client = client.into_string_client();

            scope.spawn(ex.run());

            setup_todo_collection(&client, &cache).await;

            client
                .exec(
                    r#"
{
    "op": "insert_one",
    "body": {
        "base": "todo",
        "data": {
            "title": "test_todo",
            "done": false,
            "description": "test_description"
        },
        "links": []
    }
}
"#
                    .to_string(),
                )
                .await;

            pretty_assertions::assert_eq!(
                cache.drain(),
                vec![
                    r#"PRAGMA foreign_keys = ON;"#.to_string(),
                    r#"INSERT INTO "Todo" ("title", "description", "done") VALUES ($1, $2, $3) RETURNING "id", "title", "description", "done";"#.to_string(),
                ]
            );
        })
        .await;
    }

    #[tokio::test(flavor = "current_thread")]
    async fn exec_returns_invalid_input_for_non_json() {
        let pool = Sqlite::in_memory_pool().await;
        let (client, ex) = Client::new_sqlx_db(pool);
        let client = client.into_string_client();
        let _executor = tokio::spawn(ex.run());

        let result = client.exec(r#"not json"#.to_string()).await;
        pretty_assertions::assert_eq!(result, r#"{"error":"invalid_input"}"#);
    }

    #[tokio::test(flavor = "current_thread")]
    async fn exec_returns_invalid_body_for_malformed_add_collection() {
        let pool = Sqlite::in_memory_pool().await;
        let (client, ex) = Client::new_sqlx_db(pool);
        let client = client.into_string_client();
        let _executor = tokio::spawn(ex.run());

        let result = client
            .exec(
                r#"
{
    "op": "add_collection",
    "body": ["todo", []]
}
"#
                .to_string(),
            )
            .await;
        pretty_assertions::assert_eq!(result, r#"{"error":"invalid_body"}"#);
    }

    #[tokio::test(flavor = "current_thread")]
    async fn add_collection_returns_null_output() {
        let pool = Sqlite::in_memory_pool().await;
        let (client, ex) = Client::new_sqlx_db(pool);
        let client = client.into_string_client();
        let _executor = tokio::spawn(ex.run());

        add_todo_collection(&client).await;
        add_category_collection(&client).await;

        pretty_assertions::assert_eq!(
            client
                .exec(
                    r#"
{
    "op": "add_collection",
    "body": {
        "name": "tag",
        "fields": [
            { "name": "title", "type_info": "String", "is_optional": false }
        ]
    }
}
"#
                    .to_string(),
                )
                .await,
            r#"{"output":null}"#
        );
    }

    #[tokio::test(flavor = "current_thread")]
    async fn add_collection_rejects_duplicate() {
        let pool = Sqlite::in_memory_pool().await;
        let (client, ex) = Client::new_sqlx_db(pool);
        let client = client.into_string_client();
        let _executor = tokio::spawn(ex.run());

        add_todo_collection(&client).await;
        add_category_collection(&client).await;
        todo_is_one_to_many_with_category(&client).await;

        let result = client
            .exec(
                r#"
{
    "op": "add_collection",
    "body": {
        "name": "todo",
        "fields": [
            { "name": "title", "type_info": "String", "is_optional": false }
        ]
    }
}
"#
                .to_string(),
            )
            .await;
        pretty_assertions::assert_eq!(result, r#"{"error":"CollectionAlreadyExists"}"#);
    }

    #[tokio::test(flavor = "current_thread")]
    async fn add_link_rejects_duplicate() {
        let pool = Sqlite::in_memory_pool().await;
        let (client, ex) = Client::new_sqlx_db(pool);
        let client = client.into_string_client();
        let _executor = tokio::spawn(ex.run());

        add_todo_collection(&client).await;
        add_category_collection(&client).await;
        todo_is_one_to_many_with_category(&client).await;

        let result = client
            .exec(
                r#"
{
    "op": "add_link",
    "body": {
        "ty": "optional_to_many",
        "from": "todo",
        "to": "category"
    }
}
"#
                .to_string(),
            )
            .await;
        pretty_assertions::assert_eq!(result, r#"{"error":"LinkAlreadyExists"}"#);
    }

    #[tokio::test(flavor = "current_thread")]
    async fn add_link_rejects_missing_collection() {
        let pool = Sqlite::in_memory_pool().await;
        let (client, ex) = Client::new_sqlx_db(pool);
        let client = client.into_string_client();
        let _executor = tokio::spawn(ex.run());

        add_todo_collection(&client).await;
        add_category_collection(&client).await;
        todo_is_one_to_many_with_category(&client).await;

        let result = client
            .exec(
                r#"
{
    "op": "add_link",
    "body": {
        "ty": "optional_to_many",
        "from": "todo",
        "to": "missing"
    }
}
"#
                .to_string(),
            )
            .await;
        pretty_assertions::assert_eq!(result, r#"{"error":"CollectionNotFound"}"#);
    }

    #[tokio::test(flavor = "current_thread")]
    async fn insert_one_category_returns_id_and_attributes() {
        let pool = Sqlite::in_memory_pool().await;
        let (client, ex) = Client::new_sqlx_db(pool);
        let client = client.into_string_client();
        let _executor = tokio::spawn(ex.run());

        add_todo_collection(&client).await;
        add_category_collection(&client).await;
        todo_is_one_to_many_with_category(&client).await;

        let result = client
            .exec(
                r#"
{
    "op": "insert_one",
    "body": {
        "base": "category",
        "data": { "title": "category_1" },
        "links": []
    }
}
"#
                .to_string(),
            )
            .await;

        pretty_assertions::assert_eq!(
            result,
            r#"{"output":{"id":1,"attributes":{"title":"category_1"},"links":[]}}"#
        );
    }

    #[tokio::test(flavor = "current_thread")]
    async fn update_one_updates_todo_by_id() {
        let pool = Sqlite::in_memory_pool().await;
        let (client, ex) = Client::new_sqlx_db(pool);
        let client = client.into_string_client();
        let _executor = tokio::spawn(ex.run());

        add_todo_collection(&client).await;

        client
            .exec(
                r#"
{
    "op": "insert_one",
    "body": {
        "base": "todo",
        "data": {
            "title": "before_update",
            "done": false,
            "description": "desc"
        },
        "links": []
    }
}
"#
                .to_string(),
            )
            .await;

        let result = client
            .exec(
                r#"
{
    "op": "update_one",
    "body": {
        "base": "todo",
        "id": 1,
        "data": { "title": "after_update" },
        "links": []
    }
}
"#
                .to_string(),
            )
            .await;

        pretty_assertions::assert_eq!(
            result,
            r#"{"output":{"id":1,"attributes":{"description":"desc","done":false,"title":"after_update"},"links":[]}}"#
        );
    }

    #[tokio::test(flavor = "current_thread")]
    async fn fetch_one_returns_todo_with_optional_to_many_link() {
        let pool = Sqlite::in_memory_pool().await;
        let (client, ex) = Client::new_sqlx_db(pool);
        let client = client.into_string_client();
        let _executor = tokio::spawn(ex.run());

        add_todo_collection(&client).await;
        add_category_collection(&client).await;
        todo_is_one_to_many_with_category(&client).await;

        client
            .exec(
                r#"
{
    "op": "insert_one",
    "body": {
        "base": "category",
        "data": { "title": "work" },
        "links": []
    }
}
"#
                .to_string(),
            )
            .await;

        client
            .exec(
                r#"
{
    "op": "insert_one",
    "body": {
        "base": "todo",
        "data": {
            "title": "todo_1",
            "done": true,
            "description": "desc"
        },
        "links": [
            { "ty": "set_id", "to": "category", "id": 1 }
        ]
    }
}
"#
                .to_string(),
            )
            .await;

        let result = client
            .exec(
                r#"
{
    "op": "fetch_one",
    "body": {
        "base": "todo",
        "id": 1,
        "filters": [],
        "links": [
            { "ty": "optional_to_many", "to": "category" }
        ]
    }
}
"#
                .to_string(),
            )
            .await;

        pretty_assertions::assert_eq!(
            result,
            r#"{"output":{"id":1,"attributes":{"description":"desc","done":true,"title":"todo_1"},"links":[{"id":1,"attributes":{"title":"work"}}]}}"#
        );
    }

    #[tokio::test(flavor = "current_thread")]
    async fn fetch_many_returns_inserted_todo_with_timestamp_link() {
        let pool = Sqlite::in_memory_pool().await;
        let (client, ex) = Client::new_sqlx_db(pool.clone());
        let client = client.into_string_client();
        let _executor = tokio::spawn(ex.run());

        add_todo_collection(&client).await;
        add_category_collection(&client).await;
        todo_is_one_to_many_with_category(&client).await;
        todo_is_timestamped(&client).await;

        client
            .exec(
                r#"
{
    "op": "insert_one",
    "body": {
        "base": "todo",
        "data": {
            "title": "todo_1",
            "done": true,
            "description": "description_1"
        },
        "links": []
    }
}
"#
                .to_string(),
            )
            .await;
        clear_timestams(pool.clone()).await;

        let result = client
            .exec(
                r#"
{
    "op": "fetch_many",
    "body": {
        "base": "todo",
        "filters": [],
        "links": [
            { "ty": "timestamp" }
        ],
        "pagination": { "limit": 10, "first_item": null, "order_by": [] }
    }
}
"#
                .to_string(),
            )
            .await;

        pretty_assertions::assert_eq!(
            result,
            r#"{"output":{"items":[{"id":1,"attributes":{"description":"description_1","done":true,"title":"todo_1"},"links":[{"created_at":"demo created_at","updated_at":"demo updated_at"}]}],"next_item":null}}"#
        );
    }

    #[tokio::test(flavor = "current_thread")]
    async fn fetch_many_col_eq_filter() {
        let pool = Sqlite::in_memory_pool().await;
        let (client, ex) = Client::new_sqlx_db(pool.clone());
        let client = client.into_string_client();
        let _executor = tokio::spawn(ex.run());

        add_todo_collection(&client).await;
        todo_is_timestamped(&client).await;

        client
            .exec(
                r#"
{
    "op": "insert_one",
    "body": {
        "base": "todo",
        "data": {
            "title": "done_todo",
            "done": true
        },
        "links": []
    }
}
"#
                .to_string(),
            )
            .await;

        client
            .exec(
                r#"
{
    "op": "insert_one",
    "body": {
        "base": "todo",
        "data": {
            "title": "open_todo",
            "done": false
        },
        "links": []
    }
}
"#
                .to_string(),
            )
            .await;
        clear_timestams(pool).await;

        let matching = client
            .exec(
                r#"
{
    "op": "fetch_many",
    "body": {
        "base": "todo",
        "filters": [
            { "ty": "col_eq", "col": "done", "eq": true }
        ],
        "links": [
            { "ty": "timestamp" }
        ],
        "pagination": { "limit": 10, "first_item": null, "order_by": [] }
    }
}
"#
                .to_string(),
            )
            .await;
        pretty_assertions::assert_eq!(
            matching,
            r#"{"output":{"items":[{"id":1,"attributes":{"description":null,"done":true,"title":"done_todo"},"links":[{"created_at":"demo created_at","updated_at":"demo updated_at"}]}],"next_item":null}}"#
        );

        let not_done = client
            .exec(
                r#"
{
    "op": "fetch_many",
    "body": {
        "base": "todo",
        "filters": [
            { "ty": "col_eq", "col": "done", "eq": false }
        ],
        "links": [
            { "ty": "timestamp" }
        ],
        "pagination": { "limit": 10, "first_item": null, "order_by": [] }
    }
}
"#
                .to_string(),
            )
            .await;
        pretty_assertions::assert_eq!(
            not_done,
            r#"{"output":{"items":[{"id":2,"attributes":{"description":null,"done":false,"title":"open_todo"},"links":[{"created_at":"demo created_at","updated_at":"demo updated_at"}]}],"next_item":null}}"#
        );
    }

    #[tokio::test(flavor = "current_thread")]
    async fn fetch_many_rejects_unknown_filter_field() {
        let pool = Sqlite::in_memory_pool().await;
        let (client, ex) = Client::new_sqlx_db(pool);
        let client = client.into_string_client();
        let _executor = tokio::spawn(ex.run());

        add_todo_collection(&client).await;
        todo_is_timestamped(&client).await;

        let result = client
            .exec(
                r#"
{
    "op": "fetch_many",
    "body": {
        "base": "todo",
        "filters": [
            { "ty": "col_eq", "col": "missing", "eq": "x" }
        ],
        "links": [
            { "ty": "timestamp" }
        ],
        "pagination": { "limit": 10, "first_item": null, "order_by": [] }
    }
}
"#
                .to_string(),
            )
            .await;
        pretty_assertions::assert_eq!(result, r#"{"error":"InvalidFilter"}"#);
    }

    #[tokio::test(flavor = "current_thread")]
    async fn fetch_many_rejects_filter_type_mismatch() {
        let pool = Sqlite::in_memory_pool().await;
        let (client, ex) = Client::new_sqlx_db(pool);
        let client = client.into_string_client();
        let _executor = tokio::spawn(ex.run());

        add_todo_collection(&client).await;
        todo_is_timestamped(&client).await;

        let result = client
            .exec(
                r#"
{
    "op": "fetch_many",
    "body": {
        "base": "todo",
        "filters": [
            { "ty": "col_eq", "col": "done", "eq": "not_a_bool" }
        ],
        "links": [
            { "ty": "timestamp" }
        ],
        "pagination": { "limit": 10, "first_item": null, "order_by": [] }
    }
}
"#
                .to_string(),
            )
            .await;
        pretty_assertions::assert_eq!(result, r#"{"error":"InvalidFilter"}"#);
    }

    mod insert_one {
        use sqlx::Sqlite;

        use crate::{
            connect_in_memory::ConnectInMemory, json_client::client_interface::Client,
            track_sqlx_query::watch_sqlx_calls,
        };

        use crate::json_client::test_utilities::{
            add_category_collection, add_todo_collection, setup_todo_with_category_link,
            todo_is_one_to_many_with_category,
        };

        #[tokio::test(flavor = "current_thread")]
        async fn set_id_links_existing_category() {
            let pool = Sqlite::in_memory_pool().await;
            let (client, ex) = Client::new_sqlx_db(pool);
            let client = client.into_string_client();
            let _executor = tokio::spawn(ex.run());

            add_todo_collection(&client).await;
            add_category_collection(&client).await;
            todo_is_one_to_many_with_category(&client).await;

            client
                .exec(
                    r#"
{
    "op": "insert_one",
    "body": {
        "base": "category",
        "data": { "title": "existing_category" },
        "links": []
    }
}
"#
                    .to_string(),
                )
                .await;

            let result = client
                .exec(
                    r#"
{
    "op": "insert_one",
    "body": {
        "base": "todo",
        "data": {
            "title": "todo_with_category",
            "done": true,
            "description": "linked"
        },
        "links": [
            { "ty": "set_id", "to": "category", "id": 1 }
        ]
    }
}
"#
                    .to_string(),
                )
                .await;

            pretty_assertions::assert_eq!(
                result,
                r#"{"output":{"id":1,"attributes":{"description":"linked","done":true,"title":"todo_with_category"},"links":[{"id":1,"attributes":{"title":"existing_category"}}]}}"#
            );
        }

        #[tokio::test(flavor = "current_thread")]
        async fn set_new_creates_category_and_links() {
            let pool = Sqlite::in_memory_pool().await;
            let (client, ex) = Client::new_sqlx_db(pool);
            let client = client.into_string_client();
            let _executor = tokio::spawn(ex.run());

            add_todo_collection(&client).await;
            add_category_collection(&client).await;
            todo_is_one_to_many_with_category(&client).await;

            let result = client
                .exec(
                    r#"
{
    "op": "insert_one",
    "body": {
        "base": "todo",
        "data": {
            "title": "todo_with_new_category",
            "done": false,
            "description": "set_new"
        },
        "links": [
            { "ty": "set_new", "to": "category", "value": { "title": "new_category" } }
        ]
    }
}
"#
                    .to_string(),
                )
                .await;

            pretty_assertions::assert_eq!(
                result,
                r#"{"output":{"id":1,"attributes":{"description":"set_new","done":false,"title":"todo_with_new_category"},"links":[{"id":1,"attributes":{"title":"new_category"}}]}}"#
            );
        }

        #[tokio::test(flavor = "current_thread")]
        async fn set_id_sql() {
            watch_sqlx_calls(async |scope, cache| {
                let pool = Sqlite::in_memory_pool().await;
                let (client, ex) = Client::new_sqlx_db(pool);
                let client = client.into_string_client();

                scope.spawn(ex.run());

                setup_todo_with_category_link(&client, &cache).await;

                client
                    .exec(
                        r#"
{
    "op": "insert_one",
    "body": {
        "base": "category",
        "data": { "title": "cat_for_set_id" },
        "links": []
    }
}
"#
                        .to_string(),
                    )
                    .await;

                pretty_assertions::assert_eq!(
                    cache.drain(),
                    vec![
                        "INSERT INTO \"Category\" (\"title\") VALUES ($1) RETURNING \"id\", \"title\";".to_string(),
                    ]
                );

                client
                    .exec(
                        r#"
{
    "op": "insert_one",
    "body": {
        "base": "todo",
        "data": {
            "title": "todo_set_id",
            "done": true
        },
        "links": [
            { "ty": "set_id", "to": "category", "id": 1 }
        ]
    }
}
"#
                        .to_string(),
                    )
                    .await;

                pretty_assertions::assert_eq!(
                    cache.drain(),
                    vec![
                        "INSERT INTO \"Todo\" (\"title\", \"description\", \"done\", \"fk_category_def\") VALUES ($1, $2, $3, $4) RETURNING \"id\", \"title\", \"description\", \"done\", \"fk_category_def\";".to_string(),
                        "SELECT \"Category\".\"id\" AS \"iid\", \"Category\".\"title\" AS \"btitle\" FROM \"Category\" WHERE \"id\" = $1;".to_string(),
                    ]
                );
            })
            .await;
        }
    }

    mod update_one {
        use sqlx::Sqlite;

        use crate::{
            connect_in_memory::ConnectInMemory, json_client::client_interface::Client,
            track_sqlx_query::watch_sqlx_calls,
        };

        use crate::json_client::test_utilities::{
            add_category_collection, add_todo_collection, setup_todo_with_category_link,
            todo_is_one_to_many_with_category,
        };

        #[tokio::test(flavor = "current_thread")]
        async fn set_id_links_existing_category() {
            let pool = Sqlite::in_memory_pool().await;
            let (client, ex) = Client::new_sqlx_db(pool);
            let client = client.into_string_client();
            let _executor = tokio::spawn(ex.run());

            add_todo_collection(&client).await;
            add_category_collection(&client).await;
            todo_is_one_to_many_with_category(&client).await;

            client
                .exec(
                    r#"
{
    "op": "insert_one",
    "body": {
        "base": "category",
        "data": { "title": "existing_category" },
        "links": []
    }
}
"#
                    .to_string(),
                )
                .await;

            client
                .exec(
                    r#"
{
    "op": "insert_one",
    "body": {
        "base": "todo",
        "data": {
            "title": "todo_to_update",
            "done": false,
            "description": "before"
        },
        "links": []
    }
}
"#
                    .to_string(),
                )
                .await;

            let result = client
                .exec(
                    r#"
{
    "op": "update_one",
    "body": {
        "base": "todo",
        "id": 1,
        "data": { "title": "linked_todo" },
        "links": [
            { "ty": "set_id", "to": "category", "id": 1 }
        ]
    }
}
"#
                    .to_string(),
                )
                .await;

            pretty_assertions::assert_eq!(
                result,
                r#"{"output":{"id":1,"attributes":{"description":"before","done":false,"title":"linked_todo"},"links":[1]}}"#
            );
        }

        #[tokio::test(flavor = "current_thread")]
        async fn set_new_creates_category_and_links() {
            let pool = Sqlite::in_memory_pool().await;
            let (client, ex) = Client::new_sqlx_db(pool);
            let client = client.into_string_client();
            let _executor = tokio::spawn(ex.run());

            add_todo_collection(&client).await;
            add_category_collection(&client).await;
            todo_is_one_to_many_with_category(&client).await;

            client
                .exec(
                    r#"
{
    "op": "insert_one",
    "body": {
        "base": "todo",
        "data": {
            "title": "todo_to_update",
            "done": true,
            "description": "set_new"
        },
        "links": []
    }
}
"#
                    .to_string(),
                )
                .await;

            let result = client
                .exec(
                    r#"
{
    "op": "update_one",
    "body": {
        "base": "todo",
        "id": 1,
        "data": { "description": "updated" },
        "links": [
            { "ty": "set_new", "to": "category", "value": { "title": "new_category" } }
        ]
    }
}
"#
                    .to_string(),
                )
                .await;

            pretty_assertions::assert_eq!(
                result,
                r#"{"output":{"id":1,"attributes":{"description":"updated","done":true,"title":"todo_to_update"},"links":[{"id":1,"attributes":{"title":"new_category"}}]}}"#
            );
        }

        #[tokio::test(flavor = "current_thread")]
        async fn set_null_clears_category_link() {
            let pool = Sqlite::in_memory_pool().await;
            let (client, ex) = Client::new_sqlx_db(pool);
            let client = client.into_string_client();
            let _executor = tokio::spawn(ex.run());

            add_todo_collection(&client).await;
            add_category_collection(&client).await;
            todo_is_one_to_many_with_category(&client).await;

            client
                .exec(
                    r#"
{
    "op": "insert_one",
    "body": {
        "base": "category",
        "data": { "title": "cat_to_clear" },
        "links": []
    }
}
"#
                    .to_string(),
                )
                .await;

            client
                .exec(
                    r#"
{
    "op": "insert_one",
    "body": {
        "base": "todo",
        "data": {
            "title": "todo_linked",
            "done": false,
            "description": "before_null"
        },
        "links": [
            { "ty": "set_id", "to": "category", "id": 1 }
        ]
    }
}
"#
                    .to_string(),
                )
                .await;

            let result = client
                .exec(
                    r#"
{
    "op": "update_one",
    "body": {
        "base": "todo",
        "id": 1,
        "data": {},
        "links": [
            { "ty": "set_null", "to": "category" }
        ]
    }
}
"#
                    .to_string(),
                )
                .await;

            pretty_assertions::assert_eq!(
                result,
                r#"{"output":{"id":1,"attributes":{"description":"before_null","done":false,"title":"todo_linked"},"links":[0]}}"#
            );
        }

        #[tokio::test(flavor = "current_thread")]
        async fn link_sql() {
            watch_sqlx_calls(async |scope, cache| {
                let pool = Sqlite::in_memory_pool().await;
                let (client, ex) = Client::new_sqlx_db(pool);
                let client = client.into_string_client();

                scope.spawn(ex.run());

                setup_todo_with_category_link(&client, &cache).await;

                client
                    .exec(
                        r#"
{
    "op": "insert_one",
    "body": {
        "base": "category",
        "data": { "title": "cat_for_update_set_id" },
        "links": []
    }
}
"#
                        .to_string(),
                    )
                    .await;

                pretty_assertions::assert_eq!(
                    cache.drain(),
                    vec![
                        "INSERT INTO \"Category\" (\"title\") VALUES ($1) RETURNING \"id\", \"title\";".to_string(),
                    ]
                );

                client
                    .exec(
                        r#"
{
    "op": "insert_one",
    "body": {
        "base": "todo",
        "data": {
            "title": "todo_before_update",
            "done": false,
            "description": "desc"
        },
        "links": []
    }
}
"#
                        .to_string(),
                    )
                    .await;

                pretty_assertions::assert_eq!(
                    cache.drain(),
                    vec![
                        "INSERT INTO \"Todo\" (\"title\", \"description\", \"done\") VALUES ($1, $2, $3) RETURNING \"id\", \"title\", \"description\", \"done\";".to_string(),
                    ]
                );

                client
                    .exec(
                        r#"
{
    "op": "update_one",
    "body": {
        "base": "todo",
        "id": 1,
        "data": { "title": "todo_after_update" },
        "links": [
            { "ty": "set_id", "to": "category", "id": 1 }
        ]
    }
}
"#
                        .to_string(),
                    )
                    .await;

                pretty_assertions::assert_eq!(
                    cache.drain(),
                    vec![
                        "UPDATE \"Todo\" SET \"title\" = $1, \"fk_category_def\" = $2 WHERE \"Todo\".\"id\" = $3 RETURNING \"id\", \"title\", \"description\", \"done\", \"fk_category_def\";".to_string(),
                    ]
                );
            })
            .await;

            watch_sqlx_calls(async |scope, cache| {
                let pool = Sqlite::in_memory_pool().await;
                let (client, ex) = Client::new_sqlx_db(pool);
                let client = client.into_string_client();

                scope.spawn(ex.run());

                setup_todo_with_category_link(&client, &cache).await;

                client
                    .exec(
                        r#"
{
    "op": "insert_one",
    "body": {
        "base": "todo",
        "data": {
            "title": "todo_set_new",
            "done": true,
            "description": "desc"
        },
        "links": []
    }
}
"#
                        .to_string(),
                    )
                    .await;

                pretty_assertions::assert_eq!(
                    cache.drain(),
                    vec![
                        "INSERT INTO \"Todo\" (\"title\", \"description\", \"done\") VALUES ($1, $2, $3) RETURNING \"id\", \"title\", \"description\", \"done\";".to_string(),
                    ]
                );

                client
                    .exec(
                        r#"
{
    "op": "update_one",
    "body": {
        "base": "todo",
        "id": 1,
        "data": { "description": "updated_desc" },
        "links": [
            { "ty": "set_new", "to": "category", "value": { "title": "new_cat" } }
        ]
    }
}
"#
                        .to_string(),
                    )
                    .await;

                pretty_assertions::assert_eq!(
                    cache.drain(),
                    vec![
                        "INSERT INTO \"Category\" (\"title\") VALUES ($1) RETURNING \"id\", \"title\";".to_string(),
                        "UPDATE \"Todo\" SET \"description\" = $1, \"fk_category_def\" = $2 WHERE \"Todo\".\"id\" = $3 RETURNING \"id\", \"title\", \"description\", \"done\", \"fk_category_def\";".to_string(),
                    ]
                );
            })
            .await;

            watch_sqlx_calls(async |scope, cache| {
                let pool = Sqlite::in_memory_pool().await;
                let (client, ex) = Client::new_sqlx_db(pool);
                let client = client.into_string_client();

                scope.spawn(ex.run());

                setup_todo_with_category_link(&client, &cache).await;

                client
                    .exec(
                        r#"
{
    "op": "insert_one",
    "body": {
        "base": "category",
        "data": { "title": "cat_to_null" },
        "links": []
    }
}
"#
                        .to_string(),
                    )
                    .await;

                pretty_assertions::assert_eq!(
                    cache.drain(),
                    vec![
                        "INSERT INTO \"Category\" (\"title\") VALUES ($1) RETURNING \"id\", \"title\";".to_string(),
                    ]
                );

                client
                    .exec(
                        r#"
{
    "op": "insert_one",
    "body": {
        "base": "todo",
        "data": {
            "title": "todo_to_null",
            "done": false,
            "description": "desc"
        },
        "links": [
            { "ty": "set_id", "to": "category", "id": 1 }
        ]
    }
}
"#
                        .to_string(),
                    )
                    .await;

                pretty_assertions::assert_eq!(
                    cache.drain(),
                    vec![
                        "INSERT INTO \"Todo\" (\"title\", \"description\", \"done\", \"fk_category_def\") VALUES ($1, $2, $3, $4) RETURNING \"id\", \"title\", \"description\", \"done\", \"fk_category_def\";".to_string(),
                        "SELECT \"Category\".\"id\" AS \"iid\", \"Category\".\"title\" AS \"btitle\" FROM \"Category\" WHERE \"id\" = $1;".to_string(),
                    ]
                );

                client
                    .exec(
                        r#"
{
    "op": "update_one",
    "body": {
        "base": "todo",
        "id": 1,
        "data": {},
        "links": [
            { "ty": "set_null", "to": "category" }
        ]
    }
}
"#
                        .to_string(),
                    )
                    .await;

                pretty_assertions::assert_eq!(
                    cache.drain(),
                    vec![
                        "UPDATE \"Todo\" SET \"fk_category_def\" =  NULL WHERE \"Todo\".\"id\" = $1 RETURNING \"id\", \"title\", \"description\", \"done\", \"fk_category_def\";".to_string(),
                    ]
                );
            })
            .await;
        }
    }

    mod delete_one {
        use sqlx::Sqlite;

        use crate::{
            connect_in_memory::ConnectInMemory, json_client::client_interface::Client,
            track_sqlx_query::watch_sqlx_calls,
        };

        use crate::json_client::test_utilities::{
            add_category_collection, add_todo_collection, setup_todo_with_category_link,
            todo_is_one_to_many_with_category,
        };

        #[tokio::test(flavor = "current_thread")]
        async fn deletes_todo_without_links() {
            let pool = Sqlite::in_memory_pool().await;
            let (client, ex) = Client::new_sqlx_db(pool);
            let client = client.into_string_client();
            let _executor = tokio::spawn(ex.run());

            add_todo_collection(&client).await;

            client
                .exec(
                    r#"
{
    "op": "insert_one",
    "body": {
        "base": "todo",
        "data": {
            "title": "todo_to_delete",
            "done": false,
            "description": "gone"
        },
        "links": []
    }
}
"#
                    .to_string(),
                )
                .await;

            let result = client
                .exec(
                    r#"
{
    "op": "delete_one",
    "body": {
        "base": "todo",
        "id": 1,
        "links": []
    }
}
"#
                    .to_string(),
                )
                .await;

            pretty_assertions::assert_eq!(
                result,
                r#"{"output":{"id":1,"attributes":{"description":"gone","done":false,"title":"todo_to_delete"},"links":[]}}"#
            );
        }

        #[tokio::test(flavor = "current_thread")]
        async fn optional_to_many_returns_category_fk() {
            let pool = Sqlite::in_memory_pool().await;
            let (client, ex) = Client::new_sqlx_db(pool);
            let client = client.into_string_client();
            let _executor = tokio::spawn(ex.run());

            add_todo_collection(&client).await;
            add_category_collection(&client).await;
            todo_is_one_to_many_with_category(&client).await;

            client
                .exec(
                    r#"
{
    "op": "insert_one",
    "body": {
        "base": "category",
        "data": { "title": "cat_for_delete" },
        "links": []
    }
}
"#
                    .to_string(),
                )
                .await;

            client
                .exec(
                    r#"
{
    "op": "insert_one",
    "body": {
        "base": "todo",
        "data": {
            "title": "linked_todo",
            "done": true,
            "description": "with_cat"
        },
        "links": [
            { "ty": "set_id", "to": "category", "id": 1 }
        ]
    }
}
"#
                    .to_string(),
                )
                .await;

            let result = client
                .exec(
                    r#"
{
    "op": "delete_one",
    "body": {
        "base": "todo",
        "id": 1,
        "links": [
            { "ty": "optional_to_many", "to": "category" }
        ]
    }
}
"#
                    .to_string(),
                )
                .await;

            pretty_assertions::assert_eq!(
                result,
                r#"{"output":{"id":1,"attributes":{"description":"with_cat","done":true,"title":"linked_todo"},"links":[1]}}"#
            );
        }

        #[tokio::test(flavor = "current_thread")]
        async fn link_sql() {
            watch_sqlx_calls(async |scope, cache| {
                let pool = Sqlite::in_memory_pool().await;
                let (client, ex) = Client::new_sqlx_db(pool);
                let client = client.into_string_client();

                scope.spawn(ex.run());

                setup_todo_with_category_link(&client, &cache).await;

                client
                    .exec(
                        r#"
{
    "op": "insert_one",
    "body": {
        "base": "category",
        "data": { "title": "cat_for_delete_sql" },
        "links": []
    }
}
"#
                        .to_string(),
                    )
                    .await;

                pretty_assertions::assert_eq!(
                    cache.drain(),
                    vec![
                        "INSERT INTO \"Category\" (\"title\") VALUES ($1) RETURNING \"id\", \"title\";".to_string(),
                    ]
                );

                client
                    .exec(
                        r#"
{
    "op": "insert_one",
    "body": {
        "base": "todo",
        "data": {
            "title": "todo_for_delete_sql",
            "done": false,
            "description": "desc"
        },
        "links": [
            { "ty": "set_id", "to": "category", "id": 1 }
        ]
    }
}
"#
                        .to_string(),
                    )
                    .await;

                pretty_assertions::assert_eq!(
                    cache.drain(),
                    vec![
                        "INSERT INTO \"Todo\" (\"title\", \"description\", \"done\", \"fk_category_def\") VALUES ($1, $2, $3, $4) RETURNING \"id\", \"title\", \"description\", \"done\", \"fk_category_def\";".to_string(),
                        "SELECT \"Category\".\"id\" AS \"iid\", \"Category\".\"title\" AS \"btitle\" FROM \"Category\" WHERE \"id\" = $1;".to_string(),
                    ]
                );

                client
                    .exec(
                        r#"
{
    "op": "delete_one",
    "body": {
        "base": "todo",
        "id": 1,
        "links": [
            { "ty": "optional_to_many", "to": "category" }
        ]
    }
}
"#
                        .to_string(),
                    )
                    .await;

                pretty_assertions::assert_eq!(
                    cache.drain(),
                    vec![
                        "DELETE FROM \"Todo\" WHERE \"Todo\".\"id\" = $1 RETURNING \"id\", \"title\", \"description\", \"done\", \"fk_category_def\";".to_string(),
                    ]
                );
            })
            .await;
        }
    }

    mod many_to_many {
        use sqlx::Sqlite;

        use crate::{connect_in_memory::ConnectInMemory, json_client::client_interface::Client};

        use crate::json_client::test_utilities::{
            add_tag_collection, add_todo_collection, todo_is_many_to_many_with_tag,
        };

        async fn setup_todo_tag_link(client: &crate::json_client::string_client::StringClient) {
            add_todo_collection(client).await;
            add_tag_collection(client).await;
            todo_is_many_to_many_with_tag(client).await;
        }

        #[tokio::test(flavor = "current_thread")]
        async fn insert_set_id_links_existing_tag() {
            let pool = Sqlite::in_memory_pool().await;
            let (client, ex) = Client::new_sqlx_db(pool);
            let client = client.into_string_client();
            let _executor = tokio::spawn(ex.run());

            setup_todo_tag_link(&client).await;

            client
                .exec(
                    r#"
{
    "op": "insert_one",
    "body": {
        "base": "tag",
        "data": { "title": "urgent" },
        "links": []
    }
}
"#
                    .to_string(),
                )
                .await;

            let result = client
                .exec(
                    r#"
{
    "op": "insert_one",
    "body": {
        "base": "todo",
        "data": {
            "title": "todo_with_tag",
            "done": true,
            "description": "linked"
        },
        "links": [
            { "ty": "set_id", "to": "tag", "id": 1 }
        ]
    }
}
"#
                    .to_string(),
                )
                .await;

            pretty_assertions::assert_eq!(
                result,
                r#"{"output":{"id":1,"attributes":{"description":"linked","done":true,"title":"todo_with_tag"},"links":[{"id":1,"attributes":{"title":"urgent"}}]}}"#
            );
        }

        #[tokio::test(flavor = "current_thread")]
        async fn fetch_one_returns_linked_tags() {
            let pool = Sqlite::in_memory_pool().await;
            let (client, ex) = Client::new_sqlx_db(pool);
            let client = client.into_string_client();
            let _executor = tokio::spawn(ex.run());

            setup_todo_tag_link(&client).await;

            client
                .exec(
                    r#"
{
    "op": "insert_one",
    "body": {
        "base": "tag",
        "data": { "title": "urgent" },
        "links": []
    }
}
"#
                    .to_string(),
                )
                .await;

            client
                .exec(
                    r#"
{
    "op": "insert_one",
    "body": {
        "base": "tag",
        "data": { "title": "home" },
        "links": []
    }
}
"#
                    .to_string(),
                )
                .await;

            client
                .exec(
                    r#"
{
    "op": "insert_one",
    "body": {
        "base": "todo",
        "data": {
            "title": "todo_a",
            "done": true,
            "description": "a"
        },
        "links": [
            { "ty": "set_id", "to": "tag", "id": 1 },
            { "ty": "set_id", "to": "tag", "id": 2 }
        ]
    }
}
"#
                    .to_string(),
                )
                .await;

            let result = client
                .exec(
                    r#"
{
    "op": "fetch_one",
    "body": {
        "base": "todo",
        "id": 1,
        "filters": [],
        "links": [
            { "ty": "many_to_many", "to": "tag" }
        ]
    }
}
"#
                    .to_string(),
                )
                .await;

            pretty_assertions::assert_eq!(
                result,
                r#"{"output":{"id":1,"attributes":{"description":"a","done":true,"title":"todo_a"},"links":[[{"id":1,"attributes":{"title":"urgent"}},{"id":2,"attributes":{"title":"home"}}]]}}"#
            );
        }

        #[tokio::test(flavor = "current_thread")]
        async fn fetch_many_returns_linked_tags() {
            let pool = Sqlite::in_memory_pool().await;
            let (client, ex) = Client::new_sqlx_db(pool);
            let client = client.into_string_client();
            let _executor = tokio::spawn(ex.run());

            setup_todo_tag_link(&client).await;

            client
                .exec(
                    r#"
{
    "op": "insert_one",
    "body": {
        "base": "tag",
        "data": { "title": "urgent" },
        "links": []
    }
}
"#
                    .to_string(),
                )
                .await;

            client
                .exec(
                    r#"
{
    "op": "insert_one",
    "body": {
        "base": "todo",
        "data": {
            "title": "todo_a",
            "done": true,
            "description": "a"
        },
        "links": [
            { "ty": "set_id", "to": "tag", "id": 1 }
        ]
    }
}
"#
                    .to_string(),
                )
                .await;

            let result = client
                .exec(
                    r#"
{
    "op": "fetch_many",
    "body": {
        "base": "todo",
        "filters": [],
        "links": [
            { "ty": "many_to_many", "to": "tag" }
        ],
        "pagination": { "limit": 10, "first_item": null, "order_by": [] }
    }
}
"#
                    .to_string(),
                )
                .await;

            pretty_assertions::assert_eq!(
                result,
                r#"{"output":{"items":[{"id":1,"attributes":{"description":"a","done":true,"title":"todo_a"},"links":[[{"id":1,"attributes":{"title":"urgent"}}]]}],"next_item":null}}"#
            );
        }

        #[tokio::test(flavor = "current_thread")]
        async fn update_set_id_adds_tag_link() {
            let pool = Sqlite::in_memory_pool().await;
            let (client, ex) = Client::new_sqlx_db(pool);
            let client = client.into_string_client();
            let _executor = tokio::spawn(ex.run());

            setup_todo_tag_link(&client).await;

            client
                .exec(
                    r#"
{
    "op": "insert_one",
    "body": {
        "base": "tag",
        "data": { "title": "urgent" },
        "links": []
    }
}
"#
                    .to_string(),
                )
                .await;

            client
                .exec(
                    r#"
{
    "op": "insert_one",
    "body": {
        "base": "todo",
        "data": {
            "title": "todo_to_update",
            "done": false,
            "description": "before"
        },
        "links": []
    }
}
"#
                    .to_string(),
                )
                .await;

            let result = client
                .exec(
                    r#"
{
    "op": "update_one",
    "body": {
        "base": "todo",
        "id": 1,
        "data": { "title": "linked_todo" },
        "links": [
            { "ty": "set_id", "to": "tag", "id": 1 }
        ]
    }
}
"#
                    .to_string(),
                )
                .await;

            pretty_assertions::assert_eq!(
                result,
                r#"{"output":{"id":1,"attributes":{"description":"before","done":false,"title":"linked_todo"},"links":[1]}}"#
            );
        }

        #[tokio::test(flavor = "current_thread")]
        async fn update_remove_id_removes_tag_link() {
            let pool = Sqlite::in_memory_pool().await;
            let (client, ex) = Client::new_sqlx_db(pool);
            let client = client.into_string_client();
            let _executor = tokio::spawn(ex.run());

            setup_todo_tag_link(&client).await;

            client
                .exec(
                    r#"
{
    "op": "insert_one",
    "body": {
        "base": "tag",
        "data": { "title": "urgent" },
        "links": []
    }
}
"#
                    .to_string(),
                )
                .await;

            client
                .exec(
                    r#"
{
    "op": "insert_one",
    "body": {
        "base": "tag",
        "data": { "title": "home" },
        "links": []
    }
}
"#
                    .to_string(),
                )
                .await;

            client
                .exec(
                    r#"
{
    "op": "insert_one",
    "body": {
        "base": "todo",
        "data": {
            "title": "todo_linked",
            "done": false,
            "description": "with_tags"
        },
        "links": [
            { "ty": "set_id", "to": "tag", "id": 1 },
            { "ty": "set_id", "to": "tag", "id": 2 }
        ]
    }
}
"#
                    .to_string(),
                )
                .await;

            let result = client
                .exec(
                    r#"
{
    "op": "update_one",
    "body": {
        "base": "todo",
        "id": 1,
        "data": {},
        "links": [
            { "ty": "remove_id", "to": "tag", "id": 1 }
        ]
    }
}
"#
                    .to_string(),
                )
                .await;

            pretty_assertions::assert_eq!(
                result,
                r#"{"output":{"id":1,"attributes":{"description":"with_tags","done":false,"title":"todo_linked"},"links":[1]}}"#
            );
        }

        #[tokio::test(flavor = "current_thread")]
        async fn delete_returns_linked_tag_ids() {
            let pool = Sqlite::in_memory_pool().await;
            let (client, ex) = Client::new_sqlx_db(pool);
            let client = client.into_string_client();
            let _executor = tokio::spawn(ex.run());

            setup_todo_tag_link(&client).await;

            client
                .exec(
                    r#"
{
    "op": "insert_one",
    "body": {
        "base": "tag",
        "data": { "title": "urgent" },
        "links": []
    }
}
"#
                    .to_string(),
                )
                .await;

            client
                .exec(
                    r#"
{
    "op": "insert_one",
    "body": {
        "base": "tag",
        "data": { "title": "home" },
        "links": []
    }
}
"#
                    .to_string(),
                )
                .await;

            client
                .exec(
                    r#"
{
    "op": "insert_one",
    "body": {
        "base": "todo",
        "data": {
            "title": "linked_todo",
            "done": true,
            "description": "with_tags"
        },
        "links": [
            { "ty": "set_id", "to": "tag", "id": 1 },
            { "ty": "set_id", "to": "tag", "id": 2 }
        ]
    }
}
"#
                    .to_string(),
                )
                .await;

            let result = client
                .exec(
                    r#"
{
    "op": "delete_one",
    "body": {
        "base": "todo",
        "id": 1,
        "links": [
            { "ty": "many_to_many", "to": "tag" }
        ]
    }
}
"#
                    .to_string(),
                )
                .await;

            pretty_assertions::assert_eq!(
                result,
                r#"{"output":{"id":1,"attributes":{"description":"with_tags","done":true,"title":"linked_todo"},"links":[[1,2]]}}"#
            );
        }
    }
}

#[cfg(test)]
mod test_utilities {
    use sqlx::{Pool, Sqlite};

    use crate::{
        connect_in_memory::ConnectInMemory,
        json_client::{client_interface::Client, string_client::StringClient},
        track_sqlx_query::{Cache, watch_sqlx_calls},
    };

    pub async fn setup_todo_collection(sc: &StringClient, cache: &Cache) {
        add_todo_collection(sc).await;
        cache.clear();
    }

    pub async fn setup_todo_with_category_link(sc: &StringClient, cache: &Cache) {
        add_todo_collection(sc).await;
        add_category_collection(sc).await;
        todo_is_one_to_many_with_category(sc).await;
        cache.clear();
    }

    pub async fn add_todo_collection(sc: &StringClient) {
        // add_collection
        sc.exec(
            r#"
        {
            "op": "add_collection",
            "body": {
                "name": "todo",
                "fields": [
                    { "name": "title", "type_info": "String", "is_optional": false }
                    { "name": "description", "type_info": "String", "is_optional": true }
                    { "name": "done", "type_info": "Boolean", "is_optional": false }
                ]
            }
        }
        "#
            .to_string(),
        )
        .await;
    }

    pub async fn add_category_collection(sc: &StringClient) {
        sc.exec(
            r#"
        {
            "op": "add_collection",
            "body": {
                "name": "category",
                "fields": [
                    { "name": "title", "type_info": "String", "is_optional": false }
                ]
            }
        }
        "#
            .to_string(),
        )
        .await;
    }

    pub async fn add_tag_collection(sc: &StringClient) {
        sc.exec(
            r#"
        {
            "op": "add_collection",
            "body": {
                "name": "tag",
                "fields": [
                    { "name": "title", "type_info": "String", "is_optional": false }
                ]
            }
        }
        "#
            .to_string(),
        )
        .await;
    }

    pub async fn todo_is_timestamped(sc: &StringClient) {
        sc.exec(
            r#"
        {
            "op": "add_link",
            "body": {
                "ty": "timestamp",
                "collection": "todo"
            }
        }
        "#
            .to_string(),
        )
        .await;
    }

    pub async fn category_is_timestamped(sc: &StringClient) {
        sc.exec(
            r#"
        {
            "op": "add_link",
            "body": {
                "ty": "timestamp",
                "collection": "category"
            }
        }
        "#
            .to_string(),
        )
        .await;
    }

    pub async fn todo_is_one_to_many_with_category(sc: &StringClient) {
        sc.exec(
            r#"
        {
            "op": "add_link",
            "body": {
                "ty": "optional_to_many",
                "from": "todo",
                "to": "category"
            }
        }
        "#
            .to_string(),
        )
        .await;
    }

    pub async fn todo_is_many_to_many_with_tag(sc: &StringClient) {
        sc.exec(
            r#"
        {
            "op": "add_link",
            "body": {
                "ty": "many_to_many",
                "from": "todo",
                "to": "tag"
            }
        }
        "#
            .to_string(),
        )
        .await;
    }

    /// use this utility function to make assertions between queries easier
    pub async fn clear_timestams(pool: Pool<Sqlite>) {
        sqlx::query(
            r#"
            UPDATE "Todo" SET "created_at" = "demo created_at", "updated_at" = "demo updated_at";
            "#,
        )
        .execute(&pool)
        .await;
    }

    #[tokio::test(flavor = "current_thread")]
    async fn test_todo_app() {
        watch_sqlx_calls(async |scope, cache| {
            let pool = Sqlite::in_memory_pool().await;
            let (client, ex) = Client::new_sqlx_db(pool);

            let client = client.into_string_client();

            scope.spawn(ex.run());

            add_todo_collection(&client).await;

            pretty_assertions::assert_eq!(
                cache.drain(),
                vec![
                    r#"PRAGMA foreign_keys = ON;"#.to_string(),
                    r#"CREATE TABLE "Todo" ("id" INTEGER PRIMARY KEY AUTOINCREMENT, "title" TEXT NOT NULL, "description" TEXT, "done" BOOLEAN NOT NULL);"#.to_string(),
                ]
            );

            add_category_collection(&client).await;
            // add_tag_collection(&client).await;

            pretty_assertions::assert_eq!(
                cache.drain(),
                vec![
                    r#"PRAGMA foreign_keys = ON;"#.to_string(),
                    r#"CREATE TABLE "Category" ("id" INTEGER PRIMARY KEY AUTOINCREMENT, "title" TEXT NOT NULL);"#.to_string(),
                ]
            );

            todo_is_one_to_many_with_category(&client).await;
            // todo_is_many_to_many_with_tag(&client).await;

            pretty_assertions::assert_eq!(
                cache.drain(),
                vec![
                    r#"ALTER TABLE "Todo" ADD COLUMN "fk_category_def" INTEGER  REFERENCES "Category"("id") ON DELETE SET NULL;"#.to_string(),
                ]
            );
        })
        .await;
    }
}
