use sqlx::{Database, Sqlite};

pub mod defered_builder;
pub mod direct_builder;
pub mod sanitize;

pub trait QueryBuilder {
    type Output;
    type Fragment;
    type Context;
    type SqlxDb;
    fn to_output(
        self,
        statement_builder: impl FnOnce(&mut Self::Context) -> String,
    ) -> (String, Self::Output);
    fn fragment_to_string(ctx: &mut Self::Context, fragment: Self::Fragment) -> String;
}

pub trait ExpressionToFragment<'q, T>: QueryBuilder {
    fn expression_to_fragment(&mut self, expression: T) -> <Self as QueryBuilder>::Fragment;
}

// trait to extend sqlx's Encode trait -- adapted to fit the need of this library
pub trait EncodeExtention<'q, T>: QueryBuilder {
    fn encode(
        &mut self,
        val: T,
    ) -> impl FnOnce(&mut Self::Context) -> String + 'q + use<'q, T, Self>;
}

pub trait Buildable: Sized {
    type QueryBuilder: QueryBuilder;
    fn build(self) -> (String, <Self::QueryBuilder as QueryBuilder>::Output);
}

pub trait Expression<'q, Q> {
    fn expression(
        self,
        query_builder: &mut Q,
    ) -> impl FnOnce(&mut Q::Context) -> String + 'q + use<'q, Q, Self>
    where
        Q: QueryBuilder;
}

pub trait ColumPositionConstraint {}
pub trait WhereItem<Base> {}
pub trait SelectListItem {}
pub trait JoinItem {}

pub trait PositionalPlaceholder {
    fn placeholder() -> &'static str;
}

pub trait NamedPlaceholder {
    fn placeholder(inc: usize) -> String;
}

pub trait SanitzingMechanisim {
    type SanitzingMechanisim;
}

mod sqlite {
    use sqlx::Sqlite;

    use crate::{NamedPlaceholder, SanitzingMechanisim, sanitize::by_double_quote};

    impl NamedPlaceholder for Sqlite {
        fn placeholder(inc: usize) -> String {
            format!("${}", inc)
        }
    }

    impl SanitzingMechanisim for Sqlite {
        type SanitzingMechanisim = by_double_quote;
    }
}
