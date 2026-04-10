#![allow(non_camel_case_types)]
use crate::SqlSyntax;

/// usually &'static str are hardcoded in source code at build time
/// sql injection caused by this impl is the developer's fault
/// which is unlikely because sql injection occur with malicious intent
/// ex: `String::new().leak()` or maliciouse build.rs
///
/// I should remove this impl and replace it with each valid syntax like `and_join`
impl<S> SqlSyntax<S> for &'static str {
    fn to_sql(self, str: &mut String) {
        str.push_str(self);
    }
}

pub struct and_join;

impl<S> SqlSyntax<S> for and_join {
    fn to_sql(self, str: &mut String) {
        str.push_str(" AND ");
    }
}
