use std::marker::PhantomData;

use crate::{Accept, BindItem, IntoInferFromPhantom, QueryBuilder};

pub struct Col {
    pub(crate) table: Option<String>,
    pub(crate) alias: Option<String>,
    pub(crate) col: String,
}

impl From<Col> for String {
    fn from(value: Col) -> Self {
        format!(
            "{}{}{}",
            match value.table {
                Some(table) => format!("{table}."),
                None => "".to_string(),
            },
            value.col,
            match value.alias {
                Some(alias) => format!(" AS {alias}"),
                None => "".to_string(),
            }
        )
    }
}

pub struct ColEq<T> {
    pub(crate) col: Col,
    pub(crate) item: T,
}

impl<S, T> BindItem<S> for ColEq<T>
where
    S: QueryBuilder,
    S: Accept<T>,
{
    fn bind_item(
        self,
        ctx: &mut S::Context1,
    ) -> impl FnOnce(&mut S::Context2) -> String + 'static + use<T, S> {
        let acc = S::accept(self.item, ctx);
        move |ctx2| {
            format!(
                "{} = {}",
                self.col.into_pd(PhantomData::<String>),
                acc(ctx2)
            )
        }
    }
}

impl Col {
    pub fn table(mut self, table: &str) -> Self {
        self.table = Some(table.to_string());
        self
    }
    pub fn alias(mut self, alias: &str) -> Self {
        self.alias = Some(alias.to_string());
        self
    }
    pub fn eq<T1>(self, value: T1) -> ColEq<T1> {
        ColEq {
            col: self,
            item: value,
        }
    }
}

pub mod is_null {
    use sqlx::TypeInfo;
    use std::{marker::PhantomData, ops::Not};

    use sqlx::Type;

    use crate::{BindItem, ColumPositionConstraint, QueryBuilder};

    pub trait IsNull {
        fn is_null() -> bool;
    }
    pub struct ColumnTypeCheckIfNull<T>(pub PhantomData<T>);

    impl<T> ColumPositionConstraint for ColumnTypeCheckIfNull<T> {}

    impl<S, T> BindItem<S> for ColumnTypeCheckIfNull<T>
    where
        S: QueryBuilder,
        T: Type<S> + IsNull,
    {
        fn bind_item(
            self,
            _: &mut <S as QueryBuilder>::Context1,
        ) -> impl FnOnce(&mut <S as QueryBuilder>::Context2) -> String + 'static + use<T, S>
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

    #[cfg(feature = "waiting_min_specialization")]
    impl<T> IsNull for T {
        default fn is_null() -> bool {
            false
        }
    }

    #[cfg(not(feature = "waiting_min_specialization"))]
    mod impl_is_null_no_spectialization {
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
    }
}

pub mod primary_key {
    use std::marker::PhantomData;

    use crate::{BindItem, ColumPositionConstraint, QueryBuilder};

    pub struct PrimaryKey<S>(pub PhantomData<S>);

    use sqlx::prelude::Type;

    impl<T> ColumPositionConstraint for PrimaryKey<T> {}

    impl<S> BindItem<S> for PrimaryKey<S>
    where
        S: DatabaseDefaultPrimaryKey,
        S: QueryBuilder,
        S::KeyType: Type<S>,
    {
        fn bind_item(
            self,
            _ctx: &mut <S as QueryBuilder>::Context1,
        ) -> impl FnOnce(&mut <S as QueryBuilder>::Context2) -> String + 'static + use<S> {
            |_| {
                let ty = S::KeyType::type_info();
                format!("{} {}", ty, S::default_primary_key())
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

    #[track_caller]
    pub fn col(str: &str) -> Col {
        Col {
            table: None,
            col: str.to_string(),
            alias: None,
        }
    }
    pub fn primary_key<S: DatabaseDefaultPrimaryKey>() -> PrimaryKey<S> {
        PrimaryKey(PhantomData)
    }
    pub fn col_type_check_if_null<T>() -> ColumnTypeCheckIfNull<T> {
        ColumnTypeCheckIfNull(PhantomData::<T>)
    }
}
