#![allow(non_camel_case_types)]
pub struct Todo {
    pub title: String,
    pub done: bool,
    pub description: Option<String>,
}

#[derive(Default, Debug)]
pub struct TodoPartial {
    pub title: ::claw_ql::prelude::macro_derive_collection::update<String>,
    pub done: ::claw_ql::prelude::macro_derive_collection::update<bool>,
    pub description: ::claw_ql::prelude::macro_derive_collection::update<Option<String>>,
}

#[allow(non_camel_case_types)]
#[derive(Clone, Default)]
pub struct todo;

#[allow(non_camel_case_types)]
pub mod todo_members {
    use super::prelude::*;
    use super::todo;

    #[derive(Clone, Default)]
    pub struct title;

    impl MemberBasic for title {
        fn name(&self) -> &str {
            "title"
        }
    }

    impl Member for title {
        type Collection = todo;
        type Data = String;
    }

    #[derive(Clone, Default)]
    pub struct done;

    impl MemberBasic for done {
        fn name(&self) -> &str {
            "done"
        }
    }

    impl Member for done {
        type Collection = todo;
        type Data = bool;
    }

    #[derive(Clone, Default)]
    pub struct description;

    impl MemberBasic for description {
        fn name(&self) -> &str {
            "description"
        }
    }

    impl Member for description {
        type Collection = todo;
        type Data = bool;
    }
}

pub mod prelude {
    pub use ::claw_ql::EncodeExtention;
    pub use ::claw_ql::QueryBuilder;
    pub use ::claw_ql::SanitzingMechanisim;
    pub use ::claw_ql::collections::Collection;
    pub use ::claw_ql::collections::CollectionBasic;
    pub use ::claw_ql::collections::HasHandler;
    pub use ::claw_ql::collections::Member;
    pub use ::claw_ql::collections::MemberBasic;
    pub use ::claw_ql::collections::Queries;
    pub use ::claw_ql::collections::SingleIncremintalInt;
    pub use ::claw_ql::expressions::col;
    pub use ::claw_ql::expressions::primary_key::DatabaseDefaultPrimaryKey;
    pub use ::claw_ql::sanitize::SanitizeAndHardcode;
    pub use ::claw_ql::statements::select_statement::SelectSt;
    pub use sqlx::ColumnIndex;
    pub use sqlx::Database;
    pub use sqlx::Decode;
    pub use sqlx::Encode;
    pub use sqlx::Row;
    pub use sqlx::Type;
}

mod collection_impl {
    use super::prelude::*;
    use super::todo_members::*;
    use super::*;

    impl CollectionBasic for todo {
        fn table_name_lower_case(&self) -> &'static str {
            "todo"
        }
        fn table_name(&self) -> &'static str {
            "Todo"
        }
    }

    impl Collection for todo {
        type Partial = TodoPartial;
        type Data = Todo;
        type Members = (title, done, description);
        fn members(&self) -> &Self::Members {
            &(title, done, description)
        }
        type Id = SingleIncremintalInt;
        fn id(&self) -> &Self::Id {
            &SingleIncremintalInt
        }
    }

    impl<S> Queries<S> for todo
    where
        S: Database,
    {
        fn from_row_scoped(
            &self,
            row: &<<S as QueryBuilder>::SqlxDb as Database>::Row,
        ) -> Self::Data
        where
            S: QueryBuilder,
            <S>::SqlxDb: Database,
        {
            todo!()
        }
        fn on_select(&self, stmt: &mut SelectSt<S>)
        where
            S: QueryBuilder,
        {
            todo!()
        }
    }

    impl HasHandler for TodoPartial {
        type Handler = todo;
    }

    impl HasHandler for Todo {
        type Handler = todo;
    }
}

#[derive(claw_ql_macros::Collection)]
pub struct Category {
    pub title: String,
}
