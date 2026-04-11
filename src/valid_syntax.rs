use std::{fmt::Debug, marker::PhantomData};

use sqlx::Sqlite;

use crate::{
    collections::Member,
    expressions::{col_eq, scoped_column},
    operations::fetch_one::FetchOne,
};

pub fn is_valid_syntax<F, S>(f: F, _: PhantomData<S>) -> F
where
    Statement<S, F>: ValidSyntax,
{
    f
}

#[derive(Debug)]
pub struct InvalidSyntax;

pub fn runtime_check<S, RT>() -> Result<(), InvalidSyntax> {
    Ok(())
}

pub trait ValidSyntax {
    fn runtime_check(&self) -> bool {
        true
    }
}

pub trait ValidOn<S, Base> {}

impl<S, F: ValidOn<S, B>, B> ValidOn<S, B> for (F,) {}

impl<S, B, C, E> ValidOn<S, B> for col_eq<C, E>
where
    C: ValidMember,
    C::Data: Equalable<S, E>,
{
}

pub trait Equalable<S, To> {}

impl Equalable<Sqlite, &str> for String {}

pub trait ValidMember {
    type Data;
}

impl<T, C> ValidMember for scoped_column<temp::Table<T>, temp::Member<C>>
where
    C: Member<Collection = T>,
{
    type Data = C::Data;
}

pub struct Statement<S, F>(PhantomData<(S, F)>);

impl<S, F, L, W> ValidSyntax for Statement<S, FetchOne<F, L, W>> where W: ValidOn<S, F> {}

pub trait CreateTableHeader {
    fn runtime_check(&self) -> bool {
        true
    }
}

pub trait TableIdent {}
pub trait ColIdent<Table> {}

pub mod temp {
    use std::marker::PhantomData;

    use sqlx::{Database, Pool};

    use crate::collections::{self, CollectionBasic, MemberBasic};
    use crate::expressions::scoped_column;
    use crate::expressions::table;
    use crate::query_builder::SqlSanitize;

    pub struct Table<T>(T);
    impl<S, T: CollectionBasic> SqlSanitize<S> for Table<T> {
        fn to_sql(&self) -> &str {
            self.0.table_name_lower_case()
        }
    }
    pub struct Member<T>(T);

    impl<S, T: MemberBasic> SqlSanitize<S> for Member<T> {
        fn to_sql(&self) -> &str {
            self.0.name()
        }
    }

    pub fn member<T>(t: T) -> scoped_column<Table<T::Collection>, Member<T>>
    where
        T: collections::Member,
        T::Collection: Default,
    {
        table(Table(T::Collection::default())).col(Member(t))
    }

    pub fn infer_db<S: Database>(_: &Pool<S>) -> PhantomData<S> {
        PhantomData
    }
}
