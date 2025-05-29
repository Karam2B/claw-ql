/// used in update operation, similar to Option<T> but
/// different semantic (implies intent) and serde implementation
///
/// implies intent to set a value or keep it.
/// update::set(None) means to set a value to null
///
/// there is no way around having different implementation
/// for serde trait, because Option<Option<T>> does not know
/// the difference between `{}` and `{val:null}` or might confuse
/// Some(None) with None
#[derive(Debug, PartialEq, Eq, PartialOrd, Ord, Clone)]
#[allow(non_camel_case_types)]
pub enum update<T> {
    keep,
    set(T),
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

    use super::update;

    impl<T: Serialize> Serialize for update<T> {
        fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
        where
            S: Serializer,
        {
            let mut worker = serializer.serialize_tuple_struct("optiona_update", 2)?;

            match self {
                update::keep => {
                    worker.serialize_field("keep")?;
                }
                update::set(some) => {
                    worker.serialize_field("set")?;
                    worker.serialize_field(some)?;
                }
            };

            worker.end()
        }
    }

    impl<'de, T> Deserialize<'de> for update<T>
    where
        T: Deserialize<'de>,
    {
        fn deserialize<D>(deser: D) -> Result<Self, D::Error>
        where
            D: Deserializer<'de>,
        {
            deser.deserialize_option(OptionVisitor(PhantomData))
        }
    }

    struct OptionVisitor<T>(PhantomData<T>);

    impl<'de, T> Visitor<'de> for OptionVisitor<T>
    where
        T: Deserialize<'de>,
    {
        type Value = update<T>;

        fn expecting(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
            formatter.write_str("update_option")
        }

        #[inline]
        fn visit_unit<E>(self) -> Result<Self::Value, E>
        where
            E: de::Error,
        {
            Ok(update::keep)
        }

        #[inline]
        fn visit_none<E>(self) -> Result<Self::Value, E>
        where
            E: de::Error,
        {
            Ok(update::keep)
        }

        fn visit_some<D>(self, deser: D) -> Result<Self::Value, D::Error>
        where
            D: Deserializer<'de>,
        {
            SecondDeser::<update<T>>::deserialize(deser).map(|e| return e.0)
        }

        fn __private_visit_untagged_option<D>(self, deserializer: D) -> Result<Self::Value, ()>
        where
            D: Deserializer<'de>,
        {
            match T::deserialize(deserializer) {
                Ok(ok) => Ok(update::set(ok)),
                Err(_err) => Err(()),
            }
        }
    }

    struct SecondDeser<T>(T);
    struct SecondVisitor<T>(PhantomData<T>);

    impl<'de, T: Deserialize<'de>> Deserialize<'de> for SecondDeser<update<T>> {
        fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
        where
            D: Deserializer<'de>,
        {
            deserializer
                .deserialize_tuple_struct("update_option", 2, SecondVisitor(PhantomData))
                .map(|e| SecondDeser(e))
        }
    }

    impl<'de, T: Deserialize<'de>> Visitor<'de> for SecondVisitor<T> {
        type Value = update<T>;

        fn expecting(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
            formatter.write_str("update_option")
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

                return Ok(update::keep);
            }

            if first == "set" {
                match seq.size_hint() {
                    Some(1) => {}
                    Some(_) => return Err(de::Error::custom("invalid len")),
                    None => {}
                }

                return Ok(match seq.next_element::<T>()? {
                    Some(ok) => update::set(ok),
                    None => return Err(de::Error::invalid_length(1, &"len should be exactly 2")),
                });
            }

            return Err(de::Error::custom("has to be either \"set\" or \"keep\""));
        }
    }
}
