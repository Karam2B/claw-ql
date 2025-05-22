use crate::collection_derive::main;
use quote::quote;
use syn::parse_quote;

#[test]
fn test_todo() {
    let input = parse_quote!(
        pub struct Todo {
            pub title: String,
            pub done: bool,
            pub description: Option<String>,
        }
    );

    let output = main(input);

    let expected = quote!(
        const _: () = {
            use claw_ql::prelude::derive_collection::*;

            #[cfg_attr(feature = "serde", derive(Deserialize))]
            pub struct TodoPartial {
                pub title: update<String>,
                pub done: update<bool>,
                pub description: update<Option<String>>,
            }

            impl<S> Collection<S> for Todo
            where
                S: QueryBuilder,
                for<'s> &'s str: ColumnIndex<<S as Database>::Row>,
                String: Type<S> + for<'c> Decode<'c, S> + for<'e> Encode<'e, S>,
                bool: Type<S> + for<'c> Decode<'c, S> + for<'e> Encode<'e, S>,
                Option<String>: Type<S> + for<'c> Decode<'c, S> + for<'e> Encode<'e, S>,
            {
                type PartailCollection = TodoPartial;

                fn on_select(stmt: &mut SelectSt<S>)
                where
                    S: QueryBuilder,
                {
                    stmt.select(col("title"));
                    stmt.select(col("done"));
                    stmt.select(col("description"));
                }

                fn members() -> &'static [&'static str] {
                    &["title", "done", "description"]
                }

                fn members_scoped() -> &'static [&'static str] {
                    &["todo_title", "todo_done", "todo_description"]
                }

                fn table_name() -> &'static str {
                    "todo"
                }

                fn from_row_noscope(row: &<S>::Row) -> Self
                where
                    S: sqlx::Database,
                {
                    Self {
                        title: row.get("title"),
                        done: row.get("done"),
                        description: row.get("description"),
                    }
                }

                fn from_row_scoped(row: &<S>::Row) -> Self
                where
                    S: sqlx::Database,
                {
                    Self {
                        title: row.get("todo_title"),
                        done: row.get("todo_done"),
                        description: row.get("todo_description"),
                    }
                }
            }
        };
    );

    assert_eq!(expected, output);
}
