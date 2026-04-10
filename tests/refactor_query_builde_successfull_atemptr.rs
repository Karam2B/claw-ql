#![allow(unused)]
#![allow(non_camel_case_types)]
#![warn(unused_must_use)]

// impl<'a> Encode<'a, Any | Sqlite> for &'a str {}
// impl<'a> Encode<'a, Any | Sqlite> for &'a [u8] {}
// impl<'a, T> Encode<'a, Any | Sqlite> for &'_ T where T: Ecode<'a, _> {}
// impl Encode<'_, MySql> for &'_ str {}

// impl<'q> Arguments<'q> for SqliteArguments<'q> {
//     fn add<T>(&mut self, value: T) -> Result<(), BoxDynError>
//     where
//         T: 'q + Encode<'q>,

// pub trait Executor<'executor>: Send + Debug + Sized {
//     /// Execute the query and return the total number of rows affected.
//     fn execute<'boxing, 'q, E>(
//         self,
//         query: E,
//     ) -> BoxFuture<'boxing, Result<<Self::Database as Database>::QueryResult, Error>>
//     where
//         'q: 'boxing,
//         'executor: 'boxing,
//         E: 'q + Execute<'q, Self::Database>,
//     {

// pub struct QueryBuilder<'args, DB>
// where
//     DB: Database,
// {
//     query: String,
//     init_len: usize,
//     arguments: Option<<DB as Database>::Arguments<'args>>,
// }
// impl<'args, DB: Database> QueryBuilder<'args, DB>
//      pub fn build(&mut self) -> Query<'_, DB, <DB as Database>::Arguments<'args>> {
//          Query {
//              statement: Either::Left(&self.query),
//              arguments: self.arguments.take().map(Ok),
//              database: PhantomData,
//              persistent: true,
//          }

use std::{
    fmt::{Display, format},
    marker::PhantomData,
    mem,
};

use claw_ql::{ConnectInMemory, execute::Executable};
use futures::StreamExt;
use sqlx::{
    Database, Encode, Execute, Executor, FromRow, Sqlite, Type,
    sqlite::{SqliteArguments, SqliteRow},
};
use tracing_subscriber::registry::Data;

// =================================== statments
pub trait Buildable: Sized {
    type QueryBuilder: QueryBuilder;
    fn build(self) -> (String, <Self::QueryBuilder as QueryBuilder>::Output);
}
// //////////////////////////////////// statments

// =================================== expression
pub trait Expression<'q, Q> {
    fn expression(
        self,
        ctx: &mut Q,
    ) -> impl FnOnce(&mut Q::Context) -> String + 'q + use<'q, Q, Self>
    where
        Q: QueryBuilder;
}
// ///////////////////////////////////// expression

// =================================== query builders
pub trait QueryBuilder {
    type Output;
    type Fragment;
    type Context;
    fn to_output(
        self,
        statement_builder: impl FnOnce(&mut Self::Context) -> String,
    ) -> (String, Self::Output);
    fn fragment_to_string(ctx: &mut Self::Context, from: Self::Fragment) -> String;
}

pub trait ExpressionToFragment<'q, T>: QueryBuilder {
    fn expression_to_fragment(&mut self, t: T) -> <Self as QueryBuilder>::Fragment;
}

// trait to extend sqlx's Encode trait -- adapted to fit the need of this library
pub trait EncodeExtention<'q, T>: QueryBuilder {
    fn encode(
        &mut self,
        val: T,
    ) -> impl FnOnce(&mut Self::Context) -> String + 'q + use<'q, T, Self>;
}

pub struct hardcode<T>(pub T);

impl<'q, T, Q: QueryBuilder> Expression<'q, Q> for hardcode<T> {
    fn expression(
        self,
        ctx: &mut Q,
    ) -> impl FnOnce(&mut <Q>::Context) -> String + 'q + use<'q, Q, T>
    where
        Q: QueryBuilder,
    {
        todo!();
        |_| todo!()
    }
}
// impl<'q, T, Q: QueryBuilder> EncodeExtention<'q, hardcode<T>> for Q {
//     fn encode(
//         &mut self,
//         val: hardcode<T>,
//     ) -> impl FnOnce(&mut Self::Context) -> String + 'q + use<'q, T, Q> {
//         |_| panic!()
//     }
// }

// ////////////////////////////////// query builders

pub struct QuickStatement<S: QueryBuilder> {
    pub selecting: Option<S::Fragment>,
    pub ctx: S,
}

impl<S> Buildable for QuickStatement<S>
where
    S: QueryBuilder,
{
    type QueryBuilder = S;
    fn build(self) -> (String, <Self::QueryBuilder as QueryBuilder>::Output) {
        S::to_output(self.ctx, |ctx2| {
            let wh = S::fragment_to_string(ctx2, self.selecting.unwrap());
            format!("SELECT {}", wh)
        })
    }
}

impl<Q> QuickStatement<Q>
where
    Q: QueryBuilder,
{
    pub fn simple_select<'q, T>(&mut self, t: T)
    where
        Q: ExpressionToFragment<'q, T>,
    {
        let s = self.ctx.expression_to_fragment(t);
        self.selecting = Some(s);
    }
}

pub struct select<T>(pub T);
impl<'q, T, Q> Expression<'q, Q> for select<T>
where
    Q: QueryBuilder,
    Q: EncodeExtention<'q, T>,
{
    fn expression(self, ctx: &mut Q) -> impl FnOnce(&mut Q::Context) -> String + 'q + use<'q, Q, T>
    where
        Q: QueryBuilder,
    {
        let s = Q::encode(ctx, self.0);
        move |ctx| format!("{} as selecting", s(ctx))
    }
}

pub struct lifetime_binder<'q, S: Database> {
    pub arg: S::Arguments<'q>,
    pub inc: i32,
    pub db: S,
}

impl<'q, S: Database> QueryBuilder for lifetime_binder<'q, S> {
    type Output = S::Arguments<'q>;

    type Fragment = String;

    type Context = ();

    fn to_output(self, f: impl FnOnce(&mut Self::Context) -> String) -> (String, Self::Output) {
        (f(&mut ()), self.arg)
    }

    fn fragment_to_string(_: &mut Self::Context, from: Self::Fragment) -> String {
        from
    }
}

impl<'q, S, T> EncodeExtention<'q, T> for lifetime_binder<'q, S>
where
    S: Database,
    // T = select<&String>
    T: Encode<'q, S> + Type<S> + 'q,
    // T: EncodeExt<'q, Self>,
{
    // fn bind(val: T, ctx: &mut <Self as QueryBuilder>::Context1) -> String {
    //     use sqlx::Arguments;
    //     let arg: &mut S::Arguments<'_> = ctx;
    //     arg.add(val).unwrap();
    //     "$1".to_string()
    // }
    fn encode(&mut self, val: T) -> impl FnOnce(&mut Self::Context) -> String + 'q + use<'q, T, S> {
        use sqlx::Arguments;
        self.arg.add(val).unwrap();
        self.inc += 1;
        let inc = self.inc;

        move |_| format!("${}", inc)
    }
}

impl<'q, T, Q: Database> ExpressionToFragment<'q, T> for lifetime_binder<'q, Q>
where
    T: Expression<'q, Self>,
{
    fn expression_to_fragment(&mut self, t: T) -> <Self as QueryBuilder>::Fragment {
        T::expression(t, self)(&mut ())
    }
}

#[tokio::test]
async fn lifetime() {
    let pool = Sqlite::connect_in_memory().await;

    let str = String::from("hello world");

    let mut st = QuickStatement {
        selecting: Default::default(),
        ctx: lifetime_binder {
            db: Sqlite,
            arg: Default::default(),
            inc: Default::default(),
        },
    };

    st.simple_select(select(&str));

    // this drop would fail because of trait Expression<'q> hold lifetime for str
    // drop(str);

    let (sql, arg) = st.build();

    assert_eq!(sql, "SELECT $1 as selecting".to_string());

    let r = pool
        .fetch_one(Executable {
            string: &sql,
            arguments: arg,
            db: PhantomData,
        })
        .await
        .unwrap();

    let (out,): (String,) = FromRow::from_row(&r).unwrap();

    assert_eq!(out, "hello world".to_string());

    // lifetimes were released!!
    drop(str);
}
