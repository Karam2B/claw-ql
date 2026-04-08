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

use std::{fmt::Display, marker::PhantomData, mem};

use claw_ql::{Buildable, expressions::ColEq, prelude::col};
use sqlx::{Database, Encode, Execute, Executor, Sqlite, Type};
use tracing_subscriber::registry::Data;
pub trait QueryBuilder {
    type Fragment;
    type Context1: Default + 'static;
    type Context2: From<Self::Context1>;

    fn build_sql_part_back(ctx: &mut Self::Context2, from: Self::Fragment) -> String;

    type Output;

    fn build_query(
        ctx1: Self::Context1,
        f: impl FnOnce(&mut Self::Context2) -> String,
    ) -> (String, Self::Output);
}

pub struct InnerExecutable<'q, S: Database, A> {
    pub string: &'q str,
    pub arguments: A,
    pub db: PhantomData<S>,
}

impl<'q, S: Database> Execute<'q, S> for InnerExecutable<'q, S, S::Arguments<'q>> {
    fn sql(&self) -> &'q str {
        &self.string
    }

    fn statement(&self) -> Option<&<S as Database>::Statement<'q>> {
        None
    }

    fn take_arguments(
        &mut self,
    ) -> Result<Option<<S as Database>::Arguments<'q>>, sqlx::error::BoxDynError> {
        Ok(Some(mem::take(&mut self.arguments)))
    }

    fn persistent(&self) -> bool {
        false
    }
}

pub trait BindItem<'e, S: QueryBuilder>: Sized {
    fn bind_item(
        self,
        ctx: &mut S::Context1,
    ) -> impl FnOnce(&mut S::Context2) -> String + 'e + use<'e, Self, S>;
}

pub trait EncodeExt<'e, This>: QueryBuilder + Send {
    fn accept(
        this: This,
        ctx1: &mut Self::Context1,
    ) -> impl FnOnce(&mut Self::Context2) -> String + 'e + Send + use<'e, Self, This>;
}

pub struct QuickQueryCtx<S: Database> {
    size: usize,
    arg: S::Arguments<'static>,
}

impl<S: Database> Default for QuickQueryCtx<S> {
    fn default() -> Self {
        QuickQueryCtx {
            size: 0,
            arg: Default::default(),
        }
    }
}

impl<S: Database> From<QuickQueryCtx<S>> for () {
    fn from(_this: QuickQueryCtx<S>) -> Self {
        ()
    }
}

impl QueryBuilder for sqlx::Sqlite {
    type Fragment = String;

    type Context1 = QuickQueryCtx<Self>;

    type Context2 = ();

    fn build_sql_part_back(_ctx: &mut Self::Context2, from: Self::Fragment) -> String {
        from
    }

    // lifetime here are 'static for simplicity
    // lifetime 'q has no use-case in my code if needed use support_lifetime<'q, Sqlite> instead
    type Output = <Sqlite as Database>::Arguments<'static>;

    fn build_query(
        ctx1: Self::Context1,
        f: impl FnOnce(&mut Self::Context2) -> String,
    ) -> (String, Self::Output) {
        // let noop = unsafe { &mut *(&mut ctx1.noop as *mut _) };
        let strr = f(&mut ());
        (strr, ctx1.arg)
    }

    // fn handle_bind_item<'e, T>(t: T, ctx: &mut Self::Context1) -> Self::Fragment
    // where
    //     T: BindItem<'e, Self> + 'static,
    //     Self: Sized,
    // {
    //     todo!()
    // }

    // fn handle_accept<'e, T>(t: T, ctx: &mut Self::Context1) -> Self::Fragment
    // where
    //     Self: EncodeExt<'e, T>,
    // {
    //     todo!()
    // }
}

pub struct SelectStGeneric<From, Select, WhereClause, Limit> {
    pub(crate) from: From,
    pub(crate) select: Select,
    pub(crate) where_clause: WhereClause,
    pub(crate) limit: Limit,
}

pub struct SelectSt<S: QueryBuilder> {
    pub(crate) where_clause: Vec<S::Fragment>,
}

impl Buildable for SelectSt<Sqlite> {
    type Database = Sqlite;

    fn build(self) -> (String, <Self::Database as claw_ql::QueryBuilder>::Output) {
        todo!()
    }
}

impl<S: QueryBuilder> SelectSt<S> {
    fn select<T>(&mut self, item: T)
    where
        hardcode<T>: BindItem<'static, S>,
    {
        todo!()
    }
    fn where_<'e, T>(&mut self, item: T)
    where
        // S: 'e,
        // S: SupportLifetime<'e>,
        T: BindItem<'e, S>,
    {
        todo!()
    }
}
pub struct todo;
pub struct todo_title;

impl SelectStGeneric<todo, (hardcode<todo_title>,), (ColEq<i32>,), ()> {
    fn into<Q: QueryBuilder>(self) -> SelectSt<Q>
    where
        ColEq<i32>: BindItem<'static, Q>,
    {
        todo!()
    }
}

// fn bar() {
//     let s = col("title").table("Todo").alias("todo_title").eq(3);
//     // let ss: SelectStGeneric<todo, (ColEq<i32>,), ()> = todo!();
// }

impl<T> EncodeExt<'static, T> for Sqlite
where
    T: for<'e> Encode<'e, Sqlite> + Type<Sqlite> + Send + 'static,
{
    fn accept(
        this: T,
        ctx1: &mut Self::Context1,
    ) -> impl FnOnce(&mut Self::Context2) -> String + 'static + Send + use<T> {
        use sqlx::Arguments;
        ctx1.arg.add(this).unwrap();
        ctx1.size += 1;
        let len = ctx1.size;
        move |_| format!("${}", len)
    }
}

impl<T> EncodeExt<'static, hardcode<T>> for Sqlite
where
    T: SanitizeAndHardcode<by_double_quote>,
{
    fn accept(
        this: hardcode<T>,
        ctx1: &mut Self::Context1,
    ) -> impl FnOnce(&mut Self::Context2) -> String + 'static + Send + use<T> {
        |_| todo!()
    }
}

#[derive(Default)]
pub struct Counter(i64);

impl From<Counter> for () {
    fn from(value: Counter) -> Self {}
}

impl<S> QueryBuilder for never_binder<S> {
    type Fragment = String;

    type Context1 = Counter;

    type Context2 = ();

    fn build_sql_part_back(ctx: &mut Self::Context2, from: Self::Fragment) -> String {
        todo!()
    }

    type Output = ();

    fn build_query(
        ctx1: Self::Context1,
        f: impl FnOnce(&mut Self::Context2) -> String,
    ) -> (String, Self::Output) {
        todo!()
    }
}
pub struct by_double_quote;
pub trait SanitizeAndHardcode<Escape> {
    fn sanitize(&self) -> String;
}

pub trait SanitzingMechanisim {
    type SanitzingMechanisim;
}

impl SanitzingMechanisim for Sqlite {
    type SanitzingMechanisim = by_double_quote;
}

pub struct hardcode<T>(T);

impl<E, T: SanitizeAndHardcode<E>> SanitizeAndHardcode<E> for hardcode<T>
where
    T: SanitizeAndHardcode<E>,
{
    fn sanitize(&self) -> String {
        self.0.sanitize()
    }
}

impl SanitizeAndHardcode<by_double_quote> for bool {
    fn sanitize(&self) -> String {
        match self {
            true => "true",
            false => "false",
        }
        .to_string()
    }
}

impl SanitizeAndHardcode<by_double_quote> for String {
    fn sanitize(&self) -> String {
        let mut new = String::from('\'');
        for (index, char) in self.chars().enumerate() {
            if char == '\'' {
                new.push('"');
            } else {
                new.push(char);
            }
        }
        new.push('\'');
        new
    }
}

impl SanitizeAndHardcode<by_double_quote> for &'_ str {
    fn sanitize(&self) -> String {
        let mut new = String::from('\'');
        for (index, char) in self.chars().enumerate() {
            if char == '\'' {
                new.push('"');
            } else {
                new.push(char);
            }
        }
        new.push('\'');
        new
    }
}

impl<S: Send + Sync + SanitzingMechanisim, T: SanitizeAndHardcode<S::SanitzingMechanisim>>
    EncodeExt<'static, T> for never_binder<S>
{
    fn accept(
        this: T,
        ctx1: &mut Self::Context1,
    ) -> impl FnOnce(&mut Self::Context2) -> String + 'static + Send + use<S, T> {
        |_| "".to_string()
    }
}

pub struct CreateTableSt2<S: QueryBuilder>(S);

pub struct never_binder<S>(S);
pub struct defered_binder<S>(S);
pub struct immediate_binder<S>(S);

pub struct binder_support_lifetime<'a, S>(PhantomData<&'a S>);

fn bar() {
    // CreateTableSt2(default_binder(Sqlite))
}

pub trait StatementMayNeverBind {}
