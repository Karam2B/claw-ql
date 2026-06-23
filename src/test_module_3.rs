#![allow(unused)]
#![allow(non_camel_case_types)]
#![deny(unused_must_use)]

use std::convert::Infallible;

use serde::{Deserialize, Serialize};
use sqlx::{Database, Encode, prelude::Type};

use crate::{
    database_extention::DatabaseExt,
    extentions::common_expressions::{
        MembersAndIdAliased, OnInsert, OnUpdate, TableNameExpression, V0OnInsert, V0OnUpdate,
    },
    singleton::Singleton,
    sqlx_query_builder::{
        Expression, IsOpExpression, ManyExpressions, OpExpression, StatementBuilder,
        basic_expressions::PossibleImplExpression,
    },
    tuple_trait::AsTuple,
    update_mod::Update,
};

#[derive(Debug, PartialEq, Eq, Clone, Serialize, Deserialize)]
pub struct Todo {
    pub title: String,
    pub done: bool,
    pub description: Option<String>,
}

#[derive(Default, Debug, Serialize, Deserialize)]
pub struct TodoPartial {
    pub title: Update<String>,
    pub done: Update<bool>,
    pub description: Update<Option<String>>,
}

#[allow(non_camel_case_types)]
#[derive(Clone, Default)]
pub struct todo;
pub use self::todo as todo_test;

impl Singleton for todo {
    fn singleton() -> &'static Self {
        &todo
    }
}

const _: () = {
    use crate::prelude::macro_derive_collection::*;

    impl AsTuple for TodoPartial {
        type Tuple = (Update<String>, Update<bool>, Update<Option<String>>);
        const NAMES: &'static [&'static str] = &["title", "done", "description"];
        fn into_tuple(self) -> Self::Tuple {
            (self.title, self.done, self.description)
        }
        fn from_tuple(tuple: Self::Tuple) -> Self {
            Self {
                title: tuple.0,
                done: tuple.1,
                description: tuple.2,
            }
        }
    }

    impl Collection for todo {
        fn table_name(&self) -> &str {
            "Todo"
        }
        fn table_name_lower_case(&self) -> &str {
            "todo"
        }

        type InputData = Todo;
        type UpdateData = TodoPartial;
        type OutputData = Todo;

        type Id = SingleIncremintalInt<&'static str>;
        fn id(&self) -> Self::Id {
            SingleIncremintalInt("Todo")
        }
    }

    impl HasHandler for Todo {
        type Handler = todo;
    }
    impl HasHandler for TodoPartial {
        type Handler = todo;
    }

    impl AsTuple for Todo {
        type Tuple = (String, bool, Option<String>);
        const NAMES: &'static [&'static str] = &["title", "done", "description"];

        fn into_tuple(self) -> Self::Tuple {
            (self.title, self.done, self.description)
        }

        fn from_tuple(tuple: Self::Tuple) -> Self {
            Self {
                title: tuple.0,
                done: tuple.1,
                description: tuple.2,
            }
        }
    }

    impl OnMigrate for todo {
        type Statements = CreateTable<
            create_table,
            table_as_expression<todo>,
            ManyPossible<(
                <todo as Collection>::Id,
                col_def_for_collection_member<todo_members::title>,
                col_def_for_collection_member<todo_members::done>,
                col_def_for_collection_member<todo_members::description>,
            )>,
        >;
        fn statments(&self) -> Self::Statements {
            CreateTable {
                init: create_table,
                name: table_as_expression(todo),
                col_defs: ManyPossible((
                    Collection::id(self).clone(),
                    col_def_for_collection_member(todo_members::title),
                    col_def_for_collection_member(todo_members::done),
                    col_def_for_collection_member(todo_members::description),
                )),
            }
        }
    }
};

// impl common expressions for category
#[claw_ql_macros::skip]
const _: () = {
    use crate::extentions::common_expressions::{Aliased, Identifier, MigrateExpression, Scoped};

    impl TableNameExpression for category {
        type TableNameExpression = &'static str;
        fn table_name_expression(&self) -> Self::TableNameExpression {
            "Category"
        }
        type LowerCaseTableNameExpression = &'static str;
        fn lower_case_table_name_expression(&self) -> Self::LowerCaseTableNameExpression {
            "category"
        }
    }

    impl Scoped for category {
        type Scoped = ScopedCols<'static>;
        fn scoped(&self) -> Self::Scoped {
            ScopedCols {
                table: "Category",
                cols: <Category as AsTuple>::NAMES,
            }
        }
    }

    impl Aliased for category {
        type Aliased = AliasedCols<'static>;
        fn aliased(&self, alias: &'static str) -> Self::Aliased {
            AliasedCols {
                table: "Category",
                cols: <Category as AsTuple>::NAMES,
                alias,
            }
        }
        type NumAliased = NumAliasedCols<'static>;
        fn num_aliased(&self, num: usize, alias: &'static str) -> Self::NumAliased {
            NumAliasedCols {
                table: "Category",
                cols: <Category as AsTuple>::NAMES,
                num,
                alias,
            }
        }
    }

    impl MembersAndIdAliased for category {
        type MembersAndIdAliased = AliasedCols<'static>;
        fn members_and_id_aliased(&self, alias: &'static str) -> Self::MembersAndIdAliased {
            AliasedCols {
                table: "Category",
                cols: &["id", "title"],
                alias,
            }
        }
    }

    impl Identifier for category {
        type Identifier = &'static [&'static str];
        fn identifier(&self) -> Self::Identifier {
            <Category as AsTuple>::NAMES
        }
    }

    impl V0OnUpdate for category {
        type UpdateInput = CategoryPartial;
        type UpdateExpression = CategoryPartial;

        fn on_update(self, input: Self::UpdateInput) -> Self::UpdateExpression {
            input
        }
    }

    impl V0OnInsert for category {
        type InsertInput = Category;
        type InsertExpression = Category;

        fn on_insert(self, input: Self::InsertInput) -> Self::InsertExpression {
            input
        }
    }

    impl OnInsert for category {
        type InsertInput = Category;
        type InsertExpression = Category;

        fn on_insert(&self, input: Self::InsertInput) -> Self::InsertExpression {
            input
        }
    }

    impl OnUpdate for category {
        type UpdateInput = CategoryPartial;
        type UpdateExpression = CategoryPartial;

        fn on_update(&self, input: Self::UpdateInput) -> Self::UpdateExpression {
            input
        }
    }

    impl IsOpExpression for Category {
        fn is_op(&self) -> bool {
            true
        }
    }

    impl<'q, S: DatabaseExt> ManyExpressions<'q, S> for Category
    where
        String: Type<S> + Encode<'q, S>,
    {
        fn expression(
            self,
            start: &'static str,
            join: &'static str,
            ctx: &mut StatementBuilder<'q, S>,
        ) where
            S: DatabaseExt,
        {
            ctx.syntax(start);
            ctx.bind(self.title);
        }
    }

    impl IsOpExpression for CategoryPartial {
        fn is_op(&self) -> bool {
            matches!(self.title, Update::Set(_))
        }
    }
    impl<'q, S> ManyExpressions<'q, S> for CategoryPartial
    where
        S: Database,
        String: Type<S> + Encode<'q, S>,
    {
        fn expression(
            self,
            start: &'static str,
            join: &'static str,
            ctx: &mut StatementBuilder<'q, S>,
        ) where
            S: DatabaseExt,
        {
            if let Update::Set(title) = self.title {
                ctx.syntax(start);
                ctx.sanitize("title");
                ctx.syntax(" = ");
                ctx.bind(title);
            }
        }
    }

    impl MigrateExpression for category {
        type MigrateExpression = (MigratingCol<&'static str, String>,);
        fn migrate_expression(&self) -> Self::MigrateExpression {
            (MigratingCol {
                col: "title",
                phantom: std::marker::PhantomData,
            },)
        }
    }
};

// impl common expressions for todo
#[claw_ql_macros::skip]
const _: () = {
    use crate::extentions::common_expressions::{Aliased, Identifier, MigrateExpression, Scoped};

    impl TableNameExpression for todo {
        type TableNameExpression = &'static str;
        fn table_name_expression(&self) -> Self::TableNameExpression {
            "Todo"
        }
        type LowerCaseTableNameExpression = &'static str;
        fn lower_case_table_name_expression(&self) -> Self::LowerCaseTableNameExpression {
            "todo"
        }
    }

    impl Scoped for todo {
        type Scoped = ScopedCols<'static>;

        fn scoped(&self) -> Self::Scoped {
            ScopedCols {
                table: "Todo",
                cols: <Todo as AsTuple>::NAMES,
            }
        }
    }

    impl Aliased for todo {
        type Aliased = AliasedCols<'static>;

        fn aliased(&self, alias: &'static str) -> Self::Aliased {
            AliasedCols {
                table: "Todo",
                cols: <Todo as AsTuple>::NAMES,
                alias,
            }
        }
        type NumAliased = NumAliasedCols<'static>;
        fn num_aliased(&self, num: usize, alias: &'static str) -> Self::NumAliased {
            NumAliasedCols {
                table: "Todo",
                cols: <Todo as AsTuple>::NAMES,
                num,
                alias,
            }
        }
    }

    impl MembersAndIdAliased for todo {
        type MembersAndIdAliased = AliasedCols<'static>;
        fn members_and_id_aliased(&self, alias: &'static str) -> Self::MembersAndIdAliased {
            AliasedCols {
                table: "Todo",
                cols: &["id", "title", "done", "description"],
                alias,
            }
        }
    }

    impl Identifier for todo {
        type Identifier = &'static [&'static str];
        fn identifier(&self) -> Self::Identifier {
            <Todo as AsTuple>::NAMES
        }
    }

    impl MigrateExpression for todo {
        type MigrateExpression = (
            MigratingCol<&'static str, String>,
            MigratingCol<&'static str, bool>,
            MigratingCol<&'static str, Option<String>>,
        );

        fn migrate_expression(&self) -> Self::MigrateExpression {
            (
                MigratingCol {
                    col: "title",
                    phantom: std::marker::PhantomData,
                },
                MigratingCol {
                    col: "done",
                    phantom: std::marker::PhantomData,
                },
                MigratingCol {
                    col: "description",
                    phantom: std::marker::PhantomData,
                },
            )
        }
    }

    impl V0OnInsert for todo {
        type InsertInput = Todo;
        type InsertExpression = Todo;

        fn on_insert(self, input: Self::InsertInput) -> Self::InsertExpression {
            input
        }
    }

    impl OnInsert for todo {
        type InsertInput = Todo;
        type InsertExpression = Todo;

        fn on_insert(&self, input: Self::InsertInput) -> Self::InsertExpression {
            input
        }
    }

    impl V0OnUpdate for todo {
        type UpdateInput = TodoPartial;
        type UpdateExpression = TodoPartial;

        fn on_update(self, input: Self::UpdateInput) -> Self::UpdateExpression {
            input
        }
    }

    impl OnUpdate for todo {
        type UpdateInput = TodoPartial;
        type UpdateExpression = TodoPartial;

        fn on_update(&self, input: Self::UpdateInput) -> Self::UpdateExpression {
            input
        }
    }

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
        fn expression(
            self,
            start: &'static str,
            join: &'static str,
            ctx: &mut StatementBuilder<'q, S>,
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
    }

    impl IsOpExpression for TodoPartial {
        fn is_op(&self) -> bool {
            !matches!(self.title, Update::Keep)
                || !matches!(self.done, Update::Keep)
                || !matches!(self.description, Update::Keep)
        }
    }

    impl<'q, S: DatabaseExt> ManyExpressions<'q, S> for TodoPartial
    where
        String: Type<S> + Encode<'q, S>,
        bool: Type<S> + Encode<'q, S>,
        Option<String>: Type<S> + Encode<'q, S>,
    {
        fn expression(
            self,
            start: &'static str,
            join: &'static str,
            ctx: &mut crate::sqlx_query_builder::StatementBuilder<'q, S>,
        ) where
            S: DatabaseExt,
        {
            if self.is_op() {
                ctx.syntax(start);
            }
            let mut join_now = false;
            if let Update::Set(title) = self.title {
                ctx.sanitize("title");
                ctx.syntax(" = ");
                ctx.bind(title);
                join_now = true
            }
            if let Update::Set(done) = self.done {
                if join_now {
                    ctx.syntax(join);
                }
                ctx.sanitize("done");
                ctx.syntax(" = ");
                ctx.bind(done);
                join_now = true
            }

            if let Update::Set(description) = self.description {
                if join_now {
                    ctx.syntax(join);
                }
                ctx.sanitize("description");
                ctx.syntax(" = ");
                ctx.bind(description);
            }
        }
    }
};

#[allow(non_camel_case_types)]
#[claw_ql_macros::skip]
pub mod todo_members {
    use super::todo;
    use crate::extentions::common_expressions::{V0OnInsert, V0OnUpdate};
    use crate::prelude::macro_derive_collection::*;
    use crate::sqlx_query_builder::OpExpression;

    #[derive(Clone, Default)]
    pub struct title;

    impl Member for title {
        fn name(&self) -> &str {
            "title"
        }
        type CollectionHandler = todo;
        type Data = String;
    }

    crate::member_impl_from_row_alias!(title);
    crate::member_impl_debug!(title);
    crate::member_is_unique_filter!(title, false);
    crate::member_impl_expression!(title);

    #[derive(Clone, Default)]
    pub struct done;

    impl Member for done {
        fn name(&self) -> &str {
            "done"
        }
        type CollectionHandler = todo;
        type Data = bool;
    }

    crate::member_impl_from_row_alias!(done);
    crate::member_impl_debug!(done);
    crate::member_is_unique_filter!(done, false);
    crate::member_impl_expression!(done);

    #[derive(Clone, Default)]
    pub struct description;

    impl Member for description {
        fn name(&self) -> &str {
            "description"
        }
        type CollectionHandler = todo;
        type Data = Option<String>;
    }

    crate::member_impl_from_row_alias!(description);
    crate::member_impl_debug!(description);
    crate::member_is_unique_filter!(description, false);
    crate::member_impl_expression!(description);

    #[derive(Clone, Default)]
    pub struct id;

    impl Member for id {
        fn name(&self) -> &str {
            "id"
        }
        type CollectionHandler = todo;
        type Data = <SingleIncremintalInt<&'static str> as CollectionId>::IdData;
    }

    crate::member_impl_from_row_alias!(id);
    crate::member_impl_debug!(id);
    crate::member_is_unique_filter!(id, true);
    crate::member_impl_expression!(id);
}

// FromRowAlias derive for todo
const _: () = {
    use crate::prelude::from_row_alias::*;
    impl FromRowData for todo {
        type RData = Todo;
    }
    impl<'r, R: sqlx::Row> FromRowAlias<'r, R> for todo
    where
        R: Row + 'r,
        String: Type<R::Database> + Decode<'r, R::Database>,
        bool: Type<R::Database> + Decode<'r, R::Database>,
        Option<String>: Type<R::Database> + Decode<'r, R::Database>,
        for<'a> &'a str: ColumnIndex<R>,
    {
        fn no_alias(&self, row: &'r R) -> Result<Self::RData, FromRowError> {
            Ok(Todo {
                title: row.try_get("title")?,
                done: row.try_get("done")?,
                description: row.try_get("description")?,
            })
        }
        fn pre_alias(&self, row: RowPreAliased<'r, R>) -> Result<Self::RData, FromRowError> {
            Ok(Todo {
                title: row.try_get("title")?,
                done: row.try_get("done")?,
                description: row.try_get("description")?,
            })
        }
        fn two_alias(
            &self,
            row: crate::from_row::RowTwoAliased<'r, R>,
        ) -> Result<Self::RData, crate::from_row::FromRowError>
        where
            R: sqlx::Row,
        {
            Ok(Todo {
                title: row.try_get("title")?,
                done: row.try_get("done")?,
                description: row.try_get("description")?,
            })
        }
        fn post_alias(&self, row: RowPostAliased<'r, R>) -> Result<Self::RData, FromRowError> {
            Ok(Todo {
                title: row.try_get("title")?,
                done: row.try_get("done")?,
                description: row.try_get("description")?,
            })
        }
    }
};

#[derive(Debug, PartialEq, Eq, Clone, Serialize, Deserialize)]
pub struct Category {
    pub title: String,
}

// FromRowAlias derive for category
const _: () = {
    use crate::prelude::from_row_alias::*;

    impl<'r> FromRowData for category {
        type RData = Category;
    }

    impl<'r, R: sqlx::Row> FromRowAlias<'r, R> for category
    where
        R: Row + 'r,
        String: Type<R::Database> + Decode<'r, R::Database>,
        for<'a> &'a str: ColumnIndex<R>,
    {
        fn two_alias(
            &self,
            row: crate::from_row::RowTwoAliased<'r, R>,
        ) -> Result<Self::RData, crate::from_row::FromRowError>
        where
            R: sqlx::Row,
        {
            Ok(Category {
                title: row.try_get("title")?,
            })
        }
        fn no_alias(&self, row: &'r R) -> Result<Self::RData, FromRowError> {
            Ok(Category {
                title: row.try_get("title")?,
            })
        }
        fn pre_alias(&self, row: RowPreAliased<'r, R>) -> Result<Self::RData, FromRowError> {
            Ok(Category {
                title: row.try_get("title")?,
            })
        }
        fn post_alias(&self, row: RowPostAliased<'r, R>) -> Result<Self::RData, FromRowError> {
            Ok(Category {
                title: row.try_get("title")?,
            })
        }
    }
};

#[derive(Default, Debug)]
pub struct CategoryPartial {
    pub title: Update<String>,
}

#[derive(Clone, Default, Debug)]
pub struct category;

impl Singleton for category {
    fn singleton() -> &'static Self {
        &category
    }
}

const _: () = {
    use crate::prelude::macro_derive_collection::*;

    impl Collection for category {
        fn table_name(&self) -> &str {
            "Category"
        }
        fn table_name_lower_case(&self) -> &str {
            "category"
        }

        type InputData = Category;
        type UpdateData = CategoryPartial;
        type OutputData = Category;

        type Id = SingleIncremintalInt<&'static str>;
        fn id(&self) -> Self::Id {
            SingleIncremintalInt("Category")
        }
    }

    impl HasHandler for Category {
        type Handler = category;
    }
    impl HasHandler for CategoryPartial {
        type Handler = category;
    }
    impl AsTuple for Category {
        type Tuple = (String,);
        const NAMES: &'static [&'static str] = &["title"];
        fn into_tuple(self) -> Self::Tuple {
            (self.title,)
        }
        fn from_tuple(tuple: Self::Tuple) -> Self {
            Self { title: tuple.0 }
        }
    }
    impl AsTuple for CategoryPartial {
        type Tuple = (Update<String>,);
        const NAMES: &'static [&'static str] = &["title"];
        fn into_tuple(self) -> Self::Tuple {
            (self.title,)
        }
        fn from_tuple(tuple: Self::Tuple) -> Self {
            Self { title: tuple.0 }
        }
    }
};

pub mod category_members {
    use super::category;
    use crate::{prelude::macro_derive_collection::*, sqlx_query_builder::OpExpression};

    #[derive(Clone, Default)]
    pub struct title;

    impl Member for title {
        fn name(&self) -> &str {
            "title"
        }
        type CollectionHandler = category;
        type Data = String;
    }

    crate::member_impl_from_row_alias!(title);
    crate::member_impl_debug!(title);
    crate::member_is_unique_filter!(title, false);
    crate::member_impl_expression!(title);

    #[derive(Clone, Default)]
    pub struct id;

    impl Member for id {
        fn name(&self) -> &str {
            "id"
        }
        type CollectionHandler = category;
        type Data = <SingleIncremintalInt<&'static str> as CollectionId>::IdData;
    }

    crate::member_impl_from_row_alias!(id);
    crate::member_impl_debug!(id);
    crate::member_is_unique_filter!(id, true);
    crate::member_impl_expression!(id);
}

// OnMigrate derive for category
const _: () = {
    use crate::prelude::on_migrate_derive::*;
    impl OnMigrate for category {
        type Statements = CreateTable<
            create_table,
            table_as_expression<category>,
            ManyPossible<(
                <category as Collection>::Id,
                col_def_for_collection_member<category_members::title>,
            )>,
        >;
        fn statments(&self) -> Self::Statements {
            CreateTable {
                init: create_table,
                name: table_as_expression(category),
                col_defs: ManyPossible((
                    Collection::id(self).clone(),
                    col_def_for_collection_member(category_members::title),
                )),
            }
        }
    }
};

#[derive(Debug, PartialEq, Eq, Clone, Serialize, Deserialize)]
pub struct Tag {
    pub title: String,
}

const _: () = {
    use crate::prelude::from_row_alias::*;

    impl<'r> FromRowData for tag {
        type RData = Tag;
    }

    impl<'r, R: sqlx::Row> FromRowAlias<'r, R> for tag
    where
        R: Row + 'r,
        String: Type<R::Database> + Decode<'r, R::Database>,
        for<'a> &'a str: ColumnIndex<R>,
    {
        fn two_alias(
            &self,
            row: crate::from_row::RowTwoAliased<'r, R>,
        ) -> Result<Self::RData, crate::from_row::FromRowError>
        where
            R: sqlx::Row,
        {
            Ok(Tag {
                title: row.try_get("title")?,
            })
        }
        fn no_alias(&self, row: &'r R) -> Result<Self::RData, FromRowError> {
            Ok(Tag {
                title: row.try_get("title")?,
            })
        }
        fn pre_alias(&self, row: RowPreAliased<'r, R>) -> Result<Self::RData, FromRowError> {
            Ok(Tag {
                title: row.try_get("title")?,
            })
        }
        fn post_alias(&self, row: RowPostAliased<'r, R>) -> Result<Self::RData, FromRowError> {
            Ok(Tag {
                title: row.try_get("title")?,
            })
        }
    }
};

#[derive(Default, Debug)]
pub struct TagPartial {
    pub title: Update<String>,
}

#[derive(Clone, Default, Debug)]
pub struct tag;

impl Singleton for tag {
    fn singleton() -> &'static Self {
        &tag
    }
}

const _: () = {
    use crate::prelude::macro_derive_collection::*;

    impl Collection for tag {
        fn table_name(&self) -> &str {
            "Tag"
        }
        fn table_name_lower_case(&self) -> &str {
            "tag"
        }

        type InputData = Tag;
        type UpdateData = TagPartial;
        type OutputData = Tag;

        type Id = SingleIncremintalInt<&'static str>;
        fn id(&self) -> Self::Id {
            SingleIncremintalInt("Tag")
        }
    }

    impl HasHandler for Tag {
        type Handler = tag;
    }
    impl HasHandler for TagPartial {
        type Handler = tag;
    }
    impl AsTuple for Tag {
        type Tuple = (String,);
        const NAMES: &'static [&'static str] = &["title"];
        fn into_tuple(self) -> Self::Tuple {
            (self.title,)
        }
        fn from_tuple(tuple: Self::Tuple) -> Self {
            Self { title: tuple.0 }
        }
    }
    impl AsTuple for TagPartial {
        type Tuple = (Update<String>,);
        const NAMES: &'static [&'static str] = &["title"];
        fn into_tuple(self) -> Self::Tuple {
            (self.title,)
        }
        fn from_tuple(tuple: Self::Tuple) -> Self {
            Self { title: tuple.0 }
        }
    }
};

pub mod tag_members {
    use super::tag;
    use crate::{prelude::macro_derive_collection::*, sqlx_query_builder::OpExpression};

    #[derive(Clone, Default)]
    pub struct title;

    impl Member for title {
        fn name(&self) -> &str {
            "title"
        }
        type CollectionHandler = tag;
        type Data = String;
    }

    crate::member_impl_from_row_alias!(title);
    crate::member_impl_debug!(title);
    crate::member_is_unique_filter!(title, false);
    crate::member_impl_expression!(title);

    #[derive(Clone, Default)]
    pub struct id;

    impl Member for id {
        fn name(&self) -> &str {
            "id"
        }
        type CollectionHandler = tag;
        type Data = <SingleIncremintalInt<&'static str> as CollectionId>::IdData;
    }

    crate::member_impl_from_row_alias!(id);
    crate::member_impl_debug!(id);
    crate::member_is_unique_filter!(id, true);
    crate::member_impl_expression!(id);
}

const _: () = {
    use crate::prelude::on_migrate_derive::*;
    impl OnMigrate for tag {
        type Statements = CreateTable<
            create_table,
            table_as_expression<tag>,
            ManyPossible<(
                <tag as Collection>::Id,
                col_def_for_collection_member<tag_members::title>,
            )>,
        >;
        fn statments(&self) -> Self::Statements {
            CreateTable {
                init: create_table,
                name: table_as_expression(tag),
                col_defs: ManyPossible((
                    Collection::id(self).clone(),
                    col_def_for_collection_member(tag_members::title),
                )),
            }
        }
    }
};

const _: () = {
    use crate::extentions::common_expressions::{Aliased, Identifier, MigrateExpression, Scoped};

    impl TableNameExpression for tag {
        type TableNameExpression = &'static str;
        fn table_name_expression(&self) -> Self::TableNameExpression {
            "Tag"
        }
        type LowerCaseTableNameExpression = &'static str;
        fn lower_case_table_name_expression(&self) -> Self::LowerCaseTableNameExpression {
            "tag"
        }
    }

    impl Scoped for tag {
        type Scoped = ScopedCols<'static>;
        fn scoped(&self) -> Self::Scoped {
            ScopedCols {
                table: "Tag",
                cols: <Tag as AsTuple>::NAMES,
            }
        }
    }

    impl Aliased for tag {
        type Aliased = AliasedCols<'static>;
        fn aliased(&self, alias: &'static str) -> Self::Aliased {
            AliasedCols {
                table: "Tag",
                cols: <Tag as AsTuple>::NAMES,
                alias,
            }
        }
        type NumAliased = NumAliasedCols<'static>;
        fn num_aliased(&self, num: usize, alias: &'static str) -> Self::NumAliased {
            NumAliasedCols {
                table: "Tag",
                cols: <Tag as AsTuple>::NAMES,
                num,
                alias,
            }
        }
    }

    impl MembersAndIdAliased for tag {
        type MembersAndIdAliased = AliasedCols<'static>;
        fn members_and_id_aliased(&self, alias: &'static str) -> Self::MembersAndIdAliased {
            AliasedCols {
                table: "Tag",
                cols: &["id", "title"],
                alias,
            }
        }
    }

    impl Identifier for tag {
        type Identifier = &'static [&'static str];
        fn identifier(&self) -> Self::Identifier {
            <Tag as AsTuple>::NAMES
        }
    }

    impl V0OnUpdate for tag {
        type UpdateInput = TagPartial;
        type UpdateExpression = TagPartial;

        fn on_update(self, input: Self::UpdateInput) -> Self::UpdateExpression {
            input
        }
    }

    impl V0OnInsert for tag {
        type InsertInput = Tag;
        type InsertExpression = Tag;

        fn on_insert(self, input: Self::InsertInput) -> Self::InsertExpression {
            input
        }
    }

    impl OnInsert for tag {
        type InsertInput = Tag;
        type InsertExpression = Tag;

        fn on_insert(&self, input: Self::InsertInput) -> Self::InsertExpression {
            input
        }
    }

    impl OnUpdate for tag {
        type UpdateInput = TagPartial;
        type UpdateExpression = TagPartial;

        fn on_update(&self, input: Self::UpdateInput) -> Self::UpdateExpression {
            input
        }
    }

    impl IsOpExpression for Tag {
        fn is_op(&self) -> bool {
            true
        }
    }

    impl<'q, S: DatabaseExt> ManyExpressions<'q, S> for Tag
    where
        String: Type<S> + Encode<'q, S>,
    {
        fn expression(
            self,
            start: &'static str,
            join: &'static str,
            ctx: &mut StatementBuilder<'q, S>,
        ) where
            S: DatabaseExt,
        {
            ctx.syntax(start);
            ctx.bind(self.title);
        }
    }

    impl IsOpExpression for TagPartial {
        fn is_op(&self) -> bool {
            matches!(self.title, Update::Set(_))
        }
    }
    impl<'q, S> ManyExpressions<'q, S> for TagPartial
    where
        S: Database,
        String: Type<S> + Encode<'q, S>,
    {
        fn expression(
            self,
            start: &'static str,
            join: &'static str,
            ctx: &mut StatementBuilder<'q, S>,
        ) where
            S: DatabaseExt,
        {
            if let Update::Set(title) = self.title {
                ctx.syntax(start);
                ctx.sanitize("title");
                ctx.syntax(" = ");
                ctx.bind(title);
            }
        }
    }

    impl MigrateExpression for tag {
        type MigrateExpression = (MigratingCol<&'static str, String>,);
        fn migrate_expression(&self) -> Self::MigrateExpression {
            (MigratingCol {
                col: "title",
                phantom: std::marker::PhantomData,
            },)
        }
    }
};

mod impl_link {
    use std::convert::Infallible;

    use super::{category, todo};
    use crate::links::relation_optional_to_many::OptionalToMany;
    use crate::links::{DefaultRelationKey, Link};

    impl Link<todo> for category {
        type Spec = OptionalToMany<DefaultRelationKey, todo, category>;
        fn spec(self) -> Self::Spec {
            OptionalToMany {
                fk_unique_id: DefaultRelationKey,
                from: todo,
                to: category,
            }
        }
    }
}

#[macro_export]
macro_rules! member_impl_expression {
    ($member:ident) => {
        impl $crate::extentions::common_expressions::Identifier for $member {
            type Identifier = &'static str;
            fn identifier(&self) -> Self::Identifier {
                stringify!($member)
            }
        }
        impl $crate::extentions::common_expressions::V0OnInsert for $member {
            type InsertInput = <Self as $crate::collections::Member>::Data;
            type InsertExpression = $crate::extentions::named_bind::NamedBind<
                &'static str,
                Self,
                <Self as $crate::collections::Member>::Data,
            >;

            fn on_insert(self, input: Self::InsertInput) -> Self::InsertExpression {
                $crate::extentions::named_bind::NamedBind {
                    table: $crate::extentions::common_expressions::TableNameExpression::table_name_expression(&<Self as $crate::collections::Member>::CollectionHandler::default()),
                    name: $member,
                    value: input,
                }
            }
        }

        impl $crate::extentions::common_expressions::OnInsert for $member {
            type InsertInput = <Self as $crate::collections::Member>::Data;
            type InsertExpression = <Self as $crate::collections::Member>::Data;

            fn on_insert(&self, input: Self::InsertInput) -> Self::InsertExpression {
                input
            }
        }

        impl $crate::extentions::common_expressions::Scoped for $member {
            type Scoped =
                $crate::expressions::single_col_expressions::ScopedCol<&'static str, Self>;
            fn scoped(&self) -> Self::Scoped {
                $crate::expressions::single_col_expressions::ScopedCol {
                    table:
                    $crate::extentions::common_expressions::TableNameExpression::table_name_expression(
                        &<Self as $crate::collections::Member>::CollectionHandler::default()
                    ),
                    col: $member,
                }
            }
        }

        impl $crate::extentions::common_expressions::V0OnUpdate for $member {
            type UpdateInput = <Self as $crate::collections::Member>::Data;
            type UpdateExpression = <Self as $crate::collections::Member>::Data;

            fn on_update(self, input: Self::UpdateInput) -> Self::UpdateExpression {
                input
            }
        }

        impl $crate::extentions::common_expressions::OnUpdate for $member {
            type UpdateInput = <Self as $crate::collections::Member>::Data;
            type UpdateExpression = <Self as $crate::collections::Member>::Data;

            fn on_update(&self, input: Self::UpdateInput) -> Self::UpdateExpression {
                input
            }
        }

        impl $crate::query_builder::OpExpression for $member {}
        impl<'q, S> $crate::query_builder::Expression<'q, S> for $member
        where
            S: $crate::database_extention::DatabaseExt,
        {
            fn expression(self, ctx: &mut $crate::query_builder::StatementBuilder<'q, S>) {
                ctx.sanitize(self.name());
            }
        }
    };
}

#[macro_export]
macro_rules! member_is_unique_filter {
    ($member:ident, $is_unique:expr) => {
        impl $member {
            const fn is_unique(&self) -> bool {
                $is_unique
            }
        }
    };
}

#[macro_export]
macro_rules! member_impl_from_row_alias {
    ($member:ident) => {
        impl<'r> $crate::from_row::FromRowData for $member {
            type RData = <Self as $crate::collections::Member>::Data;
        }
        impl<'r, R: ::sqlx::Row> $crate::from_row::FromRowAlias<'r, R> for $member
        where
            R: ::sqlx::Row + 'r,
            <Self as $crate::collections::Member>::Data:
                ::sqlx::Type<R::Database> + ::sqlx::Decode<'r, R::Database>,
            for<'a> &'a str: ::sqlx::ColumnIndex<R>,
        {
            fn no_alias(&self, row: &'r R) -> Result<Self::RData, $crate::from_row::FromRowError> {
                Ok(row.try_get(self.name())?)
            }
            fn pre_alias(
                &self,
                row: $crate::from_row::RowPreAliased<'r, R>,
            ) -> Result<Self::RData, $crate::from_row::FromRowError> {
                Ok(row.try_get(self.name())?)
            }
            fn post_alias(
                &self,
                row: $crate::from_row::RowPostAliased<'r, R>,
            ) -> Result<Self::RData, $crate::from_row::FromRowError> {
                Ok(row.try_get(self.name())?)
            }
            fn two_alias(
                &self,
                row: $crate::from_row::RowTwoAliased<'r, R>,
            ) -> Result<Self::RData, $crate::from_row::FromRowError> {
                Ok(row.try_get(self.name())?)
            }
        }
    };
}

#[macro_export]
macro_rules! member_impl_debug {
    ($member:ident) => {
        impl<'r> ::core::fmt::Debug for $member {
            fn fmt(&self, f: &mut ::core::fmt::Formatter<'_>) -> ::core::fmt::Result {
                write!(
                    f,
                    "{}.{}",
                    Collection::table_name(
                        <<Self as $crate::collections::Member>::CollectionHandler
                            as $crate::singleton::Singleton
                            > ::singleton()
                    ),
                    self.name()
                )
            }
        }
    };
}
