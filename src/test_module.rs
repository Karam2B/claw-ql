#![allow(unused)]
#![allow(non_camel_case_types)]
#![deny(unused_must_use)]

use serde::{Deserialize, Serialize};

use crate::{
    query_builder::{Expression, ManyExpressions, OpExpression},
    singlton_default::SingltonDefault,
    update_mod::update,
};

#[derive(Debug, PartialEq, Eq, Clone, Serialize, Deserialize)]
pub struct Todo {
    pub title: String,
    pub done: bool,
    pub description: Option<String>,
}

#[derive(Default, Debug)]
pub struct TodoPartial {
    pub title: update<String>,
    pub done: update<bool>,
    pub description: update<Option<String>>,
}

#[allow(non_camel_case_types)]
#[derive(Clone, Default)]
pub struct todo;

impl OpExpression for todo {}
impl<'q, S> Expression<'q, S> for todo {
    fn expression(self, ctx: &mut crate::query_builder::QueryBuilder<'q, S>)
    where
        S: crate::database_extention::DatabaseExt,
    {
        ctx.sanitize("title");
        ctx.syntax(&crate::query_builder::syntax::comma_join);
        ctx.sanitize("done");
        ctx.syntax(&crate::query_builder::syntax::comma_join);
        ctx.sanitize("description");
    }
}

impl SingltonDefault for todo {
    fn singlton_default() -> &'static Self {
        &todo
    }
}

// impl ZeroOrMoreExpressions for Todo
const _: () = {
    use crate::database_extention::DatabaseExt;
    use crate::query_builder::SqlSyntax;
    use crate::query_builder::{IsOpExpression, QueryBuilder};
    use sqlx::{Encode, Type};

    impl IsOpExpression for Todo {
        fn is_op(&self) -> bool {
            true
        }
    }

    impl<'q, S: DatabaseExt> ManyExpressions<'q, S> for Todo
    where
        String: Type<S> + Encode<'q, S>,
        bool: Type<S> + Encode<'q, S>,
        Option<String>: Type<S> + Encode<'q, S>,
    {
        fn expression<Start: SqlSyntax + ?Sized, Join: SqlSyntax + ?Sized>(
            self,
            start: &Start,
            join: &Join,
            ctx: &mut QueryBuilder<'q, S>,
        ) where
            S: DatabaseExt,
        {
            ctx.syntax(start);
            ctx.bind(self.title);
            ctx.syntax(join);
            ctx.bind(self.done);
            ctx.syntax(join);
            ctx.bind(self.description);
        }

        fn to_expr(
            self,
        ) -> Vec<Box<dyn crate::prelude::macro_derive_collection::BoxedExpression<'q, S> + 'q>>
        where
            Self: Sized,
        {
            todo!()
        }
    }
};

mod impl_zero_or_more_expressions_for_todo_partial {
    use super::TodoPartial;
    use crate::query_builder::ManyExpressions;
    use crate::query_builder::syntax::{comma_join, equal_join};
    use crate::update_mod::update;
    use crate::{database_extention::DatabaseExt, query_builder::IsOpExpression};
    use sqlx::{Encode, Type};

    impl IsOpExpression for TodoPartial {
        fn is_op(&self) -> bool {
            !matches!(self.title, update::keep)
                || !matches!(self.done, update::keep)
                || !matches!(self.description, update::keep)
        }
    }

    impl<'q, S: DatabaseExt> ManyExpressions<'q, S> for TodoPartial
    where
        String: Type<S> + Encode<'q, S>,
        bool: Type<S> + Encode<'q, S>,
        Option<String>: Type<S> + Encode<'q, S>,
    {
        fn to_expr(
            self,
        ) -> Vec<Box<dyn crate::prelude::macro_derive_collection::BoxedExpression<'q, S> + 'q>>
        where
            Self: Sized,
        {
            todo!()
        }

        fn expression<
            Start: crate::query_builder::SqlSyntax + ?Sized,
            Join: crate::query_builder::SqlSyntax + ?Sized,
        >(
            self,
            start: &Start,
            join: &Join,
            ctx: &mut crate::query_builder::QueryBuilder<'q, S>,
        ) where
            S: DatabaseExt,
        {
            if self.is_op() {
                ctx.syntax(start);
            }
            let mut join_now = false;
            if let update::set(title) = self.title {
                ctx.sanitize("title");
                ctx.syntax(&equal_join);
                ctx.bind(title);
                join_now = true
            }
            if let update::set(done) = self.done {
                if join_now {
                    ctx.syntax(&comma_join);
                }
                ctx.sanitize("done");
                ctx.syntax(&equal_join);
                ctx.bind(done);
                join_now = true
            }

            if let update::set(description) = self.description {
                if join_now {
                    ctx.syntax(join);
                }
                ctx.sanitize("description");
                ctx.syntax(&equal_join);
                ctx.bind(description);
            }
        }
    }
}

#[allow(non_camel_case_types)]
pub mod todo_members {
    use super::todo;
    use crate::prelude::macro_derive_collection::*;

    #[derive(Clone, Default)]
    pub struct title;

    impl MemberBasic for title {
        fn name(&self) -> &str {
            "title"
        }
    }

    impl Member for title {
        type CollectionHandler = todo;
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
        type CollectionHandler = todo;
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
        type CollectionHandler = todo;
        type Data = Option<String>;
    }
    #[derive(Clone, Default)]
    pub struct id;

    impl MemberBasic for id {
        fn name(&self) -> &str {
            SingleIncremintalInt.ident()
        }
    }
    impl Member for id {
        type CollectionHandler = todo;
        type Data = <SingleIncremintalInt as Id>::Data;
    }
}

const _: () = {
    use crate::prelude::macro_derive_collection::*;

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
        type Id = SingleIncremintalInt;
        fn id(&self) -> &Self::Id {
            &SingleIncremintalInt
        }
    }

    impl HasHandler for Todo {
        type Handler = todo;
    }
    impl HasHandler for TodoPartial {
        type Handler = todo;
    }

    impl<S> Members<S> for todo {
        fn members_names(&self) -> Vec<String> {
            vec![
                "title".to_string(),
                "done".to_string(),
                "description".to_string(),
            ]
        }
    }
};

const _: () = {
    use crate::prelude::on_migrate_derive::*;
    impl OnMigrate for todo {
        type Statements = CreateTable<
            create_table,
            table_as_expression<todo>,
            (
                <todo as Collection>::Id,
                col_def_for_collection_member<todo_members::title>,
                col_def_for_collection_member<todo_members::done>,
                col_def_for_collection_member<todo_members::description>,
            ),
        >;
        fn statments(&self) -> Self::Statements {
            CreateTable {
                init: create_table,
                name: table_as_expression(todo),
                col_defs: (
                    Collection::id(self).clone(),
                    col_def_for_collection_member(todo_members::title),
                    col_def_for_collection_member(todo_members::done),
                    col_def_for_collection_member(todo_members::description),
                ),
            }
        }
    }
};

// FromRowAlias derive for todo
const _: () = {
    use crate::prelude::from_row_alias::*;
    impl<'r, R: sqlx::Row> FromRowAlias<'r, R> for todo
    where
        R: Row + 'r,
        String: Type<R::Database> + Decode<'r, R::Database>,
        bool: Type<R::Database> + Decode<'r, R::Database>,
        Option<String>: Type<R::Database> + Decode<'r, R::Database>,
        for<'a> &'a str: ColumnIndex<R>,
    {
        type FromRowData = Todo;
        fn no_alias(&self, row: &'r R) -> Result<Self::FromRowData, FromRowError> {
            Ok(Todo {
                title: row.try_get("title")?,
                done: row.try_get("done")?,
                description: row.try_get("description")?,
            })
        }
        fn pre_alias(&self, row: pre_alias<'r, R>) -> Result<Self::FromRowData, FromRowError> {
            Ok(Todo {
                title: row.0.try_get(format!("{}title", row.1).as_str())?,
                done: row.0.try_get(format!("{}done", row.1).as_str())?,
                description: row.0.try_get(format!("{}description", row.1).as_str())?,
            })
        }
        fn post_alias(&self, row: post_alias<'r, R>) -> Result<Self::FromRowData, FromRowError> {
            Ok(Todo {
                title: row.0.try_get(format!("title{}", row.1).as_str())?,
                done: row.0.try_get(format!("done{}", row.1).as_str())?,
                description: row.0.try_get(format!("description{}", row.1).as_str())?,
            })
        }
    }
};

#[derive(Debug, PartialEq, Eq, Clone, Serialize, Deserialize)]
pub struct Category {
    pub title: String,
}

// impl ZeroOrMoreExpressions for Category
const _: () = {
    use crate::database_extention::DatabaseExt;
    use crate::prelude::macro_derive_collection::*;
    use crate::query_builder::{IsOpExpression, ManyExpressions};
    impl IsOpExpression for Category {
        fn is_op(&self) -> bool {
            true
        }
    }
    impl<'q, S: DatabaseExt> ManyExpressions<'q, S> for Category
    where
        String: Type<S> + Encode<'q, S>,
    {
        fn to_expr(
            self,
        ) -> Vec<Box<dyn crate::prelude::macro_derive_collection::BoxedExpression<'q, S> + 'q>>
        where
            Self: Sized,
        {
            todo!()
        }
        fn expression<
            Start: crate::query_builder::SqlSyntax + ?Sized,
            Join: crate::query_builder::SqlSyntax + ?Sized,
        >(
            self,
            start: &Start,
            join: &Join,
            ctx: &mut crate::query_builder::QueryBuilder<'q, S>,
        ) where
            S: crate::database_extention::DatabaseExt,
        {
            ctx.syntax(start);
            ctx.bind(self.title);
        }
    }
};

// FromRowAlias derive for category
const _: () = {
    use crate::prelude::from_row_alias::*;
    impl<'r, R: sqlx::Row> FromRowAlias<'r, R> for category
    where
        R: Row + 'r,
        String: Type<R::Database> + Decode<'r, R::Database>,
        for<'a> &'a str: ColumnIndex<R>,
    {
        type FromRowData = Category;
        fn no_alias(&self, row: &'r R) -> Result<Self::FromRowData, FromRowError> {
            Ok(Category {
                title: row.try_get("title")?,
            })
        }
        fn pre_alias(&self, row: pre_alias<'r, R>) -> Result<Self::FromRowData, FromRowError> {
            Ok(Category {
                title: row.0.try_get(format!("{}title", row.1).as_str())?,
            })
        }
        fn post_alias(&self, row: post_alias<'r, R>) -> Result<Self::FromRowData, FromRowError> {
            Ok(Category {
                title: row.0.try_get(format!("title{}", row.1).as_str())?,
            })
        }
    }
};

#[derive(Default, Debug)]
pub struct CategoryPartial {
    pub title: update<String>,
}

#[derive(Clone, Default)]
pub struct category;

impl SingltonDefault for category {
    fn singlton_default() -> &'static Self {
        &category
    }
}

const _: () = {
    use crate::prelude::macro_derive_collection::*;

    impl CollectionBasic for category {
        fn table_name_lower_case(&self) -> &'static str {
            "category"
        }
        fn table_name(&self) -> &'static str {
            "Category"
        }
    }

    impl Collection for category {
        type Partial = CategoryPartial;
        type Data = Category;
        type Id = SingleIncremintalInt;
        fn id(&self) -> &Self::Id {
            &SingleIncremintalInt
        }
    }
    impl HasHandler for Category {
        type Handler = category;
    }
    impl HasHandler for CategoryPartial {
        type Handler = category;
    }
    impl<S> Members<S> for category {
        fn members_names(&self) -> Vec<String> {
            vec!["title".to_string()]
        }
    }
};

pub mod category_members {
    use super::category;
    use crate::prelude::macro_derive_collection::*;

    #[derive(Clone, Default)]
    pub struct title;

    impl MemberBasic for title {
        fn name(&self) -> &str {
            "title"
        }
    }
    impl Member for title {
        type CollectionHandler = category;
        type Data = String;
    }

    #[derive(Clone, Default)]
    pub struct id;

    impl MemberBasic for id {
        fn name(&self) -> &str {
            SingleIncremintalInt.ident()
        }
    }
    impl Member for id {
        type CollectionHandler = category;
        type Data = <SingleIncremintalInt as Id>::Data;
    }
}

// OnMigrate derive for category
const _: () = {
    use crate::prelude::on_migrate_derive::*;
    impl OnMigrate for category {
        type Statements = CreateTable<
            create_table,
            table_as_expression<category>,
            (
                <category as Collection>::Id,
                col_def_for_collection_member<category_members::title>,
            ),
        >;
        fn statments(&self) -> Self::Statements {
            CreateTable {
                init: create_table,
                name: table_as_expression(category),
                col_defs: (
                    Collection::id(self).clone(),
                    col_def_for_collection_member(category_members::title),
                ),
            }
        }
    }
};

mod impl_link {
    use super::{category, todo};
    use crate::links::Link;
    use crate::links::relation_optional_to_many::optional_to_many;

    impl Link<todo> for category {
        type Spec = optional_to_many<String, todo, category>;
        fn spec(self, _: &todo) -> Self::Spec {
            optional_to_many {
                foriegn_key: String::from("category_id"),
                from: todo,
                to: self,
            }
        }
    }
}
