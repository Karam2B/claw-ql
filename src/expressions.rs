#![allow(non_camel_case_types)]
use crate::collections::{CollectionBasic, Member};
use crate::database_extention::DatabaseExt;
use crate::prelude::sql::AliasAndExpr;
use crate::query_builder::{Expression, ManyExpressions, OpExpression, StatementBuilder};
use sqlx::{Database, Encode, Type};
use std::marker::PhantomData;
use std::ops::Not;

pub struct ColAs<T, C> {
    pub table: T,
    pub column: C,
    pub _as: &'static str,
}

impl<T, C> OpExpression for ColAs<T, C> {}
impl<'q, T, C, S> Expression<'q, S> for ColAs<T, C>
where
    S: DatabaseExt,
    C: AsRef<str> + 'q,
    T: AsRef<str> + 'q,
{
    fn expression(self, arg: &mut StatementBuilder<'q, S>)
    where
        S: DatabaseExt,
    {
        arg.sanitize(self.table.as_ref());
        arg.syntax(".");
        arg.sanitize(self.column.as_ref());
        arg.syntax(" AS ");
        arg.sanitize(self._as);
    }
}

pub struct member_as_expression<T>(pub T);
impl<T> OpExpression for member_as_expression<T> {}
impl<'q, S, T> Expression<'q, S> for member_as_expression<T>
where
    T: Member,
    T: 'q,
{
    fn expression(self, ctx: &mut StatementBuilder<'q, S>)
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
    fn expression(self, ctx: &mut StatementBuilder<'q, S>)
    where
        S: DatabaseExt,
    {
        ctx.sanitize(self.0.table_name());
    }
}

pub mod single_col_expressions {
    use std::marker::PhantomData;

    use crate::{
        database_extention::DatabaseExt,
        query_builder::{
            Expression, IsOpExpression, OpExpression, PossibleExpression, StatementBuilder,
        },
        update_mod::Update,
    };

    pub struct ScopedCol<T, C> {
        pub table: T,
        pub col: C,
    }

    impl<T, C> OpExpression for ScopedCol<T, C> {}
    impl<'q, T, C, S> Expression<'q, S> for ScopedCol<T, C>
    where
        T: Expression<'q, S>,
        C: Expression<'q, S>,
    {
        fn expression(self, ctx: &mut StatementBuilder<'q, S>)
        where
            S: DatabaseExt,
        {
            self.table.expression(ctx);
            ctx.syntax(".");
            self.col.expression(ctx);
        }
    }

    pub struct AliasedCol<T, C, A> {
        pub table: T,
        pub col: C,
        pub alias: A,
    }

    impl<T, C, A> OpExpression for AliasedCol<T, C, A> {}
    impl<'q, T, C, A, S> Expression<'q, S> for AliasedCol<T, C, A>
    where
        T: Expression<'q, S>,
        C: Expression<'q, S>,
        A: Expression<'q, S>,
    {
        fn expression(self, ctx: &mut StatementBuilder<'q, S>)
        where
            S: DatabaseExt,
        {
            self.table.expression(ctx);
            ctx.syntax(&".");
            self.col.expression(ctx);
            ctx.syntax(&" AS ");
            self.alias.expression(ctx);
        }
    }

    pub struct UpdatingCol<C, T> {
        pub col: C,
        pub set: Update<T>,
    }

    impl<C, T> IsOpExpression for UpdatingCol<C, T> {
        fn is_op(&self) -> bool {
            matches!(self.set, Update::Set(_))
        }
    }
    impl<'a, C, T, S> PossibleExpression<'a, S> for UpdatingCol<C, T>
    where
        S: DatabaseExt,
        T: sqlx::Type<S> + sqlx::Encode<'a, S> + 'a,
        C: Expression<'a, S>,
    {
        fn expression(self, ctx: &mut StatementBuilder<'a, S>)
        where
            S: DatabaseExt,
        {
            match self.set {
                Update::Set(value) => {
                    self.col.expression(ctx);
                    ctx.syntax(&" = ");
                    ctx.bind(value);
                }
                Update::Keep => {
                    // do nothing
                }
            }
        }
        fn expression_starting(self, start: &'static str, ctx: &mut StatementBuilder<'a, S>)
        where
            S: DatabaseExt,
        {
            match self.set {
                Update::Set(value) => {
                    ctx.syntax(start);
                    self.col.expression(ctx);
                    ctx.syntax(&" = ");
                    ctx.bind(value);
                }
                Update::Keep => {
                    // do nothing
                }
            }
        }
    }

    pub struct MigratingCol<C, T> {
        pub col: C,
        pub phantom: PhantomData<T>,
    }

    impl<C, T> OpExpression for MigratingCol<C, T> {}
    impl<'a, C, T, S> Expression<'a, S> for MigratingCol<C, T>
    where
        S: DatabaseExt,
        T: sqlx::Type<S> + sqlx::Encode<'a, S> + 'a,
        C: AsRef<str> + 'a,
    {
        fn expression(self, ctx: &mut StatementBuilder<'a, S>)
        where
            S: DatabaseExt,
        {
            ctx.sanitize(self.col.as_ref());
            ctx.syntax(&" ");
            ctx.type_as_syntax::<T>();
        }
    }
}

pub mod multi_col_expressions_stack_heavy {
    use crate::{
        database_extention::DatabaseExt,
        query_builder::{IsOpExpression, ManyExpressions, StatementBuilder},
    };

    pub struct ScopedCols<'q> {
        pub table: &'q str,
        pub cols: &'q [&'q str],
    }

    impl IsOpExpression for ScopedCols<'_> {
        fn is_op(&self) -> bool {
            self.cols.len() != 0
        }
    }
    impl<'q, S> ManyExpressions<'q, S> for ScopedCols<'q>
    where
        S: DatabaseExt,
    {
        fn expression(
            self,
            start: &'static str,
            join: &'static str,
            ctx: &mut StatementBuilder<'q, S>,
        ) where
            S: DatabaseExt,
        {
            let len = self.cols.len();
            if len == 0 {
                return;
            }
            ctx.syntax(start);
            for (i, col) in self.cols.iter().enumerate() {
                ctx.sanitize(self.table);
                ctx.syntax(&".");
                ctx.sanitize(col);
                if i < len - 1 {
                    ctx.syntax(join);
                }
            }
        }
    }

    pub struct AliasedCols<'q> {
        pub table: &'q str,
        pub cols: &'q [&'q str],
        pub alias: &'q str,
    }

    impl IsOpExpression for AliasedCols<'_> {
        fn is_op(&self) -> bool {
            self.cols.len() != 0
        }
    }
    impl<'q, S> ManyExpressions<'q, S> for AliasedCols<'static>
    where
        S: DatabaseExt,
    {
        fn expression(
            self,
            start: &'static str,
            join: &'static str,
            ctx: &mut StatementBuilder<'q, S>,
        ) where
            S: DatabaseExt,
        {
            let len = self.cols.len();
            if len == 0 {
                return;
            }
            ctx.syntax(start);
            // panic!("problem, stmt {:?}", self.cols);
            for (i, item) in self.cols.into_iter().enumerate() {
                ctx.sanitize(self.table);
                ctx.syntax(&".");
                ctx.sanitize(item);
                ctx.syntax(&" AS ");
                ctx.sanitize_strings((self.alias, *item));
                if i < len - 1 {
                    ctx.syntax(join);
                }
            }
            println!("problem, stmt {:?}", ctx.stmt());
        }
    }

    pub struct NumAliasedCols<'q> {
        pub table: &'q str,
        pub cols: &'q [&'q str],
        pub num: usize,
        pub alias: &'q str,
    }

    impl IsOpExpression for NumAliasedCols<'_> {
        fn is_op(&self) -> bool {
            self.cols.len() != 0
        }
    }
    impl<'q, S> ManyExpressions<'q, S> for NumAliasedCols<'static>
    where
        S: DatabaseExt,
    {
        fn expression(
            self,
            start: &'static str,
            join: &'static str,
            ctx: &mut StatementBuilder<'q, S>,
        ) {
            let len = self.cols.len();
            if len == 0 {
                return;
            }
            ctx.syntax(start);
            for (i, item) in self.cols.into_iter().enumerate() {
                ctx.sanitize(self.table);
                ctx.syntax(&".");
                ctx.sanitize(item);
                ctx.syntax(&" AS ");
                ctx.sanitize_strings((self.alias, self.num, *item));
                if i < len - 1 {
                    ctx.syntax(join);
                }
            }
        }
    }
}

pub mod is_null {
    pub trait IsNull {
        fn is_null() -> bool;
    }

    impl<T> IsNull for Option<T> {
        fn is_null() -> bool {
            true
        }
    }

    #[cfg(feature = "nightly_rust_specialization")]
    impl<T> IsNull for T {
        default fn is_null() -> bool {
            false
        }
    }

    #[cfg(not(feature = "nightly_rust_specialization"))]
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

        impl_gens!(HashMap[K,V,S]);
    }
}

pub struct ColumnDefinition<Name, Type, Constraints> {
    pub name: Name,
    pub ty: PhantomData<Type>,
    pub constraints: Constraints,
}

impl<Name, Type, Constraints> OpExpression for ColumnDefinition<Name, Type, Constraints> {}

impl<'q, S, Name, Type, Constraints> Expression<'q, S> for ColumnDefinition<Name, Type, Constraints>
where
    S: Database,
    Name: Expression<'q, S>,
    Type: is_null::IsNull + sqlx::Type<S> + 'q,
    Constraints: ManyExpressions<'q, S> + 'q,
{
    fn expression(self, ctx: &mut StatementBuilder<'q, S>)
    where
        S: DatabaseExt,
    {
        self.name.expression(ctx);
        ctx.syntax(" ");

        ctx.type_as_syntax::<Type>();
        if <Type as is_null::IsNull>::is_null().not() {
            ctx.syntax(" NOT NULL");
        }
        self.constraints.expression(" ", ", ", ctx);
    }
}

pub struct col_def_for_collection_member<T>(pub T);

impl<T> OpExpression for col_def_for_collection_member<T> {}

impl<'q, S, T> Expression<'q, S> for col_def_for_collection_member<T>
where
    S: Database,
    T: Member + 'q,
    T::Data: sqlx::Type<S> + 'static,
    T::Data: is_null::IsNull,
{
    fn expression(self, ctx: &mut StatementBuilder<'q, S>)
    where
        S: DatabaseExt,
    {
        ctx.sanitize(self.0.name());
        ctx.syntax(" ");
        ctx.type_as_syntax::<T::Data>();
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
    fn expression(self, ctx: &mut StatementBuilder<'q, S>)
    where
        S: DatabaseExt,
    {
        ctx.sanitize(self.0.as_ref());
    }
}

pub struct pre_alias<T>(pub T, pub &'static str);

impl<T> OpExpression for pre_alias<T> {}

impl<'q, S, T: 'q + Expression<'q, S>> Expression<'q, S> for pre_alias<T> {
    fn expression(self, ctx: &mut StatementBuilder<'q, S>)
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
    fn expression(self, ctx: &mut StatementBuilder<'a, S>)
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

#[derive(Clone, Debug)]
#[cfg_attr(feature = "serde", derive(serde::Deserialize))]
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
    fn expression(self, arg: &mut StatementBuilder<'q, S>) {
        self.col.expression(arg);
        arg.syntax(" = ");
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
    fn expression(self, arg: &mut StatementBuilder<'q, S>) {
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
    fn expression(self, arg: &mut StatementBuilder<'q, S>) {
        arg.sanitize(self.table.as_ref());
        arg.syntax(&".");
        arg.sanitize(self.column.as_ref());
    }
}

// to deprecate: aliasing is only relevant to to links and should use scoped_column_with_num instead
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
    fn expression(self, arg: &mut StatementBuilder<'q, S>) {
        let alias = format!(
            "{}{}{}",
            self.alias.as_ref(),
            self.on.table.as_ref(),
            self.on.column.as_ref(),
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
    fn expression(self, arg: &mut StatementBuilder<'q, S>) {
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
        query_builder::{Expression, StatementBuilder},
    };

    impl<'q, Id, Constraint> Expression<'q, Sqlite> for super::id_constraint<Id, Constraint>
    where
        Id: AsRef<str> + 'q,
        Constraint: Expression<'q, Sqlite>,
    {
        fn expression(self, ctx: &mut StatementBuilder<'q, Sqlite>)
        where
            Sqlite: DatabaseExt,
        {
            ctx.sanitize(self.0.as_ref());
            ctx.syntax(" ");
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
            Expression, ManyExpressions, StatementBuilder,
            essential_syntax::{CLOSE_PARANTHESIS, OPEN_PARANTHESIS},
        },
    };

    impl<'q, Ons> Expression<'q, Sqlite> for super::foriegn_key<Ons>
    where
        Ons: ManyExpressions<'q, Sqlite> + 'q,
    {
        fn expression(self, ctx: &mut StatementBuilder<'q, Sqlite>)
        where
            Sqlite: DatabaseExt,
        {
            ctx.syntax(&" REFERENCES ");

            ctx.sanitize(self.references_table.as_str());
            ctx.syntax(OPEN_PARANTHESIS);
            ctx.sanitize(self.references_col.as_str());
            ctx.syntax(CLOSE_PARANTHESIS);
            self.ons.expression(" ", ", ", ctx);
        }
    }
}

pub struct on_delete_set_null;

impl OpExpression for on_delete_set_null {}

mod imp_on_delete_set_null_for_sqlite {
    use sqlx::Sqlite;

    use crate::{
        database_extention::DatabaseExt,
        query_builder::{Expression, StatementBuilder},
    };

    impl<'q> Expression<'q, Sqlite> for super::on_delete_set_null {
        fn expression(self, ctx: &mut StatementBuilder<'q, Sqlite>)
        where
            Sqlite: DatabaseExt,
        {
            ctx.syntax(&"ON DELETE SET NULL");
        }
    }
}

pub mod larger_than_or_equal {
    use crate::{
        database_extention::DatabaseExt,
        query_builder::essential_syntax::{CLOSE_PARANTHESIS, OPEN_PARANTHESIS},
        query_builder::{Expression, ManyExpressions, OpExpression, StatementBuilder},
    };

    pub struct LargerThanOrEqual<Identifiers, Values> {
        pub id: Identifiers,
        pub values: Values,
    }

    impl<I, V> OpExpression for LargerThanOrEqual<I, V> {}
    impl<'q, S, I, V> Expression<'q, S> for LargerThanOrEqual<I, V>
    where
        I: ManyExpressions<'q, S> + 'q,
        V: ManyExpressions<'q, S> + 'q,
        S: DatabaseExt,
    {
        fn expression(self, ctx: &mut StatementBuilder<'q, S>) {
            ctx.syntax(OPEN_PARANTHESIS);
            self.id.expression("", ", ", ctx);
            ctx.syntax(CLOSE_PARANTHESIS);
            ctx.syntax(" >= ");
            ctx.syntax(OPEN_PARANTHESIS);
            self.values.expression("", ", ", ctx);
            ctx.syntax(CLOSE_PARANTHESIS);
        }
    }
}

pub mod standard_naming_conventions {
    use crate::{
        database_extention::DatabaseExt,
        query_builder::{Expression, OpExpression, StatementBuilder},
    };

    // represent a standard way to name a foreign key
    pub struct ForeignKeyName<Key, To> {
        pub key: Key,
        pub to: To,
    }

    impl<Key, T> OpExpression for ForeignKeyName<Key, T> {}
    impl<'q, S, Key, T> Expression<'q, S> for ForeignKeyName<Key, T>
    where
        T: AsRef<str> + 'q,
        Key: AsRef<str> + 'q,
    {
        fn expression(self, ctx: &mut StatementBuilder<'q, S>)
        where
            S: DatabaseExt,
        {
            ctx.sanitize_strings(("fk_", self.to.as_ref(), self.key.as_ref()));
        }
    }

    // represent a standard way to name a conjuction table
    pub struct ConjuctionTableName<T1, T2, Key> {
        pub first: T1,
        pub second: T2,
        pub key: Key,
    }

    impl<T1, T2, Key> OpExpression for ConjuctionTableName<T1, T2, Key> {}
    impl<'q, S, T1, T2, Key> Expression<'q, S> for ConjuctionTableName<T1, T2, Key>
    where
        S: DatabaseExt,
        T1: AsRef<str> + 'q,
        T2: AsRef<str> + 'q,
        Key: AsRef<str> + 'q,
    {
        fn expression(self, ctx: &mut StatementBuilder<'q, S>) {
            ctx.sanitize_strings((
                "ct_",
                self.first.as_ref(),
                self.second.as_ref(),
                self.key.as_ref(),
            ));
        }
    }
}
