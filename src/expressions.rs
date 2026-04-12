#![allow(non_camel_case_types)]
use crate::collections::{CollectionBasic, Member, MemberBasic};
use crate::database_extention::DatabaseExt;
use crate::prelude::sql::AliasAndExpr;
use crate::query_builder::syntax::{comma_join, empty, equal_join, space_join};
use crate::query_builder::{Expression, ManyExpressions, OpExpression, QueryBuilder, SqlSyntax};
use sqlx::{Database, Encode, Type, TypeInfo};
use std::marker::PhantomData;
use std::ops::Not;

pub struct member_as_expression<T>(pub T);
impl<T> OpExpression for member_as_expression<T> {}
impl<'q, S, T> Expression<'q, S> for member_as_expression<T>
where
    T: MemberBasic,
    T: 'q,
{
    fn expression(self, ctx: &mut QueryBuilder<'q, S>)
    where
        S: DatabaseExt,
    {
        ctx.sanitize(self.0.name());
    }
}

impl<T> member_as_expression<T> {
    pub fn eq<V>(self, eq: V) -> col_eq<Self, V> {
        col_eq { col: self, eq }
    }
}

pub struct table_as_expression<T>(pub T);
impl<T> OpExpression for table_as_expression<T> {}
impl<'q, S, T> Expression<'q, S> for table_as_expression<T>
where
    T: CollectionBasic,
    T: 'q,
{
    fn expression(self, ctx: &mut QueryBuilder<'q, S>)
    where
        S: DatabaseExt,
    {
        ctx.sanitize(self.0.table_name());
    }
}

pub struct sqlx_type<T>(PhantomData<T>);

impl<T> Default for sqlx_type<T> {
    fn default() -> Self {
        sqlx_type(PhantomData)
    }
}

impl<T> OpExpression for sqlx_type<T> {}

impl<'q, S, T> Expression<'q, S> for sqlx_type<T>
where
    S: Database,
    T: Type<S> + 'q,
{
    fn expression(self, ctx: &mut QueryBuilder<'q, S>)
    where
        S: DatabaseExt,
    {
        ctx.syntax(&sqlx_type(PhantomData::<(S, T)>));
    }
}

impl<T> Clone for sqlx_type<T> {
    fn clone(&self) -> Self {
        sqlx_type(PhantomData)
    }
}

impl<S, T> SqlSyntax for sqlx_type<(S, T)>
where
    S: Database,
    T: Type<S>,
{
    fn to_sql(&self, stmt: &mut String) {
        stmt.push_str(T::type_info().name());
    }
}

mod is_null {
    pub trait IsNull {
        fn is_null() -> bool;
    }

    impl<T> IsNull for Option<T> {
        fn is_null() -> bool {
            true
        }
    }

    #[cfg(feature = "nightly_rust")]
    impl<T> IsNull for T {
        default fn is_null() -> bool {
            false
        }
    }

    #[cfg(not(feature = "nightly_rust"))]
    mod impl_is_null_no_spectialization {
        use std::collections::HashMap;

        use super::IsNull;

        macro_rules! impl_no_gens {
            ($($ident:ident)*) => {
                $(impl IsNull for $ident {
                    fn is_null() -> bool {
                        false
                    }
                })*
            };
        }

        impl_no_gens!(i32 i64 bool char String);

        macro_rules! impl_gens {
            ($ident:ident [$($gens:ident $(:$wheres:tt)?),*]) => {
                impl<$($gens,)*> IsNull for $ident<$($gens,)*>
                where $($gens:Sized $(+$wheres)? ),*
                {
                    fn is_null() -> bool {
                        false
                    }
                }
            };
        }

        impl_gens!(Vec[T]);
        impl_gens!(HashMap[K,V,S]);
    }
}

pub struct col_def<Name, Type, Constraints> {
    pub name: Name,
    pub ty: Type,
    pub constraints: Constraints,
}

impl<Name, Type, Constraints> OpExpression for col_def<Name, Type, Constraints> {}

impl<'q, S, Name, Type, Constraints> Expression<'q, S> for col_def<Name, Type, Constraints>
where
    S: Database,
    Name: AsRef<str> + 'q,
    Type: Expression<'q, S> + 'q,
    Constraints: ManyExpressions<'q, S> + 'q,
{
    fn expression(self, ctx: &mut QueryBuilder<'q, S>)
    where
        S: DatabaseExt,
    {
        ctx.sanitize(self.name.as_ref());
        ctx.syntax(&space_join);
        self.ty.expression(ctx);
        self.constraints.expression(&empty, &comma_join, ctx);
    }
}

pub struct col_def_for_collection_member<T>(pub T);

impl<T> OpExpression for col_def_for_collection_member<T> {}

impl<'q, S, T> Expression<'q, S> for col_def_for_collection_member<T>
where
    S: Database,
    T: Member + 'q,
    T::Data: sqlx::Type<S>,
    T::Data: is_null::IsNull,
{
    fn expression(self, ctx: &mut QueryBuilder<'q, S>)
    where
        S: DatabaseExt,
    {
        ctx.sanitize(self.0.name());
        ctx.syntax(&space_join);
        ctx.syntax(&sqlx_type(PhantomData::<(S, T::Data)>));
        let s = <T::Data as is_null::IsNull>::is_null();
        if s.not() {
            ctx.syntax(&" NOT NULL");
        }
    }
}

pub struct col<T>(pub T);

impl<Column> col<Column> {
    pub fn pre_alias<Alias>(self, alias: Alias) -> PreAlias<col<Column>, Alias> {
        PreAlias { on: self, alias }
    }
    pub fn eq<Eq>(self, eq: Eq) -> col_eq<Self, Eq> {
        col_eq { col: self, eq }
    }
}

impl<T> OpExpression for col<T> {}

impl<'q, S, T> Expression<'q, S> for col<T>
where
    T: 'q,
    T: AsRef<str>,
{
    fn expression(self, ctx: &mut QueryBuilder<'q, S>)
    where
        S: DatabaseExt,
    {
        ctx.sanitize(self.0.as_ref());
    }
}

pub struct pre_alias<T>(pub T, pub &'static str);

impl<T> OpExpression for pre_alias<T> {}

impl<'q, S, T: 'q + Expression<'q, S>> Expression<'q, S> for pre_alias<T> {
    fn expression(self, ctx: &mut QueryBuilder<'q, S>)
    where
        S: DatabaseExt,
    {
        self.0.expression(ctx);
        ctx.syntax(&" AS ");
        todo!()
    }
}

pub struct left_join {
    pub ft: String,
    pub fc: String,
    pub lt: String,
    pub lc: String,
}

impl OpExpression for left_join {}

impl<'a, S> Expression<'a, S> for left_join {
    fn expression(self, ctx: &mut QueryBuilder<'a, S>)
    where
        S: DatabaseExt,
    {
        ctx.syntax(&"LEFT JOIN ");
        ctx.sanitize(self.ft.as_str());
        ctx.syntax(&" ON ");
        ctx.sanitize(self.lt.as_str());
        ctx.syntax(&".");
        ctx.sanitize(self.lc.as_str());
        ctx.syntax(&" = ");
        ctx.sanitize(self.ft.as_str());
        ctx.syntax(&".");
        ctx.sanitize(self.fc.as_str());
    }
}

// redundant: use prealias
// pub struct local_col<T>(pub T);
// impl<'q, S, T> Expression<'q, S> for local_col<T>
// where
//     T: SqlSanitize<S>,
// {
//     fn expression(self, ctx: &mut QueryBuilder<'q, S>)
//     where
//         S: DatabaseExt,
//     {
//         tracing::warn!(
//             "todo: have better implementation for local_col 1.create col(..).alias(..) 2. handle namming conflicts"
//         );
//         ctx.sanitize(self.0);
//         ctx.syntax(".");
//         ctx.sanitize("id");
//         ctx.syntax(" AS ");
//         ctx.sanitize("local_id");
//     }
// }

pub struct col_eq<Col, Eq> {
    pub col: Col,
    pub eq: Eq,
}

impl<A, E> AliasAndExpr<A, E> for col_eq<A, E> {
    fn aliase_and_expr(alias: A, expr: E) -> Self {
        col_eq {
            col: alias,
            eq: expr,
        }
    }
}

impl<Col, Eq> OpExpression for col_eq<Col, Eq> {}

impl<'q, S, Col, Eq> Expression<'q, S> for col_eq<Col, Eq>
where
    S: DatabaseExt,
    Eq: 'q + Encode<'q, S> + Type<S>,
    Col: Expression<'q, S> + 'q,
{
    fn expression(self, arg: &mut QueryBuilder<'q, S>) {
        self.col.expression(arg);
        arg.syntax(&equal_join);
        arg.bind(self.eq);
    }
}

pub struct table<T>(pub T);

impl<T> OpExpression for table<T> {}

impl<'q, S, Table> Expression<'q, S> for table<Table>
where
    S: DatabaseExt,
    Table: AsRef<str> + 'q,
{
    fn expression(self, arg: &mut QueryBuilder<'q, S>) {
        arg.sanitize(self.0.as_ref());
    }
}

impl<Table> table<Table> {
    pub fn col<Column>(self, column: Column) -> scoped_column<Table, Column> {
        scoped_column {
            table: self.0,
            column,
        }
    }
}

pub struct scoped_column<Table, Column> {
    pub table: Table,
    pub column: Column,
}

impl<Table, Column> scoped_column<Table, Column> {
    pub fn eq<V>(self, eq: V) -> col_eq<Self, V> {
        col_eq { col: self, eq }
    }
    pub fn pre_alias<Alias>(self, alias: Alias) -> PreAlias<scoped_column<Table, Column>, Alias> {
        PreAlias { on: self, alias }
    }
}

impl<Table, Column> OpExpression for scoped_column<Table, Column> {}

impl<'q, S, Column, Table> Expression<'q, S> for scoped_column<Table, Column>
where
    S: DatabaseExt,
    Table: AsRef<str> + 'q,
    Column: AsRef<str> + 'q,
{
    fn expression(self, arg: &mut QueryBuilder<'q, S>) {
        arg.sanitize(self.table.as_ref());
        arg.syntax(&".");
        arg.sanitize(self.column.as_ref());
    }
}

pub struct PreAlias<On, Alias> {
    pub on: On,
    pub alias: Alias,
}

impl<On, Alias> OpExpression for PreAlias<On, Alias> {}

// todo: replace &str with generics
impl<'q, S, T, C, A> Expression<'q, S> for PreAlias<scoped_column<T, C>, A>
where
    Self: 'q,
    S: DatabaseExt,
    T: AsRef<str> + 'q,
    C: AsRef<str> + 'q,
    A: AsRef<str> + 'q,
{
    fn expression(self, arg: &mut QueryBuilder<'q, S>) {
        let alias = format!(
            "{}{}{}",
            self.alias.as_ref(),
            self.on.table.as_ref(),
            self.on.column.as_ref()
        );
        arg.sanitize(self.on.table.as_ref());
        arg.syntax(&".");
        arg.sanitize(self.on.column.as_ref());
        arg.syntax(&" AS ");
        arg.sanitize(alias.as_str());
    }
}

// impl<'q, S> Expression<'q, S> for PreAlias<scoped_column<&str, &str>, &str>
// where
//     Self: 'q,
//     S: DatabaseExt,
// {
//     fn expression(self, arg: &mut QueryBuilder<'q, S>) {
//         let alias = format!("{}{}", self.alias, self.on.column);
//         arg.sanitize(self.on.table);
//         arg.syntax(".");
//         arg.sanitize(self.on.column);
//         arg.syntax(" AS ");
//         arg.sanitize(alias);
//     }
// }

impl<'q, S> Expression<'q, S> for PreAlias<col<&str>, &str>
where
    Self: 'q,
    S: DatabaseExt,
{
    fn expression(self, arg: &mut QueryBuilder<'q, S>) {
        let alias = format!("{}{}", self.alias, self.on.0);
        arg.sanitize(self.on.0);
        arg.syntax(&" AS ");
        arg.sanitize(alias.as_str());
    }
}

pub struct id_constraint<Id, Constraint>(pub Id, pub Constraint);

impl<Id, Constraint> OpExpression for id_constraint<Id, Constraint> {}

mod imp_id_constraint_for_sqlite {
    use sqlx::Sqlite;

    use crate::{
        database_extention::DatabaseExt,
        query_builder::{Expression, QueryBuilder, syntax::space_join},
    };

    impl<'q, Id, Constraint> Expression<'q, Sqlite> for super::id_constraint<Id, Constraint>
    where
        Id: AsRef<str> + 'q,
        Constraint: Expression<'q, Sqlite>,
    {
        fn expression(self, ctx: &mut QueryBuilder<'q, Sqlite>)
        where
            Sqlite: DatabaseExt,
        {
            ctx.sanitize(self.0.as_ref());
            ctx.syntax(&space_join);
            self.1.expression(ctx);
        }
    }
}

pub struct foriegn_key<Ons> {
    pub references_table: String,
    pub references_col: String,
    pub ons: Ons,
}

impl<Ons> OpExpression for foriegn_key<Ons> {}

mod imp_foriegn_key_for_sqlite {
    use sqlx::Sqlite;

    use crate::{
        database_extention::DatabaseExt,
        query_builder::{
            Expression, ManyExpressions, QueryBuilder,
            syntax::{close_paranthesis, comma_join, open_paranthesis, space_join},
        },
    };

    impl<'q, Ons> Expression<'q, Sqlite> for super::foriegn_key<Ons>
    where
        Ons: ManyExpressions<'q, Sqlite> + 'q,
    {
        fn expression(self, ctx: &mut QueryBuilder<'q, Sqlite>)
        where
            Sqlite: DatabaseExt,
        {
            ctx.syntax(&" REFERENCES ");

            ctx.sanitize(self.references_table.as_str());
            ctx.syntax(&open_paranthesis);
            ctx.sanitize(self.references_col.as_str());
            ctx.syntax(&close_paranthesis);
            self.ons.expression(&space_join, &comma_join, ctx);
        }
    }
}

pub struct on_delete_set_null;

impl OpExpression for on_delete_set_null {}

mod imp_on_delete_set_null_for_sqlite {
    use sqlx::Sqlite;

    use crate::{
        database_extention::DatabaseExt,
        query_builder::{Expression, QueryBuilder},
    };

    impl<'q> Expression<'q, Sqlite> for super::on_delete_set_null {
        fn expression(self, ctx: &mut QueryBuilder<'q, Sqlite>)
        where
            Sqlite: DatabaseExt,
        {
            ctx.syntax(&"ON DELETE SET NULL");
        }
    }
}
