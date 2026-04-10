use std::marker::PhantomData;

use crate::{
    EncodeExtention, Expression, IntoInferFromPhantom, QueryBuilder, SanitzingMechanisim,
    SelectListItem, WhereItem, sanitize::SanitizeAndHardcode,
};

impl SelectListItem for &'_ str {}
impl SelectListItem for String {}

// impl<B,C> Member<B> for scoped_col<&str, C> where C: Member<B> {}
// impl<B,C> Member<B> for scoped_col<String, C> where C: Member<B> {}
// impl<B,C> Member<B> for col<C> where C: Member<B> {}

// impl Member<&str> for &str {}
// impl Member<String> for &str {}
// impl Member<String> for String {}
// impl Member<&'str> for String {}

pub struct table<Table>(pub Table);

impl<T> table<T> {
    pub fn col<C>(self, c: C) -> scoped_col<T, C> {
        scoped_col {
            table: self.0,
            col: c,
        }
    }
}

pub struct col<Name>(pub Name);

impl<C> col<C> {
    pub fn table<T>(self, t: T) -> scoped_col<T, C> {
        scoped_col {
            table: t,
            col: self.0,
        }
    }

    pub fn to_eq<ToEq>(self, to_eq: ToEq) -> col_to_eq<C, ToEq> {
        col_to_eq {
            select: self.0,
            to_eq,
        }
    }

    pub fn alias<As>(self, as_: As) -> aliased<col<C>, As> {
        aliased { select: self, as_ }
    }
}

impl<Name> SelectListItem for col<Name> {}
impl<E, Name: SanitizeAndHardcode<E>> SanitizeAndHardcode<E> for col<Name> {
    fn sanitize(&self) -> String {
        self.0.sanitize()
    }
}

pub struct col_to_eq<Name, ToEq> {
    pub select: Name,
    pub to_eq: ToEq,
}

impl<Name, ToEq> WhereItem<Name> for col_to_eq<Name, ToEq> {}

impl<'q, S, Name, ToEq> Expression<'q, S> for col_to_eq<Name, ToEq>
where
    S: SanitzingMechanisim,
    Name: 'q + SelectListItem + SanitizeAndHardcode<S::SanitzingMechanisim>,
    S: 'q + EncodeExtention<'q, ToEq>,
{
    fn expression(
        self,
        query_builder: &mut S,
    ) -> impl FnOnce(&mut <S>::Context) -> String + 'q + use<'q, S, Name, ToEq>
    where
        S: QueryBuilder,
    {
        let to_eq = S::encode(query_builder, self.to_eq);
        move |ctx| format!("{} = {}", self.select.sanitize(), to_eq(ctx))
    }
}

pub struct scoped_col<Table, Col> {
    pub table: Table,
    pub col: Col,
}

impl<T, C> scoped_col<T, C> {
    pub fn to_eq<ToEq>(self, to_eq: ToEq) -> col_to_eq<scoped_col<T, C>, ToEq> {
        col_to_eq {
            select: self,
            to_eq,
        }
    }

    pub fn alias<As>(self, as_: As) -> aliased<scoped_col<T, C>, As> {
        aliased { select: self, as_ }
    }
}

impl<Table, Name> SelectListItem for scoped_col<Table, Name>
where
    Table: SelectListItem,
    Name: SelectListItem,
{
}
impl<E, Table, Name> SanitizeAndHardcode<E> for scoped_col<Table, Name>
where
    Table: SanitizeAndHardcode<E>,
    Name: SanitizeAndHardcode<E>,
{
    fn sanitize(&self) -> String {
        format!("{}.{}", self.table.sanitize(), self.col.sanitize())
    }
}

pub struct aliased<Select, As> {
    pub select: Select,
    pub as_: As,
}

impl<Select, As> SelectListItem for aliased<Select, As>
where
    Select: SelectListItem,
    As: SelectListItem,
{
}
impl<E, Select, As> SanitizeAndHardcode<E> for aliased<Select, As>
where
    As: SanitizeAndHardcode<E>,
    Select: SanitizeAndHardcode<E>,
{
    fn sanitize(&self) -> String {
        format!("{} AS {}", self.select.sanitize(), self.as_.sanitize())
    }
}

pub mod is_null {
    use sqlx::{Database, TypeInfo};
    use std::{marker::PhantomData, ops::Not};

    use sqlx::Type;

    use crate::{ColumPositionConstraint, Expression, QueryBuilder};

    pub trait IsNull {
        fn is_null() -> bool;
    }
    pub struct ColumnTypeCheckIfNull<T>(pub PhantomData<T>);

    impl<T> ColumPositionConstraint for ColumnTypeCheckIfNull<T> {}

    impl<Q, T> Expression<'static, Q> for ColumnTypeCheckIfNull<T>
    where
        Q: QueryBuilder,
        Q: Database,
        T: Type<Q> + IsNull,
    {
        fn expression(
            self,
            query_builder: &mut Q,
        ) -> impl FnOnce(&mut <Q>::Context) -> String + use<Q, T> + 'static
        where
            Q: QueryBuilder,
        {
            |_| {
                let ty = T::type_info();
                let mut ty = ty.name().to_string();
                if T::is_null().not() {
                    ty.push_str(" NOT NULL")
                }
                ty
            }
        }
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

pub mod primary_key {
    use std::marker::PhantomData;

    use crate::{ColumPositionConstraint, Expression, QueryBuilder};

    pub struct PrimaryKey<S>(pub PhantomData<S>);

    use sqlx::{Database, prelude::Type};

    impl<T> ColumPositionConstraint for PrimaryKey<T> {}

    impl<Q> Expression<'static, Q> for PrimaryKey<Q>
    where
        Q: QueryBuilder,
        Q::SqlxDb: Database,
        Q::SqlxDb: DatabaseDefaultPrimaryKey<KeyType: Type<Q::SqlxDb>>,
    {
        fn expression(
            self,
            query_builder: &mut Q,
        ) -> impl FnOnce(&mut <Q>::Context) -> String + 'static + use<Q>
        where
            Q: QueryBuilder,
        {
            |_| {
                let ty = <Q::SqlxDb as DatabaseDefaultPrimaryKey>::KeyType::type_info();
                format!("{} {}", ty, Q::SqlxDb::default_primary_key())
            }
        }
    }

    impl DatabaseDefaultPrimaryKey for sqlx::Sqlite {
        type KeyType = i64;
        fn default_primary_key() -> &'static str {
            "PRIMARY KEY AUTOINCREMENT"
        }
    }

    // impl PrimaryKey for sqlx::Postgres {
    //     type KeyType = i64;
    //     fn default_primary_key() -> &'static str {
    //         "PRIMARY KEY"
    //     }
    // }

    pub trait DatabaseDefaultPrimaryKey {
        type KeyType;
        fn default_primary_key() -> &'static str;
    }
}

pub mod exports {
    use super::is_null::ColumnTypeCheckIfNull;
    use super::primary_key::{DatabaseDefaultPrimaryKey, PrimaryKey};
    use super::*;
    use std::marker::PhantomData;

    pub fn primary_key<S: DatabaseDefaultPrimaryKey>() -> PrimaryKey<S> {
        PrimaryKey(PhantomData)
    }
    pub fn col_type_check_if_null<T>() -> ColumnTypeCheckIfNull<T> {
        ColumnTypeCheckIfNull(PhantomData::<T>)
    }
}
