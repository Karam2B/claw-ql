/// serialization strategy
/// All I want to do in serialization is to get from
/// (unique identifier) to their perspective behaviors
///
/// ## first issue (access)
///
/// a nice quick way to do that is to have an access
/// to the crate of such behavior
///
/// Another way is to convert these behavior to web-assemblies
/// files, and have access to these files
///
/// ## second issue (unique idents)
///
/// how to ensure identifiers are unique.
///
/// I'm brainstorming few options, but I notice that in all
/// there is always some runtime check that might fail when
/// you someone want to create a non-unique identifier
use std::any::TypeId;

use serde::Serialize;
use serde_json::Value as JsonValue;

#[allow(unused)]
#[deny(unused_must_use)]
mod unique_idents {
    use core::fmt;
    use std::marker::PhantomData;

    use claw_ql::tuple_trait::BuildTuple;
    use serde::{
        Deserialize, Serialize,
        de::{self, Visitor},
        ser::SerializeSeq,
    };

    struct SerializableId<RootStrategy, Id> {
        root_strategy: RootStrategy,
        id: Id,
    }

    impl<R, I> Serialize for SerializableId<R, I>
    where
        R: UniqueStrategy,
        I: Serialize,
    {
        fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
        where
            S: serde::Serializer,
        {
            let mut map = serializer.serialize_seq(Some(2))?;

            map.serialize_element(R::static_str())?;
            map.serialize_element(&self.id)?;

            map.end()
        }
    }

    impl<'d, I> Deserialize<'d> for SerializableId<String, I>
    where
        I: Deserialize<'d>,
    {
        fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
        where
            D: serde::Deserializer<'d>,
        {
            struct MainVisitor<R, I>(PhantomData<(R, I)>);
            impl<'d, I> Visitor<'d> for MainVisitor<String, I>
            where
                I: Deserialize<'d>,
            {
                type Value = SerializableId<String, I>;

                fn expecting(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
                    formatter.write_str("a sequence of 2 elements")
                }
                fn visit_seq<A>(self, mut seq: A) -> Result<Self::Value, A::Error>
                where
                    A: serde::de::SeqAccess<'d>,
                {
                    Ok(SerializableId {
                        root_strategy: {
                            let s = seq
                                .next_element::<String>()?
                                .ok_or_else(|| de::Error::custom("missing root strategy"))?;

                            match s.as_str() {
                                "main_registery" | "in_memory_incremental_id" => {}
                                s => {
                                    return Err(de::Error::custom(format!(
                                        "invalid root strategy: {}",
                                        s
                                    )));
                                }
                            };

                            s
                        },
                        id: seq
                            .next_element::<I>()?
                            .ok_or_else(|| de::Error::custom("missing id"))?,
                    })
                }
            }

            deserializer.deserialize_seq(MainVisitor(PhantomData))
        }
    }

    trait UniqueStrategy: Sized {
        type Error;
        type UserInfo;
        fn static_str() -> &'static str;
        fn create_identifier(
            &self,
            info: Self::UserInfo,
        ) -> Result<SerializableId<Self, Self::UserInfo>, Self::Error>;
    }

    pub struct MainRegistery {}
    pub struct CrateName {
        crate_name: String,
    }

    impl UniqueStrategy for MainRegistery {
        type Error = ();
        type UserInfo = CrateName;
        fn static_str() -> &'static str {
            todo!()
        }

        fn create_identifier(
            &self,
            info: Self::UserInfo,
        ) -> Result<SerializableId<Self, Self::UserInfo>, Self::Error> {
            todo!()
        }
    }

    pub struct GitHost<Url> {
        url: Url,
    }
    pub trait ConsistentHost {}
    impl<Url: ConsistentHost> UniqueStrategy for GitHost<Url> {
        type Error = ();

        type UserInfo = ();

        fn static_str() -> &'static str {
            todo!()
        }

        fn create_identifier(
            &self,
            info: Self::UserInfo,
        ) -> Result<SerializableId<Self, Self::UserInfo>, Self::Error> {
            todo!()
        }
    }

    pub struct InMemory {}
    pub struct IncrementalId {}
    pub struct UsingCreatedAt {}

    pub trait AllowsNesting<N>: Clone {
        fn clone_next(next: &N) -> N;
    }
    impl<R, I> SerializableId<R, I>
    where
        I: BuildTuple,
        R: AllowsNesting<I>,
    {
        pub fn nest<N>(&self, id: N) -> SerializableId<R, I::Bigger<N>> {
            SerializableId {
                root_strategy: self.root_strategy.clone(),
                id: R::clone_next(&self.id).into_bigger(id),
            }
        }
    }
}

pub struct Issue {
    pub serialization_id: String,
    pub serialized_info: JsonValue,
    pub module_path: &'static str,
}

inventory::collect! { Issue }

pub struct IssueType {
    pub ident: &'static str,
    pub serialization_id: &'static str,
    pub type_id: TypeId,
}

inventory::collect! { IssueType }

#[derive(Serialize)]
pub struct Deprication {
    pub reason: Option<String>,
}

inventory::submit! {
    IssueType {
        ident: "::claw_ql::issue::Deprication",
        serialization_id: "Deprication",
        type_id: TypeId::of::<Deprication>()
    }
}
