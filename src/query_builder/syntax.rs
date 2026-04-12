#![allow(non_camel_case_types)]

use crate::query_builder::SqlSyntax;

/// usually &'static str are hardcoded in source code at build time
/// sql injection caused by this impl is the developer's fault
/// which is unlikely because sql injection occur with malicious intent
/// ex: `String::new().leak()` or maliciouse build.rs
///
/// I'm removing this impl in favor of more specifics impls
impl SqlSyntax for &'static str {
    fn to_sql(&self, str: &mut String) {
        str.push_str(self);
    }
}

#[macro_export]
macro_rules! sql_syntax {
    ($ident:ident = $literal:literal) => {
        #[allow(non_camel_case_types)]
        pub struct $ident;

        impl $crate::query_builder::SqlSyntax for $ident {
            fn to_sql(&self, str: &mut String) {
                str.push_str($literal);
            }
        }
    };
}

sql_syntax!(equal_join = " = ");
sql_syntax!(space_join = " ");
sql_syntax!(open_paranthesis = "(");
sql_syntax!(close_paranthesis = ")");
sql_syntax!(empty = "");
sql_syntax!(comma_join = ", ");
sql_syntax!(and_join = " AND ");
sql_syntax!(end_of_statement = ";");
