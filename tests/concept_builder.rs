#![allow(unexpected_cfgs)]
#![allow(unused)]
#![warn(unused_must_use)]
#![allow(non_camel_case_types)]
#![allow(non_upper_case_globals)]
use std::{any::Any, marker::PhantomData, mem, ops::Not};

use claw_ql::{
    ConnectInMemory,
    collections::{Collection, CollectionBasic, Member, MemberBasic, SingleIncremintalInt},
    execute::Executable,
    links::relation_optional_to_many::optional_to_many,
};
use futures::{StreamExt, TryStreamExt};
use sqlx::{
    ColumnIndex, Database, Decode, Encode, Execute, FromRow, Pool, Sqlite, Type, TypeInfo,
    sqlite::{SqliteArguments, SqliteOwnedBuf, SqliteRow},
};
use tracing::warn;
use tracing_subscriber::{filter::combinator::Or, registry::Data};

// in the future this will include implementation on how escaping and placeholders are
// handled for each database
pub trait DatabaseExt: Database {}
impl<T> DatabaseExt for T where T: Database {}

pub struct BuildContext<'q, S>
where
    S: DatabaseExt,
{
    // this field is becuase I cannot have my Adder::executable to have
    // reciever as `self` but `&'a mut self` instead
    // I believe this is due to a flaw in sqlx's Execute::sql lifetime
    // I just want to handle a bug that may occur for those who are not aware
    pub taken: bool,
    pub(crate) stmt: String,
    pub count: usize,
    pub arg: S::Arguments<'q>,
}

impl<'q, S: DatabaseExt> Default for BuildContext<'q, S> {
    fn default() -> Self {
        BuildContext {
            taken: false,
            stmt: String::new(),
            count: 0,
            arg: S::Arguments::default(),
        }
    }
}

pub struct aliase<T> {
    pub vl: T,
    pub as_: &'static str,
    pub table: &'static str,
}

impl<'q, T, S> Expression<'q, S> for aliase<T>
where
    T: Expression<'q, S>,
{
    fn expression(self, ctx: &mut BuildContext<'q, S>)
    where
        S: DatabaseExt,
    {
        ctx.sanitize(self.table);
        ctx.syntax(".");
        self.vl.expression(ctx);
        ctx.syntax(" AS ");
        ctx.sanitize(self.as_);
    }
}

pub trait SqlPartToSanitize<S> {
    fn to_sql(&self) -> &str;
    fn safe_to_sql(&self) -> bool {
        false
    }
}

pub trait SqlPartSafe {
    fn to_sql(self, str: &mut String);
}

/// usually &'static str are hardcoded in source code at build time
/// sql injection caused by this impl is the developer's fault
/// which is unlikely because sql injection occur with malicious intent
/// ex: `String::new().leak()` or maliciouse build.rs
///
/// should I remove this impl, maybe, this is temp for now
impl SqlPartSafe for &'static str {
    fn to_sql(self, str: &mut String) {
        str.push_str(self);
    }
}

impl<S> SqlPartToSanitize<S> for &'_ str {
    fn to_sql(&self) -> &str {
        self
    }
    fn safe_to_sql(&self) -> bool {
        false
    }
}

impl<S> SqlPartToSanitize<S> for String {
    fn to_sql(&self) -> &str {
        self.as_str()
    }
    fn safe_to_sql(&self) -> bool {
        false
    }
}

impl<S> SqlPartToSanitize<S> for bool {
    fn to_sql(&self) -> &str {
        match self {
            true => "true",
            false => "false",
        }
    }
    fn safe_to_sql(&self) -> bool {
        true
    }
}

pub struct hardcode<T>(pub T);
impl<S, T> SqlPartToSanitize<S> for hardcode<T>
where
    S: DatabaseExt,
    T: Type<S>,
{
    fn to_sql(&self) -> &str {
        todo!("hvae access to DatabaseExt api")
    }
}

// fn hardcode<T: ToString>(t:T)
// // impl<T> hardcode<>

impl<'q, S> BuildContext<'q, S>
where
    S: Database,
{
    pub fn add<V>(&mut self, value: V)
    where
        V: Encode<'q, S> + 'q + Type<S>,
    {
        use sqlx::Arguments;
        self.arg.add(value).expect("when does this ever fail?");
        self.count += 1;
        self.stmt.push_str(format!("${}", self.count).as_str());
    }

    pub fn sanitize<D: SqlPartToSanitize<S>>(&mut self, display: D) {
        if S::NAME != "SQLite" {
            panic!(
                "I only know how to sanitize for sqlite syntax for now! todo: specify behaviour in DatabaseExt trait"
            )
        }
        self.stmt.push('\'');

        let s = display.to_sql();
        let safe = display.safe_to_sql();

        if safe {
            self.stmt.push_str(s);
        } else {
            let mut s = s.chars();
            while let Some(next) = s.next() {
                match next {
                    '\'' => {
                        self.stmt.push(next);
                        self.stmt.push('\'');
                    }
                    '\\' => {
                        self.stmt.push(next);
                        self.stmt.push('\\');
                    }
                    n => self.stmt.push(n),
                }
            }
        }

        self.stmt.push('\'');
    }
    /// push str that is known to not cause sql injection,

    pub fn syntax<D: SqlPartSafe>(&mut self, str: D) {
        str.to_sql(&mut self.stmt);
    }

    #[track_caller]
    pub fn executable(&'q mut self) -> Executable<'q, S, S::Arguments<'q>> {
        if self.taken {
            panic!("should call executable only once!")
        }
        self.taken = true;
        Executable {
            string: self.stmt.as_str(),
            arguments: mem::take(&mut self.arg),
            db: PhantomData,
        }
    }
}

pub trait Expression<'q, S> {
    fn expression(self, ctx: &mut BuildContext<'q, S>)
    where
        S: DatabaseExt;
}

pub trait BoxedExpression<'q, S> {
    fn expression(self: Box<Self>, ctx: &mut BuildContext<'q, S>)
    where
        S: DatabaseExt;
}

impl<'q, S, T> BoxedExpression<'q, S> for T
where
    T: Expression<'q, S>,
{
    fn expression(self: Box<Self>, ctx: &mut BuildContext<'q, S>)
    where
        S: DatabaseExt,
    {
        Expression::expression(*self, ctx);
    }
}

impl<'q, S> Expression<'q, S> for Box<dyn BoxedExpression<'q, S>> {
    fn expression(self, ctx: &mut BuildContext<'q, S>)
    where
        S: DatabaseExt,
    {
        BoxedExpression::expression(self, ctx);
    }
}

pub trait MembersAsBoxedExpression<'q, S> {
    fn members_as_boxed_expretions() -> Vec<Box<dyn BoxedExpression<'q, S>>>;
}

impl<'q, S> MembersAsBoxedExpression<'q, S> for todo
where
    todo_members::title: Expression<'q, S>,
    todo_members::done: Expression<'q, S>,
    todo_members::description: Expression<'q, S>,
{
    fn members_as_boxed_expretions() -> Vec<Box<dyn BoxedExpression<'q, S>>> {
        vec![
            Box::new(todo_members::title),
            Box::new(todo_members::done),
            Box::new(todo_members::description),
        ]
    }
}

impl<'q, S> MembersAsBoxedExpression<'q, S> for category
where
    category_members::title: Expression<'q, S>,
{
    fn members_as_boxed_expretions() -> Vec<Box<dyn BoxedExpression<'q, S>>> {
        vec![Box::new(category_members::title)]
    }
}

#[cfg(feature = "skip_without_commnet")]
mod ok_im_going_too_far_with_generics {
    pub trait ManyExpressions<'q, S> {
        fn expression(self, start: &'static str, join: &'static str, ctx: &mut BuildContext<'q, S>)
        where
            S: DatabaseExt;
    }

    impl<'q, S> ManyExpressions<'q, S> for () {
        fn expression(self, start: &'static str, join: &'static str, ctx: &mut BuildContext<'q, S>)
        where
            S: DatabaseExt,
        {
        }
    }

    // impl<'q, T0, S> ManyExpressions<'q, S> for (T0,)
    // where
    //     T0: Expression<'q, S>,
    // {
    //     fn expression(self, start: &'static str, join: &'static str, ctx: &mut BuildContext<'q, S>)
    //     where
    //         S: DatabaseExt,
    //     {
    //         ctx.push_str(start);
    //         self.0.expression(ctx);
    //     }
    // }
    // impl<'q, T0, T1, S> ManyExpressions<'q, S> for (T0, T1)
    // where
    //     T0: Expression<'q, S>,
    //     T1: Expression<'q, S>,
    // {
    //     fn expression(self, start: &'static str, join: &'static str, ctx: &mut BuildContext<'q, S>)
    //     where
    //         S: DatabaseExt,
    //     {
    //         ctx.push_str(start);
    //         self.0.expression(ctx);
    //         ctx.push_str(join);
    //         self.1.expression(ctx);
    //     }
    // }

    // impl<'q, T, S> ManyExpressions<'q, S> for Vec<T>
    // where
    //     T: Expression<'q, S>,
    // {
    //     fn expression(self, start: &'static str, join: &'static str, ctx: &mut BuildContext<'q, S>)
    //     where
    //         S: DatabaseExt,
    //     {
    //         if self.is_empty().not() {
    //             ctx.push_str(start);
    //         }
    //         let last_index = self.len() - 1;
    //         for (i, si) in self.into_iter().enumerate() {
    //             si.expression(ctx);
    //             if i != last_index {
    //                 ctx.push_str(join)
    //             }
    //         }
    //     }
    // }

    pub struct ManyExpressionsJoined<T> {
        pub start: &'static str,
        pub join: &'static str,
        pub expressions: T,
    }

    impl<'q, S, T> Expression<'q, S> for ManyExpressionsJoined<T>
    where
        T: ManyExpressions<'q, S>,
    {
        fn expression(self, ctx: &mut BuildContext<'q, S>)
        where
            S: DatabaseExt,
        {
            ManyExpressions::expression(self.expressions, self.start, self.join, ctx);
        }
    }

    // impl<'q, T, S> ManyExpressions<'q, S> for Option<T>
    // where
    //     T: Expression<'q, S>,
    // {
    //     fn expression(self, start: &'static str, join: &'static str, ctx: &mut BuildContext<'q, S>)
    //     where
    //         S: DatabaseExt,
    //     {
    //         if let Some(this) = self {
    //             ctx.push_str(start);
    //             this.expression(ctx);
    //         }
    //     }
    // }

    // filter trait to only be applicable for (), T and Option<T>
    pub trait MaybeExpression {}
    impl<T> MaybeExpression for (T,) {}
    impl MaybeExpression for () {}
    impl<T> MaybeExpression for Option<T> {}

    pub trait PossibleExpression<'q, S> {
        fn expression(self, start: &'static str, ctx: &mut BuildContext<'q, S>)
        where
            S: DatabaseExt;
    }

    // impl<'q, S, T> PossibleExpression<'q, S> for (T,)
    // where
    //     T: Expression<'q, S>,
    // {
    //     fn expression(self, start: &'static str, ctx: &mut BuildContext<'q, S>)
    //     where
    //         S: DatabaseExt,
    //     {
    //         ctx.push_str(start);
    //         self.0.expression(ctx);
    //     }
    // }

    impl<'q, S> PossibleExpression<'q, S> for () {
        fn expression(self, start: &'static str, ctx: &mut BuildContext<'q, S>)
        where
            S: DatabaseExt,
        {
        }
    }

    // impl<'q, S, T> PossibleExpression<'q, S> for Option<T>
    // where
    //     T: Expression<'q, S>,
    // {
    //     fn expression(self, start: &'static str, ctx: &mut BuildContext<'q, S>)
    //     where
    //         S: DatabaseExt,
    //     {
    //         if let Some(this) = self {
    //             ctx.push_str(start);
    //             this.expression(ctx);
    //         }
    //     }
    // }

    pub struct PossibleExpressionJoined<T> {
        pub start: &'static str,
        pub expressions: T,
    }

    impl<'q, S, T> Expression<'q, S> for PossibleExpressionJoined<T>
    where
        T: PossibleExpression<'q, S>,
    {
        fn expression(self, ctx: &mut BuildContext<'q, S>)
        where
            S: DatabaseExt,
        {
            self.expressions.expression(self.start, ctx);
        }
    }

    impl<'q, S, SelectItems, From, Joins, Wheres, Limit, Order> Expression<'q, S>
        for SelectStatement<SelectItems, From, Joins, Wheres, Limit, Order>
    where
        SelectItems: ManyExpressions<'q, S> + 'q,
        From: Expression<'q, S> + 'q,
        Joins: ManyExpressions<'q, S> + 'q,
        Wheres: ManyExpressions<'q, S> + 'q,
        Limit: PossibleExpression<'q, S> + 'q,
        Order: PossibleExpression<'q, S> + 'q,
    {
        fn expression(self, ctx: &'q mut BuildContext<'q, S>)
        where
            S: DatabaseExt,
        {
            ctx.push_str("SELECT ");
            self.select_items.expression("", ", ", ctx);
            ctx.push_str(" FROM ");
            self.from.expression(ctx);
            self.joins.expression("", "", ctx);
            self.wheres.expression(" WHERE ", " AND ", ctx);
            self.order.expression(" ORDER ", ctx);
            self.limit.expression(" LIMIT ", ctx);
            ctx.push_str(";");
        }
    }

    async fn woundt_compile() {
        let op = FetchMany {
            from: todo,
            wheres: {
                let v = vec![];
                v as Vec<Box<dyn BoxedExpression<'_, Sqlite>>>
            },
            links: {
                let v = vec![];
                v as Vec<Box<dyn LiquidLinking<Sqlite>>>
            },
            limit: None::<Box<dyn BoxedExpression<'_, Sqlite>>>,
            order: None::<Box<dyn BoxedExpression<'_, Sqlite>>>,
        };

        let mut first_statement = SelectStatement {
            select_items: (),
            from: todo, // op.from,
            joins: (),
            wheres: (),
            order: (), //op.order,
            limit: (), //op.limit,
        };

        // op.links.extend first_statment
        // op.from.extend first_statment.select_items

        let mut build_ctx = BuildContext::default();

        first_statement.expression(&mut build_ctx);

        let pool = Sqlite::connect_in_memory().await;

        use sqlx::Executor;
        let s = pool
            .fetch_many(build_ctx.executable())
            .map(|e| {
                e.map(|e| {
                    use sqlx::Row;
                    let row = e.right().unwrap();

                    row
                })
            })
            .try_collect::<Vec<_>>()
            .await
            .unwrap();
    }
}

pub struct col_eq<Col, Eq> {
    pub col: Col,
    pub eq: Eq,
}

impl<'q, S, Col, Eq> Expression<'q, S> for col_eq<Col, Eq>
where
    S: DatabaseExt,
    Eq: 'q + Encode<'q, S> + Type<S>,
    Col: SqlPartToSanitize<S>,
{
    fn expression(self, arg: &mut BuildContext<'q, S>) {
        arg.sanitize(self.col);
        arg.syntax(" = ");
        arg.add(self.eq);
    }
}

pub trait FromRowExt<'r, R> {
    fn from_row_no_alias(row: &'r R) -> Result<Self, sqlx::Error>
    where
        Self: Sized;
    fn from_row_pre_aliased(row: &'r R, alias: &str) -> Result<Self, sqlx::Error>
    where
        Self: Sized;
}

impl<'r, R: sqlx::Row> FromRowExt<'r, R> for Todo
where
    String: Type<R::Database> + Decode<'r, R::Database>,
    bool: Type<R::Database> + Decode<'r, R::Database>,
    Option<String>: Type<R::Database> + Decode<'r, R::Database>,
    // i'm not sure if this would cause issues, because in FromRow the lifetime is specific to 'r
    for<'a> &'a str: ColumnIndex<R>,
{
    fn from_row_no_alias(row: &'r R) -> Result<Self, sqlx::Error>
    where
        Self: Sized,
    {
        Ok(Todo {
            title: row.try_get("title")?,
            done: row.try_get("done")?,
            description: row.try_get("description")?,
        })
    }
    fn from_row_pre_aliased(row: &'r R, alias: &str) -> Result<Self, sqlx::Error>
    where
        Self: Sized,
    {
        Ok(Todo {
            title: row.try_get(format!("{}title", alias).as_str())?,
            done: row.try_get(format!("{}done", alias).as_str())?,
            description: row.try_get(format!("{}description", alias).as_str())?,
        })
    }
}

impl<'r, R: sqlx::Row> FromRowExt<'r, R> for Category
where
    String: Type<R::Database> + Decode<'r, R::Database>,
    for<'a> &'a str: ColumnIndex<R>,
{
    fn from_row_no_alias(row: &'r R) -> Result<Self, sqlx::Error>
    where
        Self: Sized,
    {
        Ok(Category {
            title: row.try_get("title")?,
        })
    }
    fn from_row_pre_aliased(row: &'r R, alias: &str) -> Result<Self, sqlx::Error>
    where
        Self: Sized,
    {
        Ok(Category {
            title: row.try_get(format!("{}title", alias).as_str())?,
        })
    }
}

#[derive(sqlx::FromRow, claw_ql_macros::Collection)]
pub struct Todo {
    pub title: String,
    pub done: bool,
    pub description: Option<String>,
}

#[derive(claw_ql_macros::Collection)]
pub struct Category {
    pub title: String,
}

macro_rules! impl_for_collection {
    ($ident:ty) => {
        impl Expression<'static, Sqlite> for $ident {
            fn expression(self, arg: &mut BuildContext<'static, Sqlite>)
            where
                Sqlite: DatabaseExt,
            {
                arg.sanitize(self);
            }
        }

        impl<S> SqlPartToSanitize<S> for $ident {
            fn safe_to_sql(&self) -> bool {
                true
            }
            fn to_sql(&self) -> &str {
                self.table_name()
            }
        }
    };
}

macro_rules! impl_for_member {
    ($ident:ty) => {
        impl Expression<'static, Sqlite> for $ident {
            fn expression(self, arg: &mut BuildContext<'static, Sqlite>)
            where
                Sqlite: DatabaseExt,
            {
                arg.sanitize(self);
            }
        }

        impl<S> SqlPartToSanitize<S> for $ident {
            fn safe_to_sql(&self) -> bool {
                true
            }
            fn to_sql(&self) -> &str {
                self.name()
            }
        }
    };
}

impl_for_collection!(todo);
impl_for_member!(todo_members::title);
impl_for_member!(todo_members::done);
impl_for_member!(todo_members::description);
impl_for_collection!(category);
impl_for_member!(category_members::title);

pub trait Members {}

pub struct SelectStatement<SelectItems, From, Joins, Wheres, Order, Limit> {
    pub select_items: SelectItems,
    pub from: From,
    pub joins: Joins,
    pub wheres: Wheres,
    pub order: Order,
    pub limit: Limit,
}

impl<'q, S> Expression<'q, S>
    for SelectStatement<
        Vec<Box<dyn BoxedExpression<'q, S> + 'q>>,
        Box<dyn BoxedExpression<'q, S> + 'q>,
        Vec<Box<dyn BoxedExpression<'q, S> + 'q>>,
        Vec<Box<dyn BoxedExpression<'q, S> + 'q>>,
        Option<Box<dyn BoxedExpression<'q, S> + 'q>>,
        Option<Box<dyn BoxedExpression<'q, S> + 'q>>,
    >
{
    fn expression(self, ctx: &mut BuildContext<'q, S>)
    where
        S: DatabaseExt,
    {
        ctx.syntax("SELECT ");
        let l = self.select_items.len();
        let last_i = if l == 0 { 0 } else { l - 1 };
        for (i, item) in self.select_items.into_iter().enumerate() {
            item.expression(ctx);
            if i != last_i {
                ctx.syntax(", ");
            }
        }

        ctx.syntax(" FROM ");
        self.from.expression(ctx);

        for item in self.joins {
            item.expression(ctx);
        }
        let last_i = if self.wheres.is_empty().not() {
            ctx.syntax(" WHERE ");
            self.wheres.len() - 1
        } else {
            0
        };
        for (i, item) in self.wheres.into_iter().enumerate() {
            item.expression(ctx);
            if i != last_i {
                ctx.syntax(" AND ");
            }
        }

        if let Some(item) = self.order {
            ctx.syntax(" ORDER ");
            item.expression(ctx);
        }

        if let Some(item) = self.limit {
            ctx.syntax(" LIMIT ");
            item.expression(ctx);
        }

        ctx.syntax(";");
    }
}

pub struct CreateTable<Init, TableName, Members> {
    pub init: Init,
    pub name: TableName,
    pub members: Members,
}

pub struct create_if_not_exist;

impl<'q, S> Expression<'q, S> for create_if_not_exist {
    fn expression(self, ctx: &mut BuildContext<'q, S>)
    where
        S: DatabaseExt,
    {
        ctx.syntax("CREATE TABLE IF NOT EXISTS");
    }
}

impl<'q, S> Expression<'q, S>
    for CreateTable<
        create_if_not_exist,
        String,
        Vec<(String, Vec<Box<dyn BoxedExpression<'q, S>>>)>,
    >
{
    fn expression(self, ctx: &mut BuildContext<'q, S>)
    where
        S: DatabaseExt,
    {
        let open_b = "(";
        let close_b = ");";
        self.init.expression(ctx);
        ctx.syntax(" ");
        ctx.sanitize(self.name);
        ctx.syntax(" ");
        ctx.syntax(open_b);
        let last_i = self.members.len();
        let last_i = if last_i == 0 { 0 } else { last_i - 1 };
        for (i, each) in self.members.into_iter().enumerate() {
            ctx.sanitize(each.0);

            {
                let last_i = each.1.len();
                let last_i = if last_i == 0 {
                    0
                } else {
                    ctx.syntax(" ");
                    last_i - 1
                };
                for (i, each) in each.1.into_iter().enumerate() {
                    each.expression(ctx);
                    if i != last_i {
                        ctx.syntax(" ");
                    }
                }
            }

            if i != last_i {
                ctx.syntax(", ");
            }
        }
        ctx.syntax(close_b);
    }
}

pub struct sqlx_type<T>(PhantomData<T>);
impl<'q, S, T> Expression<'q, S> for sqlx_type<T>
where
    S: Database,
    T: Type<S>,
{
    fn expression(self, ctx: &mut BuildContext<'q, S>)
    where
        S: DatabaseExt,
    {
        ctx.stmt.push_str(T::type_info().name());
    }
}

fn create_table_for_todo<'q, S>()
-> CreateTable<create_if_not_exist, String, Vec<(String, Vec<Box<dyn BoxedExpression<'q, S>>>)>>
where
    S: Database,
    String: Type<S>,
    Option<String>: Type<S>,
    bool: Type<S>,
{
    CreateTable {
        init: create_if_not_exist,
        name: String::from(todo.table_name()),
        members: vec![
            (
                "id".to_string(),
                vec![Box::new(sqlx_type::<String>(PhantomData))],
            ),
            (
                todo_members::title.name().to_string(),
                vec![Box::new(sqlx_type::<String>(PhantomData))],
            ),
            (
                todo_members::description.name().to_string(),
                vec![Box::new(sqlx_type::<Option<String>>(PhantomData))],
            ),
            (
                todo_members::done.name().to_string(),
                vec![Box::new(sqlx_type::<bool>(PhantomData))],
            ),
        ],
    }
}

pub struct FetchMany<From, Links, Wheres, Order, Limit> {
    pub from: From,
    // extendable
    pub wheres: Wheres,
    // extendable and generate data
    pub links: Links,
    // one-timer
    pub limit: Order,
    pub order: Limit,
}

pub struct SelectStatementExtendableParts<S, J, W> {
    pub select_items: S,
    /// joins has to be non duplicating in order to be extendable
    /// otherwise I have to rewrite the code that uses this struct
    ///
    /// example of duplicating joins is optional_to_many RIGHT JOIN
    pub non_duplicating_joins: J,
    pub wheres: W,
}

impl<'q, S: 'static>
    SelectStatementExtendableParts<
        Vec<Box<dyn BoxedExpression<'q, S> + 'q>>,
        Vec<Box<dyn BoxedExpression<'q, S> + 'q>>,
        Vec<Box<dyn BoxedExpression<'q, S> + 'q>>,
    >
{
    #[rustfmt::skip]
    fn merge(&mut self, other: Self) {
        self.wheres.extend(other.wheres);
        self.select_items.extend(other.select_items);
        self.non_duplicating_joins.extend(other.non_duplicating_joins);
    }
}

pub trait LinkFetchOne<S> {
    fn extend1(
        &self,
    ) -> SelectStatementExtendableParts<
        Vec<Box<dyn BoxedExpression<'static, S>>>,
        Vec<Box<dyn BoxedExpression<'static, S>>>,
        Vec<Box<dyn BoxedExpression<'static, S>>>,
    >;

    type FromRow;
    fn extend2(&self) -> Self::FromRow {
        todo!()
    }

    type Inner;
    type SubOp;
    fn sub_op(&self, row: &<S as Database>::Row) -> (Self::SubOp, Self::Inner)
    where
        S: Database;

    type Output;
    fn take(self, extend: Self::FromRow, inner: Self::Inner) -> Self::Output;
}

pub struct col<T>(pub T);
impl<'q, S> Expression<'q, S> for col<&'static str> {
    fn expression(self, ctx: &mut BuildContext<'q, S>)
    where
        S: DatabaseExt,
    {
        ctx.sanitize(self.0);
    }
}

pub struct left_join {
    pub ft: String,
    pub fc: String,
    pub lt: String,
    pub lc: String,
}

impl<'a, S> Expression<'a, S> for left_join {
    fn expression(self, ctx: &mut BuildContext<'a, S>)
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

impl<S, F, T> LinkFetchOne<S> for optional_to_many<F, T>
where
    T: Collection<Id = SingleIncremintalInt>,
    F: Collection,
    S: Database,
    T: MembersAsBoxedExpression<'static, S>,
{
    fn extend1(
        &self,
    ) -> SelectStatementExtendableParts<
        Vec<Box<dyn BoxedExpression<'static, S>>>,
        Vec<Box<dyn BoxedExpression<'static, S>>>,
        Vec<Box<dyn BoxedExpression<'static, S>>>,
    > {
        let mut select_items = T::members_as_boxed_expretions();
        select_items.push(Box::new(col("category_id")));
        SelectStatementExtendableParts {
            select_items,
            non_duplicating_joins: vec![Box::new(left_join {
                ft: self.to.table_name().to_string(),
                fc: "id".to_string(),
                lt: self.from.table_name().to_string(),
                lc: self.foriegn_key.to_string(),
            })],
            wheres: vec![],
        }
    }

    type FromRow = T::Data;

    type Inner = ();

    type SubOp = ();

    fn sub_op(&self, row: &<S as Database>::Row) -> (Self::SubOp, Self::Inner) {
        todo!()
    }

    type Output = ();

    fn take(self, extend: Self::FromRow, inner: Self::Inner) -> Self::Output {
        todo!()
    }
}

struct local_col<T>(pub T);
impl<'q, S, T> Expression<'q, S> for local_col<T>
where
    T: SqlPartToSanitize<S>,
{
    fn expression(self, ctx: &mut BuildContext<'q, S>)
    where
        S: DatabaseExt,
    {
        warn!(
            "todo: have better implementation for local_col 1.create col(..).alias(..) 2. handle namming conflicts"
        );
        ctx.sanitize(self.0);
        ctx.syntax(".id AS local_id");
    }
}

pub trait LiquidLinking<S> {}

#[tokio::test]
async fn main() {
    let op = FetchMany {
        from: todo,
        wheres: {
            let v = vec![];
            v as Vec<Box<dyn BoxedExpression<'_, Sqlite>>>
        },
        links: category,
        limit: None::<Box<dyn BoxedExpression<'_, Sqlite>>>,
        order: None::<Box<dyn BoxedExpression<'_, Sqlite>>>,
    };

    // links.spec(&op.from)
    let link_spec = optional_to_many {
        foriegn_key: String::from("category_id"),
        from: todo,
        to: category,
    };

    let mut first_statement: SelectStatement<
        Vec<Box<dyn BoxedExpression<'_, Sqlite>>>,
        Box<dyn BoxedExpression<'_, Sqlite>>,
        Vec<Box<dyn BoxedExpression<'_, Sqlite>>>,
        Vec<Box<dyn BoxedExpression<'_, Sqlite>>>,
        Option<Box<dyn BoxedExpression<'_, Sqlite>>>,
        Option<Box<dyn BoxedExpression<'_, Sqlite>>>,
    > = SelectStatement {
        select_items: vec![
            Box::new(local_col(todo)),
            Box::new(aliase {
                vl: todo_members::title,
                as_: "todo_title",
                table: "Todo",
            }),
            Box::new(aliase {
                vl: todo_members::done,
                as_: "todo_done",
                table: "Todo",
            }),
            Box::new(aliase {
                vl: todo_members::description,
                as_: "todo_description",
                table: "Todo",
            }),
        ],
        from: Box::new(op.from), // op.from,
        joins: vec![],
        wheres: vec![],
        order: None, //op.order,
        limit: None, //op.limit,
    };

    // link to_extend statment
    {
        let extend_statment = link_spec.extend1();
        first_statement.select_items.extend(
            extend_statment
                .select_items
                .into_iter()
                .map(|e| {
                    Box::new(aliase {
                        vl: e,
                        as_: "link2",
                        table: "Category",
                    }) as Box<dyn BoxedExpression<'_, Sqlite>>
                })
                .collect::<Vec<_>>(),
        );
        first_statement
            .joins
            .extend(extend_statment.non_duplicating_joins);
        first_statement.wheres.extend(extend_statment.wheres);
    }

    let mut build_ctx = BuildContext::default();
    first_statement.expression(&mut build_ctx);

    let pool = Sqlite::connect_in_memory().await;

    {
        // let mut ctx = BuildContext::<'_, Sqlite>::default();
        // let s = create_table_for_todo().expression(&mut ctx);
        let s = pool
            .execute(Executable {
                string: &format!(
                    "
                    CREATE Table Todo (
                        id INT PRIMARY KEY,
                        title TEXT NOT NULL,
                        done INT NOT NULL,
                        description TEXT
                    );
                    CREATE Table Category (
                        id INT PRIMARY KEY,
                        title TEXT NOT NULL
                    );
                    
                    ALTER TABLE Todo ADD COLUMN category_id INT
                        CONSTRAINT category_id
                        REFERENCES Category(id)
                        ON DELETE SET NULL;
                    
                    INSERT INTO Todo (title, done, description)
                    VALUES ('hi', true, 'whatup');

                "
                ),
                arguments: Default::default(),
                db: PhantomData,
            })
            .await
            .unwrap();
    }

    use sqlx::Executor;
    struct output_of_first_st<Id, Members, E2> {
        id: Id,
        members: Members,
        extend2: E2,
    }
    let s = pool
        .fetch_optional(Executable {
            string: build_ctx.stmt.as_str(),
            arguments: mem::take(&mut build_ctx.arg),
            db: PhantomData,
        })
        .await
        .map_err(|e| format!("error: {e}, stmt: {}", build_ctx.stmt.as_str()))
        .unwrap()
        .map(|row| {
            use sqlx::Row;
            output_of_first_st {
                id: row.get::<i32, _>("local_id"),
                members: <Todo as FromRowExt<SqliteRow>>::from_row_pre_aliased(&row, "todo_")
                    .map_err(|e| {
                        format!(
                            "didn't find row with given name {e}, available {:?}",
                            row.columns()
                        )
                    })
                    .unwrap(),
                extend2:
                    <<optional_to_many<todo, category> as LinkFetchOne<Sqlite>>::FromRow as FromRowExt<SqliteRow>>::from_row_pre_aliased(
                        &row,
                        "link_1_"
                    ).unwrap(),
            }
        })
        .unwrap();

    let s = LinkFetchOne::<Sqlite>::take(link_spec, s.extend2, ());
}
