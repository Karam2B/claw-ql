#![allow(non_camel_case_types)]

use crate::query_builder::SqlSyntax;

/// usually &'static str are hardcoded in source code at build time
/// sql injection caused by this impl is the developer's fault
/// which is unlikely because sql injection occur with malicious intent
/// ex: `String::new().leak()` or maliciouse build.rs
///
/// I'm removing this impl in favor of more specifics impls
impl SqlSyntax for &'static str {
    fn to_sql(self, str: &mut String) {
        str.push_str(self);
    }
}

macro_rules! sql_syntax {
    ($($ident:ident = $literal:literal),*) => {

$(
pub struct $ident;

impl SqlSyntax for $ident {
    fn to_sql(self, str: &mut String) {
        str.push_str($literal);
    }
}
)*
    };
}

#[rustfmt::skip]
sql_syntax!(
    and_join = " AND ", space_join = " "
);
