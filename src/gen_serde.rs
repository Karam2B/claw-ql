#![allow(unused)]
use std::marker::PhantomData;

pub trait FromSerialized<'de, Format> {
    type Deserializer: Deserializer<'de>;
    fn start(self) -> Self::Deserializer;
    fn terminate(
        bar: Self::Deserializer,
    ) -> Result<(), <Self::Deserializer as Deserializer<'de>>::Err>;

    /// Deserialize using the default handler [`()`]. Only available when [`DeserializeSpec::Handler`] is [`()`].
    fn deserialize_and_terminate<T>(
        self,
    ) -> Result<T, <Self::Deserializer as Deserializer<'de>>::Err>
    where
        Self: Sized,
        T: Deserialize<'de, Self::Deserializer, Handler = ()>,
    {
        let mut s = self.start();
        let t = T::deserialize((), &mut s)?;
        Self::terminate(s)?;
        Ok(t)
    }

    /// Same as [`deserialize_and_terminate`] when `T::Handler` is [`()`] (ignores `format` / `into`).
    #[inline]
    fn deserialize_and_terminate_with<T>(
        self,
        _format: Format,
        into: PhantomData<T>,
    ) -> Result<T, <Self::Deserializer as Deserializer<'de>>::Err>
    where
        Self: Sized,
        T: Deserialize<'de, Self::Deserializer, Handler = ()>,
    {
        let _: PhantomData<T> = into;
        self.deserialize_and_terminate::<T>()
    }

    /// Deserialize `T` using an explicit **handler** (configuration). Use when [`DeserializeSpec::Handler`]
    /// is not [`()`], e.g. runtime-controlled field shapes.
    #[inline]
    fn deserialize_and_terminate_with_handler<T>(
        self,
        _format: Format,
        handler: T::Handler,
        into: PhantomData<T>,
    ) -> Result<T, <Self::Deserializer as Deserializer<'de>>::Err>
    where
        Self: Sized,
        T: Deserialize<'de, Self::Deserializer>,
    {
        let _: PhantomData<T> = into;
        let mut s = self.start();
        let t = T::deserialize(handler, &mut s)?;
        Self::terminate(s)?;
        Ok(t)
    }
}

pub trait Deserializer<'de>: Sized {
    type Err;
    type Format;
}

/// Declares **how** a type is deserialized *before* choosing a [`Deserializer`]: the [`Handler`]
/// type is chosen only by this trait’s impl author (the type owner). [`Deserialize`] for a concrete
/// `S` never introduces alternative handlers per format.
pub trait DeserializeSpec: Sized {
    type Handler;
}

/// Deserialize from `S` using a [`DeserializeSpec::Handler`] value.
pub trait Deserialize<'de, S: Deserializer<'de>>: DeserializeSpec + Sized {
    fn deserialize(handler: Self::Handler, serialized: &mut S) -> Result<Self, S::Err>;
}

/// Associated “lookup view” of a key, possibly borrowing from `Key` (see [`KnownKey::info`]).
pub trait KnownKeyInfo {
    type Info<'a>
    where
        Self: 'a;
}

/// Map field lookup by a compile-time or runtime key type `Key`.
pub trait KnownKey<Key>: KnownKeyInfo {
    fn info<'k>(key: &'k Key) -> Self::Info<'k>;
}

pub trait UnknownKey<S> {}

pub trait DeserializeMap<'de>: Deserializer<'de> {
    type MapAccess;
    fn start_map(&mut self) -> Result<Self::MapAccess, Self::Err>;

    /// deserialize pairs with unknown key value -- the deserialize impl does not know the value of the key, only knows its type
    fn deserialize_pair_unknown_key<Key, Value>(
        &mut self,
        map: &mut Self::MapAccess,
        key_handler: Key::Handler,
        value_handler: Value::Handler,
    ) -> Result<(Key, Value), Self::Err>
    where
        Key: UnknownKey<Self> + DeserializeSpec + Deserialize<'de, Self>,
        Value: DeserializeSpec + Deserialize<'de, Self>;

    /// deserialize pairs with known key value -- the deserialize impl known the value of the key
    fn deserialize_pair_known_key<Key, Value>(
        &mut self,
        map: &mut Self::MapAccess,
        key: Key,
        value_handler: Value::Handler,
    ) -> Result<Value, Self::Err>
    where
        Self: KnownKey<Key>,
        Value: DeserializeSpec + Deserialize<'de, Self>;

    fn finish(&mut self, map: Self::MapAccess) -> Result<(), Self::Err>;
}

pub trait DeserializeSeq<'de>: Deserializer<'de> {
    type SeqAccess;
    fn start_seq(&mut self) -> Result<Self::SeqAccess, Self::Err>;
    fn deserialize_value<T>(
        &mut self,
        seq: &mut Self::SeqAccess,
        handler: T::Handler,
    ) -> Result<T, Self::Err>
    where
        T: DeserializeSpec + Deserialize<'de, Self>;
    fn finish(&mut self, seq: Self::SeqAccess) -> Result<(), Self::Err>;
}

pub enum MapOrSeq<Map, Seq> {
    Map(Map),
    Seq(Seq),
}

pub trait DeserializeMapOrSeq<'de>: DeserializeMap<'de> {
    type SeqAccess;
    fn start_map_or_seq(&mut self)
    -> Result<MapOrSeq<Self::MapAccess, Self::SeqAccess>, Self::Err>;

    fn deserialize_pair_unknown_key<Key, Value>(
        &mut self,
        map: &mut Self::MapAccess,
        key_handler: Key::Handler,
        value_handler: Value::Handler,
    ) -> Result<(Key, Value), Self::Err>
    where
        Key: UnknownKey<Self> + DeserializeSpec + Deserialize<'de, Self>,
        Value: DeserializeSpec + Deserialize<'de, Self>,
    {
        DeserializeMap::deserialize_pair_unknown_key(self, map, key_handler, value_handler)
    }

    fn deserialize_pair_known_key<Key, Value>(
        &mut self,
        map: &mut Self::MapAccess,
        key: Key,
        value_handler: Value::Handler,
    ) -> Result<Value, Self::Err>
    where
        Self: KnownKey<Key>,
        Value: DeserializeSpec + Deserialize<'de, Self>,
    {
        DeserializeMap::deserialize_pair_known_key(self, map, key, value_handler)
    }

    fn deserialize_value<T>(
        &mut self,
        seq: &mut Self::SeqAccess,
        handler: T::Handler,
    ) -> Result<T, Self::Err>
    where
        T: DeserializeSpec + Deserialize<'de, Self>;
    fn finish_map(&mut self, map: Self::MapAccess) -> Result<(), Self::Err>;
    fn finish_seq(&mut self, seq: Self::SeqAccess) -> Result<(), Self::Err>;
}

#[cfg(test)]
mod dynamic_client_side {
    use std::{marker::PhantomData, sync::Arc};

    use crate::gen_serde::{
        Deserialize, DeserializeMap, DeserializeMapOrSeq, DeserializeSpec, Deserializer,
        FromSerialized, KnownKey, MapOrSeq, json_format_side::JsonFormat,
    };

    #[derive(Debug, PartialEq, Eq)]
    pub enum StringOrInt {
        String(String),
        Int(i64),
    }

    /// Runtime configuration for [`DynamicExample`]: which JSON shape to expect for `another_field`.
    #[derive(Clone, Copy)]
    pub struct DynamicExampleHandler {
        pub another_field_is_string: bool,
    }

    #[derive(Debug, PartialEq, Eq)]
    pub struct DynamicExample {
        pub title: String,
        pub another_field: StringOrInt,
    }

    impl DeserializeSpec for DynamicExample {
        type Handler = DynamicExampleHandler;
    }

    impl<'de, S> Deserialize<'de, S> for DynamicExample
    where
        S: Deserializer<'de>,
        S: DeserializeMapOrSeq<'de>,
        String: Deserialize<'de, S>,
        i64: Deserialize<'de, S>,
        S: KnownKey<&'static str>,
        S::Err: From<&'static str>,
    {
        fn deserialize(handler: Self::Handler, serialized: &mut S) -> Result<Self, S::Err> {
            match serialized.start_map_or_seq()? {
                MapOrSeq::Map(mut map) => {
                    let title = DeserializeMap::deserialize_pair_known_key(
                        serialized,
                        &mut map,
                        "title",
                        (),
                    )?;
                    let another_field = if handler.another_field_is_string {
                        StringOrInt::String(DeserializeMap::deserialize_pair_known_key(
                            serialized,
                            &mut map,
                            "another_field",
                            (),
                        )?)
                    } else {
                        StringOrInt::Int(DeserializeMap::deserialize_pair_known_key(
                            serialized,
                            &mut map,
                            "another_field",
                            (),
                        )?)
                    };
                    DeserializeMapOrSeq::finish_map(serialized, map)?;
                    Ok(DynamicExample {
                        title,
                        another_field,
                    })
                }
                MapOrSeq::Seq(_) => Err(S::Err::from("expected JSON object for DynamicExample")),
            }
        }
    }

    #[test]
    fn te_dynamic_example_string_branch() {
        let s: Arc<str> = Arc::from(r#"{"title":"hi","another_field":"from_json"}"#.to_string());
        let got: DynamicExample = s
            .deserialize_and_terminate_with_handler(
                JsonFormat,
                DynamicExampleHandler {
                    another_field_is_string: true,
                },
                PhantomData::<DynamicExample>,
            )
            .unwrap();
        assert_eq!(
            got,
            DynamicExample {
                title: "hi".into(),
                another_field: StringOrInt::String("from_json".into()),
            }
        );
    }

    #[test]
    fn te_dynamic_example_int_branch() {
        let s: Arc<str> = Arc::from(r#"{"title":"hi","another_field":42}"#.to_string());
        let got: DynamicExample = s
            .deserialize_and_terminate_with_handler(
                JsonFormat,
                DynamicExampleHandler {
                    another_field_is_string: false,
                },
                PhantomData::<DynamicExample>,
            )
            .unwrap();
        assert_eq!(
            got,
            DynamicExample {
                title: "hi".into(),
                another_field: StringOrInt::Int(42),
            }
        );
    }
}

#[cfg(test)]
mod client_side {
    use crate::gen_serde::{
        Deserialize, DeserializeMap, DeserializeMapOrSeq, DeserializeSpec, Deserializer,
        FromSerialized, KnownKey, MapOrSeq,
        json_format_side::{JsonFormat, PartialDeserialize, StringArcRef},
    };
    use std::{marker::PhantomData, sync::Arc};

    #[derive(Debug, PartialEq, Eq, PartialOrd, Ord)]
    pub struct BasicObject {
        pub title: String,
    }

    impl DeserializeSpec for BasicObject {
        type Handler = ();
    }

    impl<'de, S> Deserialize<'de, S> for BasicObject
    where
        S: Deserializer<'de>,
        S: DeserializeMapOrSeq<'de>,
        String: Deserialize<'de, S>,
        S: KnownKey<&'static str>,
    {
        fn deserialize(_handler: Self::Handler, serialized: &mut S) -> Result<Self, S::Err> {
            match serialized.start_map_or_seq()? {
                MapOrSeq::Map(mut map) => {
                    let title = DeserializeMap::deserialize_pair_known_key(
                        serialized,
                        &mut map,
                        "title",
                        (),
                    )?;
                    DeserializeMapOrSeq::finish_map(serialized, map)?;
                    Ok(Self { title })
                }
                MapOrSeq::Seq(mut seq) => {
                    let title = serialized.deserialize_value::<String>(&mut seq, ())?;
                    DeserializeMapOrSeq::finish_seq(serialized, seq)?;
                    Ok(Self { title })
                }
            }
        }
    }

    #[test]
    fn te_object() {
        let s: Arc<str> = Arc::from(r#"{"title":"first_title"}"#.to_string());

        let s = s
            .deserialize_and_terminate_with(JsonFormat, PhantomData::<BasicObject>)
            .unwrap();

        pretty_assertions::assert_eq!(
            s,
            BasicObject {
                title: String::from("first_title")
            }
        );
    }

    #[test]
    fn te_array() {
        let s: Arc<str> = Arc::from(r#"["first_title"]"#.to_string());

        let s = s
            .deserialize_and_terminate_with(JsonFormat, PhantomData::<BasicObject>)
            .unwrap();

        pretty_assertions::assert_eq!(
            s,
            BasicObject {
                title: String::from("first_title")
            }
        );
    }

    #[test]
    fn te_i64_json() {
        let s: Arc<str> = Arc::from(" 42 ".to_string());
        let v: i64 = s
            .deserialize_and_terminate_with(JsonFormat, PhantomData::<i64>)
            .unwrap();
        assert_eq!(v, 42);
    }

    #[test]
    fn te_f64_json() {
        let s: Arc<str> = Arc::from("1.25e2".to_string());
        let v: f64 = s
            .deserialize_and_terminate_with(JsonFormat, PhantomData::<f64>)
            .unwrap();
        assert!((v - 125.0).abs() < 1e-9);
    }

    #[test]
    fn te_unit_null() {
        let s: Arc<str> = Arc::from("  null  ".to_string());
        s.deserialize_and_terminate_with(JsonFormat, PhantomData::<()>)
            .unwrap();
    }

    #[test]
    fn te_option_string() {
        let none: Arc<str> = Arc::from("null".to_string());
        let v: Option<String> = none
            .deserialize_and_terminate_with(JsonFormat, PhantomData::<Option<String>>)
            .unwrap();
        assert_eq!(v, None);

        let some: Arc<str> = Arc::from("\"x\"".to_string());
        let v: Option<String> = some
            .deserialize_and_terminate_with(JsonFormat, PhantomData::<Option<String>>)
            .unwrap();
        assert_eq!(v.as_deref(), Some("x"));
    }

    #[test]
    fn te_string_arc_ref_zero_copy() {
        let inner: Arc<str> = Arc::from(r#"  "plain"  "#.to_string());
        let doc = inner.clone();
        let got: StringArcRef = doc
            .deserialize_and_terminate_with(JsonFormat, PhantomData::<StringArcRef>)
            .unwrap();
        assert_eq!(got.as_str(), "plain");
        assert!(Arc::ptr_eq(&inner, &got.0));
        assert_eq!(got.1, 3..8);
    }

    #[test]
    fn te_string_arc_ref_escaped() {
        let inner: Arc<str> = Arc::from(r#""a\"b\\c""#.to_string());
        let doc = inner.clone();
        let got: StringArcRef = doc
            .deserialize_and_terminate_with(JsonFormat, PhantomData::<StringArcRef>)
            .unwrap();
        assert_eq!(got.as_str(), r#"a"b\c"#);
        assert!(!Arc::ptr_eq(&inner, &got.0));
        assert_eq!(got.1, 0..got.as_str().len());
    }

    #[derive(Debug, PartialEq, Eq)]
    pub struct Collection {
        pub identifier: String,
        pub data: PartialDeserialize,
    }

    impl DeserializeSpec for Collection {
        type Handler = ();
    }

    impl<'de, S> Deserialize<'de, S> for Collection
    where
        S: Deserializer<'de>,
        S: DeserializeMapOrSeq<'de>,
        String: Deserialize<'de, S>,
        PartialDeserialize: Deserialize<'de, S>,
        S: KnownKey<&'static str>,
        S::Err: From<&'static str>,
    {
        fn deserialize(_handler: Self::Handler, serialized: &mut S) -> Result<Self, S::Err> {
            match serialized.start_map_or_seq()? {
                MapOrSeq::Map(mut map) => {
                    let identifier = DeserializeMap::deserialize_pair_known_key(
                        serialized,
                        &mut map,
                        "identifier",
                        (),
                    )?;
                    let data = DeserializeMap::deserialize_pair_known_key(
                        serialized,
                        &mut map,
                        "data",
                        (),
                    )?;
                    DeserializeMapOrSeq::finish_map(serialized, map)?;
                    Ok(Collection { identifier, data })
                }
                MapOrSeq::Seq(_) => Err(S::Err::from("expected JSON object for Collection")),
            }
        }
    }

    #[test]
    fn partial_des() {
        let s: Arc<str> =
            Arc::from(r#"{"identifier":"todo", "data": {"title": "from_string_key"}}"#.to_string());

        let got: Collection = s
            .deserialize_and_terminate_with(JsonFormat, PhantomData::<Collection>)
            .unwrap();

        assert_eq!(got.identifier, "todo");
        assert_eq!(got.data.fragment(), r#"{"title": "from_string_key"}"#);
        let data: BasicObject = got.data.continue_deserialize().unwrap();
        assert_eq!(
            data,
            BasicObject {
                title: "from_string_key".to_string()
            }
        );
    }

    /// Same shape as [`DeTodo`], but uses a runtime [`String`] key with [`KnownKey<String>`].
    #[derive(Debug, PartialEq, Eq, PartialOrd, Ord)]
    pub struct DeTodoStringKey {
        pub title: String,
    }

    impl DeserializeSpec for DeTodoStringKey {
        type Handler = ();
    }

    impl<'de, S> Deserialize<'de, S> for DeTodoStringKey
    where
        S: Deserializer<'de>,
        S: DeserializeMapOrSeq<'de>,
        String: Deserialize<'de, S>,
        S: KnownKey<String>,
    {
        fn deserialize(_handler: Self::Handler, serialized: &mut S) -> Result<Self, S::Err> {
            match serialized.start_map_or_seq()? {
                MapOrSeq::Map(mut map) => {
                    let title = DeserializeMap::deserialize_pair_known_key(
                        serialized,
                        &mut map,
                        String::from("title"),
                        (),
                    )?;
                    DeserializeMapOrSeq::finish_map(serialized, map)?;
                    Ok(Self { title })
                }
                MapOrSeq::Seq(mut seq) => {
                    let title = serialized.deserialize_value::<String>(&mut seq, ())?;
                    DeserializeMapOrSeq::finish_seq(serialized, seq)?;
                    Ok(Self { title })
                }
            }
        }
    }

    #[test]
    fn te_object_known_key_string() {
        let s: Arc<str> = Arc::from(r#"{"title":"from_string_key"}"#.to_string());
        let got: DeTodoStringKey = s
            .deserialize_and_terminate_with(JsonFormat, PhantomData::<DeTodoStringKey>)
            .unwrap();
        assert_eq!(
            got,
            DeTodoStringKey {
                title: "from_string_key".into(),
            }
        );
    }
}

mod json_format_side {
    use std::marker::PhantomData;
    use std::ops::Range;
    use std::sync::Arc;

    use crate::gen_serde::{
        Deserialize, DeserializeMap, DeserializeMapOrSeq, DeserializeSeq, DeserializeSpec,
        Deserializer, FromSerialized, KnownKey, KnownKeyInfo, MapOrSeq, UnknownKey,
    };

    pub struct JsonAsArcCursor {
        pub inner: Arc<str>,
        pub start: usize,
    }
    pub struct JsonFormat;

    impl<'de> FromSerialized<'de, JsonFormat> for Arc<str> {
        type Deserializer = JsonAsArcCursor;

        fn start(self) -> Self::Deserializer {
            JsonAsArcCursor {
                inner: self,
                start: 0,
            }
        }

        fn terminate(
            bar: Self::Deserializer,
        ) -> Result<(), <Self::Deserializer as Deserializer<'de>>::Err> {
            let mut chars = bar.inner[bar.start..].chars().into_iter();

            while let Some(next) = chars.next() {
                if next.is_whitespace() {
                    continue;
                } else {
                    return Err(format!("expected whitespace found: {:?}", next));
                }
            }

            Ok(())
        }
    }

    impl<'de> Deserializer<'de> for JsonAsArcCursor {
        type Err = String;
        type Format = JsonFormat;
    }

    impl DeserializeSpec for String {
        type Handler = ();
    }

    impl<'de> Deserialize<'de, JsonAsArcCursor> for String {
        fn deserialize(
            _handler: Self::Handler,
            serialized: &mut JsonAsArcCursor,
        ) -> Result<Self, <JsonAsArcCursor as Deserializer<'de>>::Err> {
            let mut chars = serialized.inner[serialized.start..].chars().into_iter();

            let mut output = String::new();

            let mut peek = chars.next();
            while let Some(next) = peek {
                if next.is_whitespace() {
                    serialized.start += next.len_utf8();
                    peek = chars.next();
                    continue;
                }
                if next == '"' {
                    serialized.start += next.len_utf8();
                    peek = chars.next();
                    break;
                }
                return Err(format!("expected string found: {:?}", next));
            }

            while let Some(next) = peek {
                if next == '"' {
                    serialized.start += next.len_utf8();
                    break;
                }
                if next == '\\' {
                    let adv = match chars.next() {
                        Some(e) if e == '"' || e == '\\' => {
                            output.push(e);
                            '\\'.len_utf8() + e.len_utf8()
                        }
                        Some(n) => return Err(format!("unused escape at {:?}", n))?,
                        None => return Err("terminated at escape")?,
                    };
                    serialized.start += adv;
                } else {
                    output.push(next);
                    serialized.start += next.len_utf8();
                }
                peek = chars.next();
            }

            Ok(output)
        }
    }

    impl DeserializeSpec for bool {
        type Handler = ();
    }

    impl<'de> Deserialize<'de, JsonAsArcCursor> for bool {
        fn deserialize(
            _handler: Self::Handler,
            serialized: &mut JsonAsArcCursor,
        ) -> Result<Self, <JsonAsArcCursor as Deserializer<'de>>::Err> {
            let mut chars = serialized.inner[serialized.start..].chars().into_iter();

            while let Some(next) = chars.next() {
                serialized.start += 1;
                if !next.is_whitespace() {
                    break;
                }
            }

            match chars.next() {
                Some('t') => {
                    if chars.next() != Some('r') {
                        return Err("invalid char")?;
                    }
                    if chars.next() != Some('u') {
                        return Err("invalid char")?;
                    }
                    if chars.next() != Some('e') {
                        return Err("invalid char")?;
                    }

                    serialized.start += 4;
                    return Ok(true);
                }
                Some('f') => {
                    if chars.next() != Some('a') {
                        return Err("invalid char")?;
                    }
                    if chars.next() != Some('l') {
                        return Err("invalid char")?;
                    }
                    if chars.next() != Some('s') {
                        return Err("invalid char")?;
                    }
                    if chars.next() != Some('e') {
                        return Err("invalid char")?;
                    }
                    serialized.start += 5;
                    return Ok(false);
                }
                _ => return Err(format!("expected bool found: {:?}", chars.next())),
            }
        }
    }

    macro_rules! impl_deserialize_json_int_for {
        ($($t:ty),* $(,)?) => {$(
            impl DeserializeSpec for $t {
                type Handler = ();
            }

            impl<'de> Deserialize<'de, JsonAsArcCursor> for $t {
                fn deserialize(
                    _handler: Self::Handler,
                    serialized: &mut JsonAsArcCursor,
                ) -> Result<Self, <JsonAsArcCursor as Deserializer<'de>>::Err> {
                    deserialize_json_number_str(serialized)?
                        .parse::<$t>()
                        .map_err(|e: std::num::ParseIntError| e.to_string())
                }
            }
        )*};
    }

    macro_rules! impl_deserialize_json_float_for {
        ($($t:ty),* $(,)?) => {$(
            impl DeserializeSpec for $t {
                type Handler = ();
            }

            impl<'de> Deserialize<'de, JsonAsArcCursor> for $t {
                fn deserialize(
                    _handler: Self::Handler,
                    serialized: &mut JsonAsArcCursor,
                ) -> Result<Self, <JsonAsArcCursor as Deserializer<'de>>::Err> {
                    deserialize_json_number_str(serialized)?
                        .parse::<$t>()
                        .map_err(|e: std::num::ParseFloatError| e.to_string())
                }
            }
        )*};
    }

    impl_deserialize_json_int_for!(
        i8, i16, i32, i64, i128, isize, u8, u16, u32, u64, u128, usize
    );
    impl_deserialize_json_float_for!(f32, f64);

    impl DeserializeSpec for () {
        type Handler = ();
    }

    impl<'de> Deserialize<'de, JsonAsArcCursor> for () {
        fn deserialize(
            _handler: Self::Handler,
            serialized: &mut JsonAsArcCursor,
        ) -> Result<Self, <JsonAsArcCursor as Deserializer<'de>>::Err> {
            let s = serialized.inner.as_ref();
            let i = skip_ws_json(s, serialized.start);
            if s.len() < i + 4 || &s[i..i + 4] != "null" {
                return Err(format!(
                    "expected null for unit type, found {:?}",
                    s.get(i..s.len().min(i + 8))
                ));
            }
            serialized.start = i + 4;
            Ok(())
        }
    }

    impl<T: DeserializeSpec> DeserializeSpec for Option<T> {
        type Handler = T::Handler;
    }

    impl<'de, T> Deserialize<'de, JsonAsArcCursor> for Option<T>
    where
        T: DeserializeSpec + Deserialize<'de, JsonAsArcCursor>,
    {
        fn deserialize(
            handler: Self::Handler,
            serialized: &mut JsonAsArcCursor,
        ) -> Result<Self, <JsonAsArcCursor as Deserializer<'de>>::Err> {
            let s = serialized.inner.as_ref();
            let i = skip_ws_json(s, serialized.start);
            if s.len() >= i + 4 && &s[i..i + 4] == "null" {
                serialized.start = i + 4;
                return Ok(None);
            }
            Ok(Some(T::deserialize(handler, serialized)?))
        }
    }

    /// Holds the full JSON buffer plus the byte range of one value that was not parsed yet.
    #[derive(Clone, PartialEq, Eq, Debug)]
    pub struct PartialDeserialize(pub Arc<str>, pub Range<usize>);

    impl PartialDeserialize {
        /// The raw JSON substring for this deferred value.
        pub fn fragment(&self) -> &str {
            self.0
                .get(self.1.clone())
                .expect("claw_ql_bug: partial range must stay in bounds")
        }

        /// Parse the captured slice as `T` using the same [`JsonFormat`] rules as the outer cursor.
        pub fn continue_deserialize<T>(&self) -> Result<T, String>
        where
            T: for<'de> Deserialize<'de, JsonAsArcCursor, Handler = ()>,
        {
            let slice: Arc<str> = Arc::from(self.fragment());
            FromSerialized::deserialize_and_terminate_with(slice, JsonFormat, PhantomData::<T>)
        }
    }

    /// Captures one complete JSON value as a [`Range`] in `inner`, advances the cursor past it.
    impl DeserializeSpec for PartialDeserialize {
        type Handler = ();
    }

    impl<'de> Deserialize<'de, JsonAsArcCursor> for PartialDeserialize {
        fn deserialize(
            _handler: Self::Handler,
            serialized: &mut JsonAsArcCursor,
        ) -> Result<Self, <JsonAsArcCursor as Deserializer<'de>>::Err> {
            let s = serialized.inner.as_ref();
            let start = skip_ws_json(s, serialized.start);
            let end = end_of_json_value(s, start)?;
            let range = start..end;
            serialized.start = end;
            Ok(PartialDeserialize(Arc::clone(&serialized.inner), range))
        }
    }

    /// JSON string decoded to UTF-8, stored as either a subslice of the original buffer (fast
    /// path when there are no `\\` escapes) or a dedicated [`Arc<str>`] after decoding.
    #[derive(Clone, Debug, PartialEq, Eq)]
    pub struct StringArcRef(pub Arc<str>, pub Range<usize>);

    impl StringArcRef {
        pub fn as_str(&self) -> &str {
            self.0
                .get(self.1.clone())
                .expect("claw_ql_bug: string arc range must stay in bounds")
        }
    }

    impl std::ops::Deref for StringArcRef {
        type Target = str;

        fn deref(&self) -> &Self::Target {
            self.as_str()
        }
    }

    impl AsRef<str> for StringArcRef {
        fn as_ref(&self) -> &str {
            self.as_str()
        }
    }

    impl DeserializeSpec for StringArcRef {
        type Handler = ();
    }

    impl<'de> Deserialize<'de, JsonAsArcCursor> for StringArcRef {
        fn deserialize(
            _handler: Self::Handler,
            serialized: &mut JsonAsArcCursor,
        ) -> Result<Self, <JsonAsArcCursor as Deserializer<'de>>::Err> {
            let (parsed, end) = parse_json_string_arc_ref(&serialized.inner, serialized.start)?;
            serialized.start = end;
            Ok(parsed)
        }
    }

    const OPEN_BRACE: char = '{';
    const CLOSE_BRACE: char = '}';
    const OPEN_BRACE_BYTE: u8 = OPEN_BRACE as u8;
    const CLOSE_BRACE_BYTE: u8 = CLOSE_BRACE as u8;

    /// Built in [`JsonAsArcCursor::start_map`]: maps JSON object keys to half-open byte ranges of each **value** in `inner`.
    pub struct JsonMapAccess {
        pub entries: Vec<(
            Arc<str>,
            Range<usize>, // key
            Range<usize>, // value
        )>,
        /// Byte offset in `inner` just after the closing `}` of this object.
        pub after_object: usize,
    }

    fn skip_ws_json(s: &str, mut i: usize) -> usize {
        let b = s.as_bytes();
        while i < b.len() {
            match b[i] {
                b' ' | b'\t' | b'\n' | b'\r' => i += 1,
                _ => break,
            }
        }
        i
    }

    /// `open_quote_at` must point at `"`. Returns decoded contents and exclusive end index.
    fn parse_json_string(s: &str, open_quote_at: usize) -> Result<(String, usize), String> {
        let b = s.as_bytes();
        if b.get(open_quote_at) != Some(&b'"') {
            return Err(format!("expected '\"' at byte {}", open_quote_at));
        }
        let mut i = open_quote_at + 1;
        let mut out = String::new();
        while i < b.len() {
            match b[i] {
                b'"' => return Ok((out, i + 1)),
                b'\\' => {
                    i += 1;
                    match b.get(i) {
                        Some(&ch) if ch == b'"' || ch == b'\\' => {
                            out.push(ch as char);
                            i += 1;
                        }
                        Some(_) => return Err(format!("unsupported escape at {}", i)),
                        None => return Err("terminated inside escape".into()),
                    }
                }
                _ => {
                    let ch = s[i..]
                        .chars()
                        .next()
                        .ok_or_else(|| "eof in string".to_string())?;
                    i += ch.len_utf8();
                    out.push(ch);
                }
            }
        }
        Err("unterminated string".into())
    }

    /// Like [`parse_json_string`], but returns a [`StringArcRef`] that borrows from `inner` when
    /// the JSON string contains no backslash escapes; otherwise returns decoded text in a new `Arc`.
    fn parse_json_string_arc_ref(
        inner: &Arc<str>,
        start: usize,
    ) -> Result<(StringArcRef, usize), String> {
        let s = inner.as_ref();
        let mut i = skip_ws_json(s, start);
        let b = s.as_bytes();
        if b.get(i) != Some(&b'"') {
            return Err(format!("expected string at byte {}", i));
        }
        let content_start = i + 1;
        i = content_start;
        let mut decoded: Option<String> = None;

        while i < b.len() {
            match b[i] {
                b'"' => {
                    let end_after = i + 1;
                    let arc_ref = match decoded {
                        None => StringArcRef(Arc::clone(inner), content_start..i),
                        Some(s_out) => {
                            let len = s_out.len();
                            StringArcRef(Arc::from(s_out), 0..len)
                        }
                    };
                    return Ok((arc_ref, end_after));
                }
                b'\\' => {
                    if decoded.is_none() {
                        decoded = Some(s[content_start..i].to_string());
                    }
                    let out = decoded.as_mut().expect("claw_ql_bug: set above");
                    i += 1;
                    match b.get(i) {
                        Some(&ch) if ch == b'"' || ch == b'\\' => {
                            out.push(ch as char);
                            i += 1;
                        }
                        Some(_) => return Err(format!("unsupported escape at {}", i)),
                        None => return Err("terminated inside escape".into()),
                    }
                }
                _ => {
                    let ch = s[i..]
                        .chars()
                        .next()
                        .ok_or_else(|| "eof in string".to_string())?;
                    let step = ch.len_utf8();
                    if let Some(out) = decoded.as_mut() {
                        out.push(ch);
                    }
                    i += step;
                }
            }
        }
        Err("unterminated string".into())
    }

    fn scan_number_end(s: &str, start: usize) -> usize {
        let b = s.as_bytes();
        let mut i = start;
        while i < b.len() {
            match b[i] {
                b'0'..=b'9' | b'-' | b'+' | b'.' | b'e' | b'E' => i += 1,
                _ => break,
            }
        }
        i
    }

    fn deserialize_json_number_str(serialized: &mut JsonAsArcCursor) -> Result<&str, String> {
        let s = serialized.inner.as_ref();
        let i0 = skip_ws_json(s, serialized.start);
        if i0 >= s.len() {
            return Err("unexpected end where number expected".into());
        }
        let b = s.as_bytes();
        match b[i0] {
            b'0'..=b'9' | b'-' | b'+' => {}
            _ => return Err(format!("expected number, found {:?}", b[i0] as char)),
        }
        let i1 = scan_number_end(s, i0);
        if i1 <= i0 {
            return Err("invalid number".into());
        }
        let slice = &s[i0..i1];
        serialized.start = i1;
        Ok(slice)
    }

    fn scan_balanced(s: &str, start: usize, open: u8, close: u8) -> Result<usize, String> {
        let b = s.as_bytes();
        if b.get(start) != Some(&open) {
            return Err(format!("expected {:?} at {}", open as char, start));
        }
        let mut depth = 1usize;
        let mut i = start + 1;
        while i < b.len() && depth > 0 {
            match b[i] {
                b'"' => {
                    let (_, end) = parse_json_string(s, i)?;
                    i = end;
                }
                o if o == open => {
                    depth += 1;
                    i += 1;
                }
                c if c == close => {
                    depth -= 1;
                    i += 1;
                }
                _ => i += 1,
            }
        }
        if depth != 0 {
            return Err("unbalanced brackets".into());
        }
        Ok(i)
    }

    fn end_of_json_value(s: &str, start: usize) -> Result<usize, String> {
        let start = skip_ws_json(s, start);
        let b = s.as_bytes();
        if start >= b.len() {
            return Err("unexpected end of input where value expected".into());
        }
        match b[start] {
            b'"' => parse_json_string(s, start).map(|(_, end)| end),
            x if x == OPEN_BRACE_BYTE => scan_balanced(s, start, OPEN_BRACE_BYTE, CLOSE_BRACE_BYTE),
            b'[' => scan_balanced(s, start, b'[', b']'),
            b't' => {
                if s.get(start..start + 4) == Some("true") {
                    Ok(start + 4)
                } else {
                    Err(format!("expected 'true' at {}", start))
                }
            }
            b'f' => {
                if s.get(start..start + 5) == Some("false") {
                    Ok(start + 5)
                } else {
                    Err(format!("expected 'false' at {}", start))
                }
            }
            b'n' => {
                if s.get(start..start + 4) == Some("null") {
                    Ok(start + 4)
                } else {
                    Err(format!("expected 'null' at {}", start))
                }
            }
            b'0'..=b'9' | b'-' | b'+' => Ok(scan_number_end(s, start)),
            _ => Err(format!(
                "unexpected byte at start of value: {:?}",
                b[start] as char
            )),
        }
    }

    fn parse_json_object_at(s: &str, start: usize) -> Result<(JsonMapAccess, usize), String> {
        let mut i = skip_ws_json(s, start);
        let b = s.as_bytes();
        if b.get(i) != Some(&OPEN_BRACE_BYTE) {
            return Err(format!("expected '{{' at byte {}", i));
        }
        i += 1;
        i = skip_ws_json(s, i);
        let mut entries = Vec::new();
        if b.get(i) == Some(&CLOSE_BRACE_BYTE) {
            let after_object = i + 1;
            return Ok((
                JsonMapAccess {
                    entries,
                    after_object,
                },
                after_object,
            ));
        }
        loop {
            if b.get(i) != Some(&b'"') {
                return Err(format!("expected '\"' for object key at {}", i));
            }
            let key_open = i;
            let (key_decoded, j) = parse_json_string(s, i)?;
            let key_range = key_open..j;
            i = skip_ws_json(s, j);
            if b.get(i) != Some(&b':') {
                return Err(format!("expected ':' after object key at {}", i));
            }
            i += 1;
            i = skip_ws_json(s, i);
            let v_start = i;
            let v_end = end_of_json_value(s, v_start)?;
            entries.push((Arc::<str>::from(key_decoded), key_range, v_start..v_end));
            i = v_end;
            i = skip_ws_json(s, i);
            match b.get(i) {
                Some(b',') => {
                    i += 1;
                    i = skip_ws_json(s, i);
                }
                Some(&CLOSE_BRACE_BYTE) => {
                    let after_object = i + 1;
                    return Ok((
                        JsonMapAccess {
                            entries,
                            after_object,
                        },
                        after_object,
                    ));
                }
                _ => {
                    return Err(format!("expected comma or '}}' after value at byte {}", i));
                }
            }
        }
    }

    impl KnownKeyInfo for JsonAsArcCursor {
        type Info<'a> = &'a str;
    }

    impl KnownKey<&'static str> for JsonAsArcCursor {
        fn info<'k>(key: &'k &'static str) -> Self::Info<'k> {
            *key
        }
    }

    impl KnownKey<String> for JsonAsArcCursor {
        fn info<'k>(key: &'k String) -> Self::Info<'k> {
            key.as_str()
        }
    }

    impl UnknownKey<JsonAsArcCursor> for String {}

    impl<'de> DeserializeMap<'de> for JsonAsArcCursor {
        type MapAccess = JsonMapAccess;

        fn start_map(&mut self) -> Result<Self::MapAccess, Self::Err> {
            let s = self.inner.as_ref();
            let (map_access, _after_obj) = parse_json_object_at(s, self.start)?;
            Ok(map_access)
        }

        fn deserialize_pair_unknown_key<Key, Value>(
            &mut self,
            map: &mut Self::MapAccess,
            key_handler: Key::Handler,
            value_handler: Value::Handler,
        ) -> Result<(Key, Value), Self::Err>
        where
            Key: UnknownKey<Self> + DeserializeSpec + Deserialize<'de, Self>,
            Value: DeserializeSpec + Deserialize<'de, Self>,
        {
            if map.entries.is_empty() {
                return Err("no object entries left to deserialize".into());
            }
            let (_k_arc, key_range, value_range) = map.entries.swap_remove(0);
            self.start = key_range.start;
            let key = Key::deserialize(key_handler, self)?;
            if self.start != key_range.end {
                return Err(format!(
                    "key JSON did not parse to expected span (got {}, expected {})",
                    self.start, key_range.end
                ));
            }
            self.start = value_range.start;
            let value = Value::deserialize(value_handler, self)?;
            if self.start != value_range.end {
                return Err(format!(
                    "value JSON did not parse to expected span (got {}, expected {})",
                    self.start, value_range.end
                ));
            }
            Ok((key, value))
        }

        fn deserialize_pair_known_key<Key, Value>(
            &mut self,
            map: &mut Self::MapAccess,
            key: Key,
            value_handler: Value::Handler,
        ) -> Result<Value, Self::Err>
        where
            Self: KnownKey<Key>,
            Value: DeserializeSpec + Deserialize<'de, Self>,
        {
            let needle = <Self as KnownKey<Key>>::info(&key);
            let idx = map
                .entries
                .iter()
                .position(|(k, _, _)| k.as_ref() == needle)
                .ok_or_else(|| format!("missing key {:?}", needle))?;
            let (_k_arc, _key_range, value_range) = map.entries.swap_remove(idx);
            self.start = value_range.start;
            let value = Value::deserialize(value_handler, self)?;
            if self.start != value_range.end {
                return Err(format!(
                    "value for key {:?} did not parse to expected span (got end {}, expected {})",
                    needle, self.start, value_range.end
                ));
            }
            Ok(value)
        }

        fn finish(&mut self, map: Self::MapAccess) -> Result<(), Self::Err> {
            self.start = map.after_object;
            Ok(())
        }
    }

    const OPEN_BRACKET: char = '[';
    const CLOSE_BRACKET: char = ']';
    const OPEN_BRACKET_BYTE: u8 = OPEN_BRACKET as u8;

    impl<'de> DeserializeMapOrSeq<'de> for JsonAsArcCursor {
        type SeqAccess = ();

        fn start_map_or_seq(
            &mut self,
        ) -> Result<MapOrSeq<Self::MapAccess, Self::SeqAccess>, Self::Err> {
            let s = self.inner.as_ref();
            let i = skip_ws_json(s, self.start);
            match s.as_bytes().get(i) {
                Some(&OPEN_BRACE_BYTE) => {
                    self.start = i;
                    DeserializeMap::start_map(self).map(MapOrSeq::Map)
                }
                Some(ch) if *ch == OPEN_BRACKET_BYTE => {
                    self.start = i;
                    DeserializeSeq::start_seq(self).map(MapOrSeq::Seq)
                }
                Some(b) => Err(format!(
                    "expected JSON object or array, found byte {:?} at {}",
                    *b as char, i
                )),
                None => Err(format!(
                    "expected JSON object or array, found end of input at offset {}",
                    i
                )),
            }
        }

        fn deserialize_value<T>(
            &mut self,
            seq: &mut Self::SeqAccess,
            handler: T::Handler,
        ) -> Result<T, Self::Err>
        where
            T: DeserializeSpec + Deserialize<'de, Self>,
        {
            DeserializeSeq::deserialize_value(self, seq, handler)
        }

        fn finish_map(&mut self, map: Self::MapAccess) -> Result<(), Self::Err> {
            DeserializeMap::finish(self, map)
        }

        fn finish_seq(&mut self, seq: Self::SeqAccess) -> Result<(), Self::Err> {
            DeserializeSeq::finish(self, seq)
        }
    }

    impl<'de> DeserializeSeq<'de> for JsonAsArcCursor {
        type SeqAccess = ();

        fn start_seq(&mut self) -> Result<Self::SeqAccess, Self::Err> {
            let mut chars = self.inner[self.start..].chars().into_iter();

            let mut peek = chars.next();
            while let Some(next) = peek {
                self.start += 1;
                if !next.is_whitespace() {
                    break;
                }
                peek = chars.next();
            }

            match peek {
                Some(OPEN_BRACKET) => {
                    // Opening `[` was already counted in the loop above; do not advance again.
                }
                _ => {
                    return Err(format!("expected start of a sequence found {:?}", peek));
                }
            }
            Ok(())
        }

        fn deserialize_value<T>(
            &mut self,
            _: &mut Self::SeqAccess,
            handler: T::Handler,
        ) -> Result<T, Self::Err>
        where
            T: DeserializeSpec + Deserialize<'de, Self>,
        {
            let mut chars = self.inner[self.start..].chars().into_iter();

            while let Some(next) = chars.next() {
                if !next.is_whitespace() {
                    break;
                }
                // self.start += 1;
            }

            let value = T::deserialize(handler, self)?;
            // check for comma

            let mut chars = self.inner[self.start..].chars().into_iter();
            let mut inc = 0;
            while let Some(next) = chars.next() {
                inc += 1;
                if next == ',' {
                    self.start += inc;
                    break;
                }
                if next.is_whitespace() {
                    continue;
                }
                break; // without incrementing
            }

            Ok(value)
        }

        fn finish(&mut self, _: Self::SeqAccess) -> Result<(), Self::Err> {
            loop {
                let mut chars = self.inner[self.start..].chars();
                let Some(next) = chars.next() else {
                    return Err(format!("expected {} found end of input", CLOSE_BRACKET));
                };
                if next.is_whitespace() {
                    self.start += next.len_utf8();
                    continue;
                }
                if next == CLOSE_BRACKET {
                    self.start += next.len_utf8();
                    return Ok(());
                }
                return Err(format!("expected {} found {:?}", CLOSE_BRACKET, next));
            }
        }
    }
}
