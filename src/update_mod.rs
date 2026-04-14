/// used in update operation, similar to Option<T> but
/// different semantic (implies intent) and serde implementation
///
/// implies intent to set a value or keep it.
/// update::set(None) means to set a value to null even if it is possiblly non-null in database
///
/// there is no way around having different serde implementation from Option<T>
/// because Option<Option<T>> does not know
/// the difference between `{}` and `{val:null}` or might confuse
/// Some(None) with None
#[derive(Default, Debug, PartialEq, Eq, PartialOrd, Ord, Clone)]
#[allow(non_camel_case_types)]
pub enum Update<T> {
    #[default]
    Keep,
    Set(T),
}

#[cfg(feature = "serde")]
mod impl_deserialize {
    use std::{
        fmt::{self},
        marker::PhantomData,
    };

    use serde::{
        Deserialize, Deserializer, Serialize, Serializer,
        de::{self, Visitor},
        ser::SerializeTupleStruct,
    };

    use super::Update;

    impl<T: Serialize> Serialize for Update<T> {
        fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
        where
            S: Serializer,
        {
            let mut worker = serializer.serialize_tuple_struct("optiona_update", 2)?;

            match self {
                Update::Keep => {
                    worker.serialize_field("keep")?;
                }
                Update::Set(some) => {
                    worker.serialize_field("set")?;
                    worker.serialize_field(some)?;
                }
            };

            worker.end()
        }
    }

    impl<'de, T> Deserialize<'de> for Update<T>
    where
        T: Deserialize<'de>,
    {
        fn deserialize<D>(deser: D) -> Result<Self, D::Error>
        where
            D: Deserializer<'de>,
        {
            deser.deserialize_option(EntryPointVisitor(PhantomData))
        }
    }

    struct EntryPointVisitor<T>(PhantomData<T>);

    impl<'de, T> Visitor<'de> for EntryPointVisitor<T>
    where
        T: Deserialize<'de>,
    {
        type Value = Update<T>;

        fn expecting(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
            formatter.write_str("either ['set', T] or ['keep']")
        }

        #[inline]
        fn visit_unit<E>(self) -> Result<Self::Value, E>
        where
            E: de::Error,
        {
            Ok(Update::Keep)
        }

        #[inline]
        fn visit_none<E>(self) -> Result<Self::Value, E>
        where
            E: de::Error,
        {
            Ok(Update::Keep)
        }

        fn visit_some<D>(self, deser: D) -> Result<Self::Value, D::Error>
        where
            D: Deserializer<'de>,
        {
            HelperDeser::<Update<T>>::deserialize(deser).map(|e| return e.0)
        }

        fn __private_visit_untagged_option<D>(self, deserializer: D) -> Result<Self::Value, ()>
        where
            D: Deserializer<'de>,
        {
            match T::deserialize(deserializer) {
                Ok(ok) => Ok(Update::Set(ok)),
                Err(_err) => Err(()),
            }
        }
    }

    struct HelperDeser<T>(T);

    impl<'de, T: Deserialize<'de>> Deserialize<'de> for HelperDeser<Update<T>> {
        fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
        where
            D: Deserializer<'de>,
        {
            deserializer
                .deserialize_tuple_struct("update_option helper deser", 2, LastVisitor(PhantomData))
                .map(|e| HelperDeser(e))
        }
    }

    struct LastVisitor<T>(PhantomData<T>);

    impl<'de, T: Deserialize<'de>> Visitor<'de> for LastVisitor<T> {
        type Value = Update<T>;

        fn expecting(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
            formatter.write_str("either ['set', T] or ['keep'] ok")
        }
        fn visit_seq<A>(self, mut seq: A) -> Result<Self::Value, A::Error>
        where
            A: de::SeqAccess<'de>,
        {
            let first = match seq.next_element::<String>()? {
                Some(ok) => ok,
                None => return Err(de::Error::invalid_length(0, &"len should be exactly 2")),
            };

            if first == "keep" {
                match seq.size_hint() {
                    Some(0) => {}
                    Some(_) => return Err(de::Error::custom("invalid len")),
                    None => {}
                }

                return Ok(Update::Keep);
            }

            if first == "set" {
                match seq.size_hint() {
                    Some(1) => {}
                    Some(_) => return Err(de::Error::custom("invalid len")),
                    None => {}
                }

                return Ok(match seq.next_element::<T>()? {
                    Some(ok) => Update::Set(ok),
                    None => return Err(de::Error::invalid_length(1, &"len should be exactly 2")),
                });
            }

            return Err(de::Error::custom(
                "first element has to be either \"set\" or \"keep\"",
            ));
        }
    }
}
