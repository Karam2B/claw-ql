use crate::collections::{CollectionBasic, Member, MemberBasic};
use crate::{DatabaseExt, Expression, OpExpression, SqlSanitize, ZeroOrMoreExpressions};
use crate::{QueryBuilder, SqlSyntax};
use sqlx::{Database, Encode, Type, TypeInfo};
use std::fmt::Display;
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
        ctx.syntax(self);
    }
}

impl<S, T> SqlSyntax<S> for sqlx_type<T>
where
    S: Database,
    T: Type<S>,
{
    fn to_sql(self, stmt: &mut String) {
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
    Name: SqlSanitize<S> + 'q,
    Type: Expression<'q, S> + 'q,
    Constraints: ZeroOrMoreExpressions<'q, S> + 'q,
{
    fn expression(self, ctx: &mut QueryBuilder<'q, S>)
    where
        S: DatabaseExt,
    {
        ctx.sanitize(self.name);
        ctx.syntax(" ");
        self.ty.expression(ctx);
        self.constraints.expression(" ", " ", ctx);
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
        ctx.syntax(" ");
        ctx.syntax(sqlx_type(PhantomData::<T::Data>));
        let s = <T::Data as is_null::IsNull>::is_null();
        if s.not() {
            ctx.syntax(" NOT NULL");
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
    T: SqlSanitize<S>,
{
    fn expression(self, ctx: &mut QueryBuilder<'q, S>)
    where
        S: DatabaseExt,
    {
        ctx.sanitize(self.0);
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
        ctx.syntax(" AS ");
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
        ctx.syntax("LEFT JOIN ");
        ctx.sanitize(self.ft.clone());
        ctx.syntax(" ON ");
        ctx.sanitize(self.lt);
        ctx.syntax(".");
        ctx.sanitize(self.lc);
        ctx.syntax(" = ");
        ctx.sanitize(self.ft);
        ctx.syntax(".");
        ctx.sanitize(self.fc);
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

impl<Col, Eq> OpExpression for col_eq<Col, Eq> {}

impl<'q, S, Col, Eq> Expression<'q, S> for col_eq<Col, Eq>
where
    S: DatabaseExt,
    Eq: 'q + Encode<'q, S> + Type<S>,
    Col: SqlSanitize<S> + 'q,
{
    fn expression(self, arg: &mut QueryBuilder<'q, S>) {
        arg.sanitize(self.col);
        arg.syntax(" = ");
        arg.bind(self.eq);
    }
}

pub struct table<T>(pub T);

impl<T> OpExpression for table<T> {}

impl<'q, S, Table> Expression<'q, S> for table<Table>
where
    S: DatabaseExt,
    Table: SqlSanitize<S> + 'q,
{
    fn expression(self, arg: &mut QueryBuilder<'q, S>) {
        arg.sanitize(self.0);
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
    pub fn pre_alias<Alias>(self, alias: Alias) -> PreAlias<scoped_column<Table, Column>, Alias> {
        PreAlias { on: self, alias }
    }
}

impl<Table, Column> OpExpression for scoped_column<Table, Column> {}

impl<'q, S, Column, Table> Expression<'q, S> for scoped_column<Table, Column>
where
    S: DatabaseExt,
    Table: SqlSanitize<S> + 'q,
    Column: SqlSanitize<S> + 'q,
{
    fn expression(self, arg: &mut QueryBuilder<'q, S>) {
        arg.sanitize(self.table);
        arg.syntax(".");
        arg.sanitize(self.column);
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
    T: SqlSanitize<S> + 'q,
    C: SqlSanitize<S> + 'q + Display,
    A: SqlSanitize<S> + 'q + Display,
{
    fn expression(self, arg: &mut QueryBuilder<'q, S>) {
        let alias = format!("{}{}", self.alias, self.on.column);
        arg.sanitize(self.on.table);
        arg.syntax(".");
        arg.sanitize(self.on.column);
        arg.syntax(" AS ");
        arg.sanitize(alias);
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
        arg.syntax(" AS ");
        arg.sanitize(alias);
    }
}

pub struct id_constraint<Id, Constraint>(pub Id, pub Constraint);

impl<Id, Constraint> OpExpression for id_constraint<Id, Constraint> {}

mod imp_id_constraint_for_sqlite {
    use sqlx::Sqlite;

    use crate::{Expression, SqlSanitize};

    impl<'q, Id, Constraint> Expression<'q, Sqlite> for super::id_constraint<Id, Constraint>
    where
        Id: SqlSanitize<Sqlite> + 'q,
        Constraint: Expression<'q, Sqlite>,
    {
        fn expression(self, ctx: &mut crate::QueryBuilder<'q, Sqlite>)
        where
            Sqlite: crate::DatabaseExt,
        {
            ctx.sanitize(self.0);
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
    use crate::{Expression, ZeroOrMoreExpressions};
    use sqlx::Sqlite;

    impl<'q, Ons> Expression<'q, Sqlite> for super::foriegn_key<Ons>
    where
        Ons: ZeroOrMoreExpressions<'q, Sqlite> + 'q,
    {
        fn expression(self, ctx: &mut crate::QueryBuilder<'q, Sqlite>)
        where
            Sqlite: crate::DatabaseExt,
        {
            let o = "(";
            let c = ")";
            ctx.syntax(" REFERENCES ");

            ctx.sanitize(self.references_table);
            ctx.syntax(o);
            ctx.sanitize(self.references_col);
            ctx.syntax(c);
            self.ons.expression(" ", " ", ctx);
        }
    }
}

pub struct on_delete_set_null;

impl OpExpression for on_delete_set_null {}

mod imp_on_delete_set_null_for_sqlite {
    use crate::Expression;
    use sqlx::Sqlite;
    impl<'q> Expression<'q, Sqlite> for super::on_delete_set_null {
        fn expression(self, ctx: &mut crate::QueryBuilder<'q, Sqlite>)
        where
            Sqlite: crate::DatabaseExt,
        {
            ctx.syntax("ON DELETE SET NULL");
        }
    }
}
