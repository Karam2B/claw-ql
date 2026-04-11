use std::collections::HashMap;

use serde::Serialize;
use serde_json::Number;

#[derive(Debug)]
pub enum Value {
    NoComp,
    Null,
    Bool(bool),
    Number(Number),
    String(String),
    Array(Vec<Value>),
    Object(HashMap<String, Value>),
}

impl PartialEq<serde_json::Value> for Value {
    fn eq(&self, other: &serde_json::Value) -> bool {
        match self {
            Value::NoComp => true,
            Value::Null => other.is_null(),
            Value::Bool(bool) => other.as_bool().map(|e| e == *bool) == Some(true),
            Value::Number(val) => other.as_number().map(|e| *e == *val) == Some(true),
            Value::String(val) => other.as_str().map(|e| e == *val) == Some(true),
            Value::Array(values) => {
                let mut other = if let Some(e) = other.as_array() {
                    e.iter()
                } else {
                    return false;
                };
                let mut this = values.iter();
                loop {
                    match (other.next(), this.next()) {
                        (Some(other), Some(this)) => {
                            if this == other {
                                continue;
                            }
                            return false;
                        }
                        (None, None) => return true,
                        _ => return false,
                    }
                }
            }
            Value::Object(hash_map) => {
                let other = if let Some(e) = other.as_object() {
                    e
                } else {
                    return false;
                };
                if hash_map.keys().count() != other.keys().count() {
                    return false;
                }
                for (key, value) in hash_map.iter() {
                    let found = if let Some(va) = other.get(key) {
                        va
                    } else {
                        return false;
                    };
                    if value == found {
                        continue;
                    } else {
                        return false;
                    }
                }
                true
            }
        }
    }
}

impl From<serde_json::Value> for Value {
    fn from(value: serde_json::Value) -> Self {
        match value {
            serde_json::Value::Null => Self::Null,
            serde_json::Value::Bool(b) => Self::Bool(b),
            serde_json::Value::Number(number) => Self::Number(number),
            serde_json::Value::String(str) => Self::String(str),
            serde_json::Value::Array(values) => {
                Self::Array(values.into_iter().map(|e| e.into()).collect())
            }
            serde_json::Value::Object(map) => Self::Object({
                let mut map_r = HashMap::<String, Value>::new();
                for (key, value) in map {
                    map_r.insert(key, value.into());
                }
                map_r
            }),
        }
    }
}

pub mod __private {
    pub use serde_json;
    pub use std::collections::HashMap as Map;
    pub use std::vec;
}

pub fn to_value<T: Serialize>(t: T) -> Result<Value, serde_json::Error> {
    let t = serde_json::to_value(t)?.into();
    Ok(t)
}

#[macro_export]
#[doc(hidden)]
macro_rules! json {
    //////////////////////////////////////////////////////////////////////////
    // TT muncher for parsing the inside of an array [...]. Produces a vec![...]
    // of the elements.
    //
    // Must be invoked as: json!(@array [] $($tt)*)
    //////////////////////////////////////////////////////////////////////////

    // Done with trailing comma.
    (@array [$($elems:expr,)*]) => {
        $crate::json_value_cmp::__private::vec![$($elems,)*]
    };

    // Done without trailing comma.
    (@array [$($elems:expr),*]) => {
        $crate::json_value_cmp::__private::vec![$($elems),*]
    };

    // Next element is `null`.
    (@array [$($elems:expr,)*] null $($rest:tt)*) => {
        $crate::json_value_cmp::json!(@array [$($elems,)* $crate::json_value_cmp::json!(null)] $($rest)*)
    };

    // Next element is `no_cmp`.
    (@array [$($elems:expr,)*] no_cmp $($rest:tt)*) => {
        $crate::json_value_cmp::json!(@array [$($elems,)* $crate::json_value_cmp::json!(no_cmp)] $($rest)*)
    };

    // Next element is `true`.
    (@array [$($elems:expr,)*] true $($rest:tt)*) => {
        $crate::json_value_cmp::json!(@array [$($elems,)* $crate::json_value_cmp::json!(true)] $($rest)*)
    };

    // Next element is `false`.
    (@array [$($elems:expr,)*] false $($rest:tt)*) => {
        $crate::json_value_cmp::json!(@array [$($elems,)* $crate::json_value_cmp::json!(false)] $($rest)*)
    };

    // Next element is an array.
    (@array [$($elems:expr,)*] [$($array:tt)*] $($rest:tt)*) => {
        $crate::json_value_cmp::json!(@array [$($elems,)* $crate::json_value_cmp::json!([$($array)*])] $($rest)*)
    };

    // Next element is a map.
    (@array [$($elems:expr,)*] {$($map:tt)*} $($rest:tt)*) => {
        $crate::json_value_cmp::json!(@array [$($elems,)* $crate::json_value_cmp::json!({$($map)*})] $($rest)*)
    };

    // Next element is an expression followed by comma.
    (@array [$($elems:expr,)*] $next:expr, $($rest:tt)*) => {
        $crate::json_value_cmp::json!(@array [$($elems,)* $crate::json_value_cmp::json!($next),] $($rest)*)
    };

    // Last element is an expression with no trailing comma.
    (@array [$($elems:expr,)*] $last:expr) => {
        $crate::json_value_cmp::json!(@array [$($elems,)* $crate::json_value_cmp::json!($last)])
    };

    // Comma after the most recent element.
    (@array [$($elems:expr),*] , $($rest:tt)*) => {
        $crate::json_value_cmp::json!(@array [$($elems,)*] $($rest)*)
    };

    // Unexpected token after most recent element.
    (@array [$($elems:expr),*] $unexpected:tt $($rest:tt)*) => {
        $crate::json_value_cmp::__private::serde_json::json_unexpected!($unexpected)
    };

    //////////////////////////////////////////////////////////////////////////
    // TT muncher for parsing the inside of an object {...}. Each entry is
    // inserted into the given map variable.
    //
    // Must be invoked as: json!(@object $map () ($($tt)*) ($($tt)*))
    //
    // We require two copies of the input tokens so that we can match on one
    // copy and trigger errors on the other copy.
    //////////////////////////////////////////////////////////////////////////

    // Done.
    (@object $object:ident () () ()) => {};

    // Insert the current entry followed by trailing comma.
    (@object $object:ident [$($key:tt)+] ($value:expr) , $($rest:tt)*) => {
        let _ = $object.insert(($($key)+).into(), $value);
        $crate::json_value_cmp::json!(@object $object () ($($rest)*) ($($rest)*));
    };

    // // Current entry followed by unexpected token.
    (@object $object:ident [$($key:tt)+] ($value:expr) $unexpected:tt $($rest:tt)*) => {
        $crate::serde_json::json_unexpected!($unexpected);
    };

    // Insert the last entry without trailing comma.
    (@object $object:ident [$($key:tt)+] ($value:expr)) => {
        let _ = $object.insert(($($key)+).into(), $value);
    };

    // Next value is `null`.
    (@object $object:ident ($($key:tt)+) (: null $($rest:tt)*) $copy:tt) => {
        $crate::json_value_cmp::json!(@object $object [$($key)+] ($crate::json_value_cmp::json!(null)) $($rest)*);
    };

    // Next value is `null`.
    (@object $object:ident ($($key:tt)+) (: no_cmp $($rest:tt)*) $copy:tt) => {
        $crate::json_value_cmp::json!(@object $object [$($key)+] ($crate::json_value_cmp::json!(no_cmp)) $($rest)*);
    };

    // Next value is `true`.
    (@object $object:ident ($($key:tt)+) (: true $($rest:tt)*) $copy:tt) => {
        $crate::json_value_cmp::json!(@object $object [$($key)+] ($crate::json_value_cmp::json!(true)) $($rest)*);
    };

    // Next value is `false`.
    (@object $object:ident ($($key:tt)+) (: false $($rest:tt)*) $copy:tt) => {
        $crate::json_value_cmp::json!(@object $object [$($key)+] ($crate::json_value_cmp::json!(false)) $($rest)*);
    };

    // Next value is an array.
    (@object $object:ident ($($key:tt)+) (: [$($array:tt)*] $($rest:tt)*) $copy:tt) => {
        $crate::json_value_cmp::json!(@object $object [$($key)+] ($crate::json_value_cmp::json!([$($array)*])) $($rest)*);
    };

    // Next value is a map.
    (@object $object:ident ($($key:tt)+) (: {$($map:tt)*} $($rest:tt)*) $copy:tt) => {
        $crate::json_value_cmp::json!(@object $object [$($key)+] ($crate::json_value_cmp::json!({$($map)*})) $($rest)*);
    };

    // Next value is an expression followed by comma.
    (@object $object:ident ($($key:tt)+) (: $value:expr , $($rest:tt)*) $copy:tt) => {
        $crate::json_value_cmp::json!(@object $object [$($key)+] ($crate::json_value_cmp::json!($value)) , $($rest)*);
    };

    // Last value is an expression with no trailing comma.
    (@object $object:ident ($($key:tt)+) (: $value:expr) $copy:tt) => {
        $crate::json_value_cmp::json!(@object $object [$($key)+] ($crate::json_value_cmp::json!($value)));
    };

    // Missing value for last entry. Trigger a reasonable error message.
    (@object $object:ident ($($key:tt)+) (:) $copy:tt) => {
        // "unexpected end of macro invocation"
        $crate::json_value_cmp::json!();
    };

    // Missing colon and value for last entry. Trigger a reasonable error
    // message.
    (@object $object:ident ($($key:tt)+) () $copy:tt) => {
        // "unexpected end of macro invocation"
        $crate::json_value_cmp::json!();
    };

    // Misplaced colon. Trigger a reasonable error message.
    (@object $object:ident () (: $($rest:tt)*) ($colon:tt $($copy:tt)*)) => {
        // Takes no arguments so "no rules expected the token `:`".
        $crate::json_value_cmp::json_unexpected!($colon);
    };

    // Found a comma inside a key. Trigger a reasonable error message.
    (@object $object:ident ($($key:tt)*) (, $($rest:tt)*) ($comma:tt $($copy:tt)*)) => {
        // Takes no arguments so "no rules expected the token `,`".
        $crate::json_value_cmp::json_unexpected!($comma);
    };

    // Key is fully parenthesized. This avoids clippy double_parens false
    // positives because the parenthesization may be necessary here.
    (@object $object:ident () (($key:expr) : $($rest:tt)*) $copy:tt) => {
        $crate::::json!(@object $object ($key) (: $($rest)*) (: $($rest)*));
    };

    // Refuse to absorb colon token into key expression.
    (@object $object:ident ($($key:tt)*) (: $($unexpected:tt)+) $copy:tt) => {
        $crate::json_value_cmp::json_expect_expr_comma!($($unexpected)+);
    };

    // Munch a token into the current key.
    (@object $object:ident ($($key:tt)*) ($tt:tt $($rest:tt)*) $copy:tt) => {
        $crate::json_value_cmp::json!(@object $object ($($key)* $tt) ($($rest)*) ($($rest)*));
    };

    //////////////////////////////////////////////////////////////////////////
    // The main implementation.
    //
    // Must be invoked as: json!($($json)+)
    //////////////////////////////////////////////////////////////////////////
    (no_cmp) => {
        $crate::json_value_cmp::Value::NoComp
    };

    (null) => {
        $crate::json_value_cmp::Value::Null
    };

    (true) => {
        $crate::json_value_cmp::Value::Bool(true)
    };

    (false) => {
        $crate::json_value_cmp::Value::Bool(false)
    };

    ([]) => {
        $crate::json_value_cmp::Value::Array($crate::json_value_cmp::__private::vec![])
    };

    ([ $($tt:tt)+ ]) => {
        $crate::json_value_cmp::Value::Array($crate::json_value_cmp::json!(@array [] $($tt)+))
    };

    ({}) => {
        $crate::json_value_cmp::Value::Object($crate::json_value_cmp::__private::Map::new())
    };

    ({ $($tt:tt)+ }) => {
        $crate::json_value_cmp::Value::Object({
            let mut object = $crate::json_value_cmp::__private::Map::new();
            $crate::json_value_cmp::json!(@object object () ($($tt)+) ($($tt)+));
            object
        })
    };

    // Any Serialize type: numbers, strings, struct literals, variables etc.
    // Must be below every other rule.
    ($other:expr) => {
        $crate::json_value_cmp::to_value(&$other).unwrap()
    };
}

pub use json;
