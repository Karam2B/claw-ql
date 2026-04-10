#![allow(unexpected_cfgs)]
#![allow(unused)]
#![warn(unused_must_use)]
#![allow(non_camel_case_types)]
#![allow(non_upper_case_globals)]
use crate::todo_members::{description, done, title};
use claw_ql::collections::{Collection, CollectionBasic, Id, MemberBasic, SingleIncremintalInt};
use claw_ql::direct_builder::direct_bind;
use claw_ql::execute::Executable;
use claw_ql::expressions::{aliased, scoped_col, table};
use claw_ql::links::Link;
use claw_ql::links::relation_optional_to_many::optional_to_many;
use claw_ql::prelude::join::left_join;
use claw_ql::sanitize::SanitizeAndHardcode;
use claw_ql::{ConnectInMemory, collections::Member};
use claw_ql::{EncodeExtention, Expression, WhereItem};
use claw_ql_macros::sql;
use futures::{StreamExt, TryStreamExt};
use sqlx::Database;
use sqlx::Executor;
use sqlx::Row;
use sqlx::sqlite::SqliteRow;
use sqlx::{Pool, Sqlite, sqlite::SqliteArguments};
use std::marker::PhantomData;
use std::net::ToSocketAddrs;
use std::ops::Not;
use std::{fmt::format, sync::LazyLock, vec::IntoIter};

fn execute<T>(st: T, p: &Pool<Sqlite>) {}

#[derive(claw_ql_macros::Collection)]
pub struct Todo {
    pub title: String,
    pub done: bool,
    pub description: Option<String>,
}

#[derive(claw_ql_macros::Collection)]
pub struct Category {
    pub title: String,
}

pub trait Queries<S>: Collection {
    fn select_list<'a>(&'a self) -> impl Iterator<Item = &'a str> + use<'a, Self, S>;
    fn from_row(&self, row: &<S as Database>::Row, aliase: &str) -> Self::Data
    where
        S: Database;
}

impl Queries<Sqlite> for todo {
    fn select_list<'a>(&'a self) -> impl Iterator<Item = &'a str> + use<'a> {
        [
            todo_members::title.name(),
            todo_members::done.name(),
            todo_members::description.name(),
        ]
        .into_iter()
    }
    fn from_row(&self, row: &<Sqlite as Database>::Row, aliase: &str) -> Self::Data
    where
        Sqlite: Database,
    {
        use sqlx::Row;
        Todo {
            title: row.get(&*format!("{}title", aliase)),
            done: row.get(&*format!("{}done", aliase)),
            description: row.get(&*format!("{}description", aliase)),
        }
    }
}

impl Queries<Sqlite> for category {
    fn select_list<'a>(&'a self) -> impl Iterator<Item = &'a str> + use<'a> {
        [category_members::title.name()].into_iter()
    }
    fn from_row(&self, row: &<Sqlite as Database>::Row, aliase: &str) -> Self::Data
    where
        Sqlite: Database,
    {
        use sqlx::Row;
        Category {
            title: row.get(&*format!("{}title", aliase)),
        }
    }
}

impl Link<todo> for category {
    type Spec = optional_to_many<todo, category>;

    fn spec(self, base: &todo) -> Self::Spec {
        panic!()
    }
}

pub struct Select<From, Wheres, Link, Return, Limit> {
    pub from: From,
    pub wheres: Wheres,
    pub link: Link,
    pub returning: Return,
    pub limit: Limit,
}

pub struct col_eq<Field, ToBe>(pub Field, pub ToBe);
impl<B, F, T> WhereItem<B> for col_eq<F, T> where F: Member<Collection = B, Data = T> {}
impl Expression<'static, direct_bind<'static, Sqlite>> for col_eq<todo_members::title, String> {
    fn expression(
        self,
        query_builder: &mut direct_bind<'static, Sqlite>,
    ) -> impl FnOnce(&mut ()) -> String + 'static + use<>
    where
        direct_bind<'static, Sqlite>: claw_ql::QueryBuilder,
    {
        let s = query_builder.encode(self.1);
        move |c| format!("{} = {}", self.0.name().sanitize(), s(c))
    }
}
trait WhereExpression {
    fn expression_b(
        self: Box<Self>,
        qb: &mut direct_bind<'static, Sqlite>,
    ) -> Box<dyn FnOnce(&mut ()) -> String>;
}
impl WhereExpression for col_eq<todo_members::title, String> {
    fn expression_b(
        self: Box<Self>,
        qb: &mut direct_bind<'static, Sqlite>,
    ) -> Box<dyn FnOnce(&mut ()) -> String> {
        Box::new(self.expression(qb))
    }
}

pub struct SelectFragment<S, J, W> {
    pub select_items: S,
    pub none_duplicating_join: J,
    pub wheres: W,
}

pub trait FetchManyLink<S> {
    type SelectItems;
    type Join;
    type Wheres;
    fn select_fragment(&self) -> SelectFragment<Self::SelectItems, Self::Join, Self::Wheres>;

    type Inner: Default;
    fn from_row(&self, row: &<S as Database>::Row) -> Self::Inner
    where
        S: Database;

    type SubOp;
    fn sub_op(&self, inner: &mut Self::Inner) -> Self::SubOp;

    type Output;
    fn take(self, inner: Self::Inner) -> Self::Output;
}

impl<F, T> FetchManyLink<Sqlite> for optional_to_many<F, T>
where
    F: Collection,
    T: Collection<Id: Id<SqlIdent: ToString>>,
    T: Queries<Sqlite>,
{
    type SelectItems = SelectItem<(
        scoped_col<String, String>,
        SelectItem<Vec<aliased<scoped_col<String, String>, String>>>,
    )>;
    type Join = left_join;
    type Wheres = ();
    fn select_fragment(&self) -> SelectFragment<Self::SelectItems, Self::Join, Self::Wheres> {
        let sf = SelectFragment {
            select_items: SelectItem((
                table(self.from.table_name().to_string()).col(self.foriegn_key.clone()),
                SelectItem(
                    self.to
                        .select_list()
                        .map(|col| {
                            table(self.to.table_name().to_string())
                                .col(col.to_string())
                                .alias(format!("{}_{}", "", ""))
                        })
                        .collect::<Vec<_>>(),
                ),
            )),
            none_duplicating_join: left_join {
                foriegn_table: self.to.table_name().to_string(),
                foriegn_column: self.to.id().ident().to_string(),
                local_table: self.from.table_name().to_string(),
                local_column: self.foriegn_key.to_string(),
            },
            wheres: (),
        };
        sf
    }

    type SubOp = ();
    type Inner = Option<(i64, <T as Collection>::Data)>;
    fn from_row(&self, row: &<Sqlite as Database>::Row) -> Self::Inner {
        let fk: i64 = row.get(self.foriegn_key.as_str());
        let entity = Queries::from_row(&self.to, row, "aliase");
        Some((fk, entity))
    }
    fn sub_op(&self, inner: &mut Self::Inner) -> Self::SubOp {}

    type Output = Option<(i64, <T as Collection>::Data)>;
    fn take(self, inner: Self::Inner) -> Self::Output {
        inner
    }
}

pub struct CollectionOutput<Id, Base, Link> {
    pub id: Id,
    pub base: Base,
    pub link: Link,
}

pub trait OExpression {
    fn to_sql(self) -> String;
}

impl<Table: ToString, Col: ToString> OExpression for scoped_col<Table, Col> {
    fn to_sql(self) -> String {
        format!("`{}_{}`", self.table.to_string(), self.col.to_string())
    }
}

impl<As: ToString, Col: OExpression> OExpression for aliased<Col, As> {
    fn to_sql(self) -> String {
        format!("{} AS `{}`", self.select.to_sql(), self.as_.to_string())
    }
}

pub struct SelectItem<T>(pub T);

impl<T: OExpression> OExpression for SelectItem<Vec<T>> {
    fn to_sql(self) -> String {
        self.0
            .into_iter()
            .map(|e| e.to_sql())
            .collect::<Vec<_>>()
            .join(",")
    }
}
impl<T0: OExpression, T1: OExpression> OExpression for SelectItem<(T0, T1)> {
    fn to_sql(self) -> String {
        vec![self.0.0.to_sql(), self.0.1.to_sql()].join(",")
    }
}

async fn fetch_many<F, W, L>(s: Select<F, (W,), (L,), (), ()>, pool: Pool<Sqlite>) -> String
where
    F: Collection<Id = SingleIncremintalInt>,
    F: Queries<Sqlite>,
    W: Expression<'static, direct_bind<'static, Sqlite>>,
    W: WhereItem<F>,
    L: Link<F>,
    L::Spec: FetchManyLink<Sqlite, SelectItems: OExpression>,
{
    let mut arg = direct_bind::<'static, Sqlite>::default();
    let spec = s.link.0.spec(&s.from);
    let fragment = spec.select_fragment();

    let mut select = vec![];

    select.extend(s.from.select_list().map(|e| {
        format!(
            "`{}`.`{col}` AS `{}_{col}`",
            s.from.table_name(),
            s.from.table_name_lower_case(),
            col = e
        )
    }));

    select.extend(fragment.select_items.into_iter());

    select.extend([format!(
        "`{}`.id AS local_id",
        s.from.table_name().sanitize()
    )]);

    let mut q = format!("SELECT {}", select.join(", "));

    q.push_str(" FROM ");
    q.push_str(s.from.table_name());

    let mut wheres = vec![s.wheres.0.expression(&mut arg)(&mut ())];

    if wheres.is_empty().not() {
        q.push_str(" WHERE ");
        q.push_str(&wheres.join(" AND "));
    }

    use sqlx::Executor;
    let res = pool
        .fetch_many(Executable {
            string: &q,
            arguments: arg.arg,
            db: PhantomData,
        })
        .map(|e| {
            e.map(|e| {
                use sqlx::Row;
                let row = e.right().unwrap();

                CollectionOutput {
                    id: row.get::<'_, i64, _>("local_id"),
                    base: s
                        .from
                        .from_row(&row, &format!("{}_", s.from.table_name_lower_case())),
                    link: (),
                }
            })
        })
        .try_collect::<Vec<_>>()
        .await
        .unwrap();

    q
}

#[tokio::test]
#[allow(unused)]
#[warn(unused_must_use)]
async fn main() {
    let pool = Sqlite::connect_in_memory().await;

    let s = sql!(
        SELECT FROM todo t
        LINK category
        WHERE title.col_eq("tobe".to_string())
    );

    fetch_many(s, pool).await;

    ()
}

#[cfg(skip_without_comment)]
mod one_or_many {
    enum quantity {
        only_one_entry,
        array_of_entries,
    }
    fn retrun_is<F>(select: &Select<F, (), (), (), ()>) -> quantity {
        if select.from.is_collection().not() {
            return quantity::only_one_entry;
        }
        if select.wheres.any(|e| e.is_unique()) {
            return quantity::only_one_entry;
        }
        quantity::array_of_entries
    }

    use crate::_todo;

    use super::col_eq;

    struct One;
    struct Many;
    trait OneOrMany {
        type OneOrMany;
    }
    trait OneOrVec {
        type Gen<T>;
    }
    impl OneOrVec for One {
        type Gen<T> = T;
    }
    impl OneOrVec for Many {
        type Gen<T> = Vec<T>;
    }

    trait UniqueField {
        type OneOrMany;
    }
    impl UniqueField for _todo::title {
        type OneOrMany = Many;
    }
    impl UniqueField for _todo::done {
        type OneOrMany = One;
    }

    // unique field
    impl<F: UniqueField, T> OneOrMany for col_eq<F, T> {
        type OneOrMany = F::OneOrMany;
    }

    impl OneOrMany for (Many, Many) {
        type OneOrMany = Many;
    }

    impl OneOrMany for (One, Many) {
        type OneOrMany = One;
    }

    impl OneOrMany for (Many, One) {
        type OneOrMany = One;
    }

    impl OneOrMany for (One, One) {
        type OneOrMany = One;
    }

    impl OneOrMany for super::todo {
        type OneOrMany = Many;
    }

    fn one_or_many<F, W>(s: (F, W)) -> <(F, W) as OneOrMany>::OneOrMany
    where
        F: OneOrMany,
        (F, W): OneOrMany,
        W: OneOrMany,
        (<F as OneOrMany>::OneOrMany, <W as OneOrMany>::OneOrMany): OneOrMany,
        <(<F as OneOrMany>::OneOrMany, <W as OneOrMany>::OneOrMany) as OneOrMany>::OneOrMany:
            OneOrVec,
    {
        todo!()
    }
    use super::todo;

    fn main() {
        let many: Vec<todo> = one_or_many((super::todo, col_eq(_todo::title, 3)));
        let one: todo = one_or_many((super::todo, col_eq(_todo::done, 3)));
    }
}

pub struct DynamicCol {
    pub members: Vec<String>,
}
impl CollectionBasic for DynamicCol {
    fn table_name(&self) -> &str {
        todo!()
    }
    fn table_name_lower_case(&self) -> &str {
        todo!()
    }
}
impl Collection for DynamicCol {
    type Partial = ();

    type Data = ();

    type Members = ();

    fn members(&self) -> &Self::Members {
        todo!()
    }

    type Id = ();

    fn id(&self) -> &Self::Id {
        todo!()
    }
}
impl Queries<Sqlite> for DynamicCol {
    // fn select_list(&self) -> Vec<String> {
    //     vec![format!("title")]
    // }
    fn select_list<'a>(&'a self) -> impl Iterator<Item = &'a str> + use<'a> {
        self.members.iter().map(|e| e.as_str())
    }
    fn from_row(&self, row: &<Sqlite as Database>::Row, aliase: &str) -> Self::Data
    where
        Sqlite: Database,
    {
        use sqlx::Row;
    }
}

trait JsonCol {
    fn clone_self(&self) -> Box<dyn JsonCol>;
    fn select_list(&self) -> Vec<&str>;
}

impl<T> JsonCol for T
where
    T: Clone + Queries<Sqlite> + 'static,
{
    fn clone_self(&self) -> Box<dyn JsonCol> {
        Box::new(self.clone())
    }

    fn select_list(&self) -> Vec<&str> {
        Queries::select_list(self).collect()
    }
}
impl CollectionBasic for Box<dyn JsonCol> {
    fn table_name(&self) -> &str {
        todo!()
    }

    fn table_name_lower_case(&self) -> &str {
        todo!()
    }
}
impl Collection for Box<dyn JsonCol> {
    type Partial = ();

    type Data = ();

    type Members = ();

    fn members(&self) -> &Self::Members {
        todo!()
    }

    type Id = ();

    fn id(&self) -> &Self::Id {
        todo!()
    }
}
impl Queries<Sqlite> for Box<dyn JsonCol> {
    fn from_row(&self, row: &<Sqlite as Database>::Row, aliase: &str) -> Self::Data
    where
        Sqlite: Database,
    {
        todo!()
    }
    fn select_list<'a>(&'a self) -> impl Iterator<Item = &'a str> + use<'a> {
        JsonCol::select_list(&**self).into_iter()
    }
}
impl Clone for Box<dyn JsonCol> {
    fn clone(&self) -> Self {
        JsonCol::clone_self(&**self)
    }
}
impl ToOwned for dyn JsonCol {
    type Owned = Box<dyn JsonCol>;
    fn to_owned(&self) -> Self::Owned {
        self.clone_self()
    }
}
