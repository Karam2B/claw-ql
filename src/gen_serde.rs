#![allow(unused)]
use std::marker::PhantomData;

pub trait FromSerialized<'de, Format> {
    type Deserializer: Deserializer<'de>;
    fn start(self) -> Self::Deserializer;
    fn terminate(
        bar: Self::Deserializer,
    ) -> Result<(), <Self::Deserializer as Deserializer<'de>>::Err>;
    fn deserialize_and_terminate<T>(
        self,
    ) -> Result<T, <Self::Deserializer as Deserializer<'de>>::Err>
    where
        Self: Sized,
        T: Deserialize<'de, Self::Deserializer>,
    {
        let mut s = self.start();
        let t = T::deserialize(&mut s)?;
        Self::terminate(s)?;
        Ok(t)
    }

    #[inline]
    fn deserialize_and_terminate_with<T>(
        self,
        format: Format,
        into: PhantomData<T>,
    ) -> Result<T, <Self::Deserializer as Deserializer<'de>>::Err>
    where
        Self: Sized,
        T: Deserialize<'de, Self::Deserializer>,
    {
        self.deserialize_and_terminate::<T>()
    }
}

pub trait Deserializer<'de>: Sized {
    type Err;
    type Format;
}

pub trait Deserialize<'de, S: Deserializer<'de>>: Sized {
    fn deserialize(serialized: &mut S) -> Result<Self, S::Err>;
}

pub trait DeserializeKey {
    type Output;
}

impl DeserializeKey for usize {
    type Output = ();
}

impl DeserializeKey for &'static str {
    type Output = ();
}

impl<T> DeserializeKey for PhantomData<T> {
    type Output = T;
}

pub trait DeserializeKeyFor<'de, S: DeserializeMap<'de>>: DeserializeKey {
    fn deserialize_key(
        self,
        serialized: &mut S,
        map_acess: &mut S::MapAccess,
    ) -> Result<Self::Output, S::Err>;
}

pub trait DeserializeMap<'de>: Deserializer<'de> {
    type MapAccess;
    fn start_map(&mut self) -> Result<Self::MapAccess, Self::Err>;
    fn deserialize_pair<Key, Value>(
        &mut self,
        map: &mut Self::MapAccess,
        key_helper: Key,
    ) -> Result<(Key::Output, Value), Self::Err>
    where
        Key: DeserializeKeyFor<'de, Self>,
        Value: Deserialize<'de, Self>;
    fn finish(&mut self, map: Self::MapAccess) -> Result<(), Self::Err>;
}

pub trait DeserializeSeq<'de>: Deserializer<'de> {
    type SeqAccess;
    fn start_seq(&mut self) -> Result<Self::SeqAccess, Self::Err>;
    fn deserialize_value<T>(&mut self, seq: &mut Self::SeqAccess) -> Result<T, Self::Err>
    where
        T: Deserialize<'de, Self>;
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
    fn deserialize_pair<Key, Value>(
        &mut self,
        map: &mut Self::MapAccess,
        key_helper: Key,
    ) -> Result<(Key::Output, Value), Self::Err>
    where
        Key: DeserializeKeyFor<'de, Self>,
        Value: Deserialize<'de, Self>,
        Key::Output: Default;
    fn deserialize_value<T>(&mut self, seq: &mut Self::SeqAccess) -> Result<T, Self::Err>
    where
        T: Deserialize<'de, Self>;
    fn finish_map(&mut self, map: Self::MapAccess) -> Result<(), Self::Err>;
    fn finish_seq(&mut self, seq: Self::SeqAccess) -> Result<(), Self::Err>;
}

mod client_side {
    use crate::gen_serde::{
        Deserialize, DeserializeKeyFor, DeserializeMap, DeserializeMapOrSeq, Deserializer,
        FromSerialized, MapOrSeq, json_format_side::JsonFormat,
    };
    use std::{marker::PhantomData, sync::Arc};

    #[derive(Debug, PartialEq, Eq, PartialOrd, Ord)]
    pub struct DeTodo {
        pub title: String,
    }

    impl<'de, S> Deserialize<'de, S> for DeTodo
    where
        S: Deserializer<'de>,
        S: DeserializeMapOrSeq<'de>,
        String: Deserialize<'de, S>,
        &'static str: DeserializeKeyFor<'de, S>,
    {
        fn deserialize(serialized: &mut S) -> Result<Self, S::Err> {
            match serialized.start_map_or_seq()? {
                MapOrSeq::Map(mut map) => {
                    let (_, title) =
                        DeserializeMap::deserialize_pair(serialized, &mut map, "title")?;
                    DeserializeMapOrSeq::finish_map(serialized, map)?;
                    Ok(Self { title })
                }
                MapOrSeq::Seq(mut seq) => {
                    let title = serialized.deserialize_value::<String>(&mut seq)?;
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
            .deserialize_and_terminate_with(JsonFormat, PhantomData::<DeTodo>)
            .unwrap();

        pretty_assertions::assert_eq!(
            s,
            DeTodo {
                title: String::from("first_title")
            }
        );
    }

    #[test]
    fn te_array() {
        let s: Arc<str> = Arc::from(r#"["first_title"]"#.to_string());

        let s = s
            .deserialize_and_terminate_with(JsonFormat, PhantomData::<DeTodo>)
            .unwrap();

        pretty_assertions::assert_eq!(
            s,
            DeTodo {
                title: String::from("first_title")
            }
        );
    }
}

// mod partial_deserialize {
//     pub struct PartialArc {
//         pub inner: Arc<str>,
//         pub range: Range<usize>,
//     }
//     pub struct PartialDeserialize<T> {
//         pub key: String,
//         pub value: PartialArc,
//     }

//     #[test]
//     fn test() {
//         let first: Arc<str> =
//             Arc::from(r#"{"key":"todo","value":{"todo_title":"first_title"}}"#.to_string());
//         let second: Arc<str> =
//             Arc::from(r#"{"key":"todo","value":{"cat_title":"first_title"}}"#.to_string());

//         let first = first
//             .deserialize_and_terminate_with(JsonFormat, PhantomData::<PartialDeserialize<Todo>>)
//             .unwrap();

//         if first.key == "todo" {
//             let todo = first.value.continue_as::<Todo>().unwrap();

//             assert_eq!(todo.todo_title, "first_title");
//         } else if first.key == "category" {
//             first.value.continue_as::<Category>().unwrap();
//             assert_eq!(category.cat_title, "first_title");
//         }
//     }
// }

mod json_format_side {
    use std::ops::Range;
    use std::sync::Arc;

    use crate::gen_serde::{
        Deserialize, DeserializeKeyFor, DeserializeMap, DeserializeMapOrSeq, DeserializeSeq,
        Deserializer, FromSerialized, MapOrSeq,
    };

    pub struct ArcCursor {
        pub inner: Arc<str>,
        pub start: usize,
    }
    pub struct JsonFormat;

    impl<'de> FromSerialized<'de, JsonFormat> for Arc<str> {
        type Deserializer = ArcCursor;

        fn start(self) -> Self::Deserializer {
            ArcCursor {
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

    impl<'de> Deserializer<'de> for ArcCursor {
        type Err = String;
        type Format = JsonFormat;
    }

    impl<'de> Deserialize<'de, ArcCursor> for String {
        fn deserialize(
            serialized: &mut ArcCursor,
        ) -> Result<Self, <ArcCursor as Deserializer<'de>>::Err> {
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

    impl<'de> Deserialize<'de, ArcCursor> for bool {
        fn deserialize(
            serialized: &mut ArcCursor,
        ) -> Result<Self, <ArcCursor as Deserializer<'de>>::Err> {
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

    const OPEN_BRACE: char = '{';
    const CLOSE_BRACE: char = '}';
    const OPEN_BRACE_BYTE: u8 = OPEN_BRACE as u8;
    const CLOSE_BRACE_BYTE: u8 = CLOSE_BRACE as u8;

    /// Built in [`ArcCursor::start_map`]: maps JSON object keys to half-open byte ranges of each **value** in `inner`.
    pub struct JsonMapAccess {
        pub entries: Vec<(
            Arc<str>,
            Range<usize>, // key
            Range<usize>, // value
        )>,
        /// Set by [`DeserializeKeyFor::deserialize_key`] before the value is parsed.
        pub pending_value_end: Option<usize>,
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
            b'0'..=b'9' | b'-' => Ok(scan_number_end(s, start)),
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
                    pending_value_end: None,
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
                            pending_value_end: None,
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

    impl<'de> DeserializeMap<'de> for ArcCursor {
        type MapAccess = JsonMapAccess;

        fn start_map(&mut self) -> Result<Self::MapAccess, Self::Err> {
            let s = self.inner.as_ref();
            let (map_access, _after_obj) = parse_json_object_at(s, self.start)?;
            Ok(map_access)
        }

        fn deserialize_pair<Key, Value>(
            &mut self,
            map: &mut Self::MapAccess,
            key_helper: Key,
        ) -> Result<(Key::Output, Value), Self::Err>
        where
            Key: DeserializeKeyFor<'de, Self>,
            Value: Deserialize<'de, Self>,
        {
            let out_key = key_helper.deserialize_key(self, map)?;
            let expected_end = map.pending_value_end.take().ok_or_else(|| {
                "internal error: DeserializeKeyFor did not set pending_value_end".to_string()
            })?;
            let value = Value::deserialize(self)?;
            if self.start != expected_end {
                return Err(format!(
                    "value for key did not parse to expected span (got end {}, expected {})",
                    self.start, expected_end
                ));
            }
            Ok((out_key, value))
        }

        fn finish(&mut self, map: Self::MapAccess) -> Result<(), Self::Err> {
            self.start = map.after_object;
            Ok(())
        }
    }

    impl<'de> DeserializeKeyFor<'de, ArcCursor> for &'static str {
        fn deserialize_key(
            self,
            serialized: &mut ArcCursor,
            map_access: &mut JsonMapAccess,
        ) -> Result<Self::Output, String> {
            let idx = map_access
                .entries
                .iter()
                .position(|(k, _, _)| k.as_ref() == self)
                .ok_or_else(|| format!("missing key {:?}", self))?;
            let (_k, _key_range, value_range) = map_access.entries.swap_remove(idx);
            map_access.pending_value_end = Some(value_range.end);
            serialized.start = value_range.start;
            Ok(())
        }
    }

    const OPEN_BRACKET: char = '[';
    const CLOSE_BRACKET: char = ']';
    const OPEN_BRACKET_BYTE: u8 = OPEN_BRACKET as u8;

    impl<'de> DeserializeMapOrSeq<'de> for ArcCursor {
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

        fn deserialize_pair<Key, Value>(
            &mut self,
            map: &mut Self::MapAccess,
            key_helper: Key,
        ) -> Result<(Key::Output, Value), Self::Err>
        where
            Key: DeserializeKeyFor<'de, Self>,
            Value: Deserialize<'de, Self>,
            Key::Output: Default,
        {
            DeserializeMap::deserialize_pair(self, map, key_helper)
        }

        fn deserialize_value<T>(&mut self, seq: &mut Self::SeqAccess) -> Result<T, Self::Err>
        where
            T: Deserialize<'de, Self>,
        {
            DeserializeSeq::deserialize_value(self, seq)
        }

        fn finish_map(&mut self, map: Self::MapAccess) -> Result<(), Self::Err> {
            DeserializeMap::finish(self, map)
        }

        fn finish_seq(&mut self, seq: Self::SeqAccess) -> Result<(), Self::Err> {
            DeserializeSeq::finish(self, seq)
        }
    }

    impl<'de> DeserializeSeq<'de> for ArcCursor {
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

        fn deserialize_value<T>(&mut self, _: &mut Self::SeqAccess) -> Result<T, Self::Err>
        where
            T: Deserialize<'de, Self>,
        {
            let mut chars = self.inner[self.start..].chars().into_iter();

            while let Some(next) = chars.next() {
                if !next.is_whitespace() {
                    break;
                }
                // self.start += 1;
            }

            let value = T::deserialize(self)?;
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
