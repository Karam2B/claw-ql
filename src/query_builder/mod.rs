#![allow(non_camel_case_types)]
#![allow(unexpected_cfgs)]

use sqlx::{Encode, Type};

use crate::{
    database_extention::DatabaseExt,
    query_builder::functional_expr::{BoxedExpression, StaticExpression},
};
pub mod functional_expr;
pub mod sanitize;
pub mod syntax;

pub struct QueryBuilder<'q, S>
where
    S: DatabaseExt,
{
    stmt: String,
    count: usize,
    arg: S::Arguments<'q>,
}

impl<'q, S: DatabaseExt> QueryBuilder<'q, S> {
    pub fn stmt(&self) -> &str {
        &self.stmt
    }

    pub fn new<Expr>(expr: Expr) -> Self
    where
        Expr: Expression<'q, S>,
    {
        let mut this = Self::default();

        expr.expression(&mut this);

        this
    }
}

impl<'q, S: DatabaseExt> Default for QueryBuilder<'q, S> {
    fn default() -> Self {
        QueryBuilder {
            stmt: String::new(),
            count: 0,
            arg: S::Arguments::default(),
        }
    }
}

pub trait SqlSanitize<S> {
    fn to_sql(&self, strs: &mut String) {
        panic!()
    }
}

pub trait SqlSyntax {
    fn to_sql(&self, str: &mut String);
}

/// assert is not Expression is not empty
/// prevent downstream crate from `impl Expression<LocalType> for ForiegnTypes`
pub trait OpExpression {}

/// Representing an sql string/expression/statement that is not empty
///
/// # Type Generics `S`
/// representing types that implement `sqlx::Database` like `Sqlite` and `MySQL`
///
/// # lifetime Generics `'q`
/// represent the ability to send a reference to an in-memory database
///
/// example
/// ```rust
///     use sqlx::Sqlite;
///     use claw_ql::{
///         connect_in_memory::ConnectInMemory,
///         query_builder::{Expression, OpExpression, QueryBuilder},
///         use_executor,
///     };
///
///     struct Str<'a>(&'a str);
///     impl<'q> OpExpression for Str<'q> {}
///     impl<'q> Expression<'q, Sqlite> for Str<'q> {
///         fn expression(self, ctx: &mut QueryBuilder<'q, Sqlite>) {
///             ctx.syntax(&"SELECT ");
///             ctx.bind(self.0);
///             ctx.syntax(&";");
///         }
///     }
///
///     #[tokio::main]
///     async fn main() {
///         let pool = Sqlite::connect_in_memory().await;
///
///         let mut statment = String::from("hello world");
///
///         let holding_lifetime = QueryBuilder::new(Str(
///             &   /*'statment*/   statment
///         ));
///         
///         // restricted region
///         // let _ = &mut statment
///
///         let lifetime_droped = use_executor!(fetch_one(&pool, holding_lifetime)).unwrap();
///     }
/// ```
///
/// the only reason why there is lifetime in expression interface is: because in Sqlite you can send a string reference to an in-memory-database (instead of serializing the ref to and owned String and sending it over the netword like MySQL and PostgreQL)
///
/// in fact all impelentation of `sqlx::Encode` (used internally by `QueryBuilder::bind`) are static expect for `impl<'s> Encode<'s, Sqlite> for &'s`
///
/// this lifetime itroduce restriction on all references made between `holding_lifetime` and `lifetime_droped`
///
pub trait Expression<'q, S>: OpExpression + 'q {
    fn expression(self, ctx: &mut QueryBuilder<'q, S>)
    where
        S: DatabaseExt;
}

impl OpExpression for String {}
impl<'q, S> Expression<'q, S> for String
where
    S: DatabaseExt,
{
    fn expression(self, ctx: &mut QueryBuilder<'q, S>)
    where
        S: DatabaseExt,
    {
        ctx.sanitize(self.as_str());
    }
}

impl OpExpression for &'_ str {}
impl<'q, 's, S> Expression<'q, S> for &'s str
where
    's: 'q,
    S: DatabaseExt,
{
    fn expression(self, ctx: &mut QueryBuilder<'q, S>)
    where
        S: DatabaseExt,
    {
        ctx.sanitize(self);
    }
}

pub struct ExpressionAsBind<T>(pub T);
impl<T> OpExpression for ExpressionAsBind<T> {}
impl<'q, S, T> Expression<'q, S> for ExpressionAsBind<T>
where
    S: sqlx::Database,
    T: 'q + sqlx::Type<S> + sqlx::Encode<'q, S>,
{
    fn expression(self, ctx: &mut QueryBuilder<'q, S>)
    where
        S: DatabaseExt,
    {
        ctx.bind(self.0);
    }
}

pub trait IsOpExpression {
    fn is_op(&self) -> bool;
}

pub trait PossibleExpression<'q, S>: IsOpExpression + 'q {
    fn expression_starting<Start: SqlSyntax + ?Sized>(
        self,
        start: &Start,
        ctx: &mut QueryBuilder<'q, S>,
    ) where
        S: DatabaseExt;
    fn expression(self, ctx: &mut QueryBuilder<'q, S>)
    where
        S: DatabaseExt;
}

pub trait ToStaticExpressions<S> {
    fn to_static_expr(self) -> Vec<Box<dyn StaticExpression<S> + Send>>
    where
        Self: Sized;
}

pub trait ManyExpressions<'q, S>: IsOpExpression + 'q {
    fn to_expr(self) -> Vec<Box<dyn BoxedExpression<'q, S> + 'q>>
    where
        Self: Sized;
    fn expression<Start: SqlSyntax + ?Sized, Join: SqlSyntax + ?Sized>(
        self,
        start: &Start,
        join: &Join,
        ctx: &mut QueryBuilder<'q, S>,
    ) where
        S: DatabaseExt;
}

impl<'q, S> ManyExpressions<'q, S> for () {
    fn to_expr(self) -> Vec<Box<dyn BoxedExpression<'q, S> + 'q>>
    where
        Self: Sized,
    {
        vec![]
    }
    fn expression<Start: SqlSyntax + ?Sized, Join: SqlSyntax + ?Sized>(
        self,
        _: &Start,
        _: &Join,
        _: &mut QueryBuilder<'q, S>,
    ) where
        S: DatabaseExt,
    {
    }
}

impl<'q, S, T> ManyExpressions<'q, S> for T
where
    T: Expression<'q, S> + 'q,
{
    fn to_expr(self) -> Vec<Box<dyn BoxedExpression<'q, S> + 'q>>
    where
        Self: Sized,
    {
        vec![Box::new(self)]
    }

    fn expression<Start: SqlSyntax + ?Sized, Join: SqlSyntax + ?Sized>(
        self,
        start: &Start,
        _: &Join,
        ctx: &mut QueryBuilder<'q, S>,
    ) where
        S: DatabaseExt,
    {
        ctx.syntax(start);
        Expression::expression(self, ctx);
    }
}

impl<'q, S> QueryBuilder<'q, S>
where
    S: DatabaseExt,
{
    pub fn bind<V>(&mut self, value: V)
    where
        V: Encode<'q, S> + 'q + Type<S>,
    {
        use sqlx::Arguments;
        self.arg.add(value).expect("when does this ever fail?");
        self.count += 1;
        self.stmt.push_str(format!("${}", self.count).as_str());
    }

    pub fn sanitize(&mut self, display: &str) {
        S::sanitize(display, &mut self.stmt);
    }

    /// push str that is known to not cause sql injection,
    pub fn syntax<D: SqlSyntax + ?Sized>(&mut self, str: &D) {
        str.to_sql(&mut self.stmt);
    }

    pub fn unwrap(self) -> (String, S::Arguments<'q>) {
        (self.stmt, self.arg)
    }
}

#[cfg(feature = "skip_without_comments")]
mod mut_query_builder {
    use std::hash::BuildHasher;

    use crate::{
        OpExpression,
        expressions::{col, col_eq},
    };

    pub trait MutBuilder<'q> {
        type Bound: 'static;
        type Ctx1: BuilderCtx1<'q, Builder = Self>;
        type Ctx2: BuilderCtx2<'q, Builder = Self>;
    }

    pub trait BuilderCtx1<'q> {
        type Builder: MutBuilder<'q, Ctx1 = Self>;
        fn bind<V>(&mut self, value: V) -> <Self::Builder as MutBuilder<'q>>::Bound;
    }
    pub trait BuilderCtx2<'q> {
        type Builder: MutBuilder<'q, Ctx2 = Self>;
        fn sanitize<S>(&mut self, san: S);
        fn bind(&mut self, san: <Self::Builder as MutBuilder<'q>>::Bound);
        fn syntax<S>(&mut self, san: S);
    }

    pub trait MutExpression<'q, S: MutBuilder<'q>>: 'q {
        type Intermid: 'static;
        fn stage_1(self, ctx: &mut S::Ctx1) -> Self::Intermid;
        fn stage_2(this: Self::Intermid, ctx: &mut S::Ctx2);
    }

    impl<'q, S, Col, Eq> MutExpression<'q, S> for col_eq<Col, Eq>
    where
        Col: 'static + AsRef<str>,
        Eq: 'q,
        S: MutBuilder<'q>,
    {
        type Intermid = col_eq<Col, S::Bound>;
        fn stage_1(self, ctx: &mut <S as MutBuilder<'q>>::Ctx1) -> Self::Intermid {
            let eq = ctx.bind(self.eq);
            col_eq { col: self.col, eq }
        }
        fn stage_2(this: Self::Intermid, ctx: &mut <S as MutBuilder<'q>>::Ctx2) {
            ctx.sanitize(this.col.as_ref());
            ctx.syntax(" = ");
            ctx.bind(this.eq);
        }
    }

    /// keep old api just for reference while I'm building MutQueryBuilder
    mod old_api {

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
            fn expression_to_fragment(&mut self, expression: T)
            -> <Self as QueryBuilder>::Fragment;
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

        pub trait MutExpression<'q, S> {
            type Rest;
            fn stage_1(self, bind_ctx: &mut BindBuilder<'q, S>) -> Self::Rest
            where
                S: DatabaseExt;
            fn stage_2(rest: Self::Rest, syntax_ctx: &mut SyntaxBuilder<'q, S>)
            where
                S: DatabaseExt;
        }

        impl<'q, S, T> MutExpression<'q, S> for T
        where
            T: Expression<'q, S>,
        {
            type Rest = RestDefaultImpl;
            fn stage_1(self, bind_ctx: &mut BindBuilder<'q, S>) -> Self::Rest
            where
                S: DatabaseExt,
            {
                RestDefaultImpl(self.expression(bind_ctx))
            }

            fn stage_2(rest: Self::Rest, syntax_ctx: &mut SyntaxBuilder<'q, S>)
            where
                S: DatabaseExt,
            {
                rest.resume(syntax_ctx)
            }
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
        mod sanitize {

            use sqlx::Sqlite;

            use crate::{Expression, QueryBuilder, SanitzingMechanisim};

            pub trait SanitizeAndHardcode<Escape> {
                fn sanitize(&self) -> String;
            }

            pub struct by_double_quote;

            /// explicitly hardcode the inner value
            pub struct hardcode<T>(pub T);

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

            impl<'q, Q, T> Expression<'q, Q> for hardcode<T>
            where
                Q: QueryBuilder + SanitzingMechanisim,
                T: SanitizeAndHardcode<Q::SanitzingMechanisim> + 'q,
            {
                fn expression(
                    self,
                    ctx: &mut Q,
                ) -> impl FnOnce(&mut <Q>::Context) -> String + 'q + use<'q, Q, T>
                where
                    Q: QueryBuilder,
                {
                    move |_| self.0.sanitize()
                }
            }
        }

        mod direct_builder {

            use std::{iter::StepBy, marker::PhantomData};

            use sqlx::{Arguments, Database, Encode, Type, sqlite::SqliteArguments};

            use crate::{
                EncodeExtention, Expression, ExpressionToFragment, QueryBuilder,
                SanitzingMechanisim,
            };

            pub struct direct_bind<'q, S: Database> {
                pub increment: usize,
                pub arg: S::Arguments<'q>,
                pub db: PhantomData<S>,
            }

            impl<'q, S: Database> direct_bind<'q, S> {
                pub fn new(db: S) -> Self {
                    Self {
                        increment: Default::default(),
                        arg: Default::default(),
                        db: PhantomData,
                    }
                }
            }

            impl<'q, S: Database> Default for direct_bind<'q, S> {
                fn default() -> Self {
                    Self {
                        increment: Default::default(),
                        arg: Default::default(),
                        db: PhantomData,
                    }
                }
            }

            impl<'q, S: Database> QueryBuilder for direct_bind<'q, S> {
                type Fragment = String;
                type Context = ();
                type SqlxDb = S;
                type Output = S::Arguments<'q>;

                fn fragment_to_string(ctx: &mut Self::Context, from: Self::Fragment) -> String {
                    from
                }
                fn to_output(
                    self,
                    statement_builder: impl FnOnce(&mut Self::Context) -> String,
                ) -> (String, Self::Output) {
                    (statement_builder(&mut ()), self.arg)
                }
            }

            impl<'q, S: Database, T> EncodeExtention<'q, T> for direct_bind<'q, S>
            where
                T: Encode<'q, S> + Type<S> + Send + 'q,
            {
                fn encode(
                    &mut self,
                    val: T,
                ) -> impl FnOnce(&mut Self::Context) -> String + 'q + use<'q, T, S>
                {
                    self.arg.add(val).expect("bug maybe?");
                    self.increment += 1;
                    let increment = self.increment;
                    move |_| format!("${}", increment)
                }
            }

            impl<'q, E, S> ExpressionToFragment<'q, E> for direct_bind<'q, S>
            where
                E: Expression<'q, Self>,
                S: Database,
            {
                fn expression_to_fragment(&mut self, t: E) -> <Self as QueryBuilder>::Fragment {
                    t.expression(self)(&mut ())
                }
            }

            impl<'q, S> SanitzingMechanisim for direct_bind<'q, S>
            where
                S: Database + SanitzingMechanisim,
            {
                type SanitzingMechanisim = S::SanitzingMechanisim;
            }
        }

        mod defered_builder {

            use std::marker::PhantomData;

            use sqlx::{Database, Encode, database::HasStatementCache, prelude::Type};

            use crate::{
                EncodeExtention, Expression, ExpressionToFragment, PositionalPlaceholder,
                QueryBuilder, SanitzingMechanisim,
            };

            pub struct DeferedCtx<S: Database> {
                pub ctx1: Vec<Option<Box<dyn Stored<S>>>>,
                pub output: S::Arguments<'static>,
            }

            pub struct DeferedFragment<S: Database>(
                pub Box<dyn FnOnce(&mut DeferedCtx<S>) -> String>,
            );

            impl<S: Database> DeferedFragment<S> {
                pub fn into_string(self, ctx: &mut DeferedCtx<S>) -> String {
                    self.0(ctx)
                }
            }

            pub trait Stored<S>: 'static {
                fn bind_stored(self: Box<Self>, ctx: &mut S::Arguments<'static>)
                where
                    S: Database;
            }

            impl<S, T> Stored<S> for T
            where
                S: sqlx::Database,
                T: Type<S> + Encode<'static, S> + 'static + Send,
            {
                #[inline]
                fn bind_stored(self: Box<Self>, ctx: &mut S::Arguments<'static>) {
                    use sqlx::Arguments;
                    ctx.add(*self).expect("internal bug, maybe?");
                }
            }

            pub struct defered_binder<S> {
                pub stored: Vec<Option<Box<dyn Stored<S>>>>,
                pub db: PhantomData<S>,
            }

            impl<S> Default for defered_binder<S> {
                fn default() -> Self {
                    defered_binder {
                        stored: Default::default(),
                        db: PhantomData,
                    }
                }
            }

            impl<S: Database> QueryBuilder for defered_binder<S> {
                type SqlxDb = S;
                type Fragment = DeferedFragment<S>;

                type Context = DeferedCtx<S>;

                type Output = S::Arguments<'static>;

                fn fragment_to_string(ctx: &mut Self::Context, fragment: Self::Fragment) -> String {
                    fragment.into_string(ctx)
                }
                fn to_output(
                    self,
                    statement_builder: impl FnOnce(&mut Self::Context) -> String,
                ) -> (String, Self::Output) {
                    let mut ctx = DeferedCtx {
                        ctx1: self.stored,
                        output: Default::default(),
                    };
                    let sql_statment = statement_builder(&mut ctx);
                    return (sql_statment, ctx.output);
                }
            }

            impl<S, T> EncodeExtention<'static, T> for defered_binder<S>
            where
                S: Database + PositionalPlaceholder,
                T: Stored<S>,
            {
                fn encode(
                    &mut self,
                    val: T,
                ) -> impl FnOnce(&mut Self::Context) -> String + 'static + use<T, S>
                {
                    self.stored.push(Some(Box::new(val)));
                    let len = self.stored.len();
                    move |ctx1| {
                        let bring_back = ctx1
                            .ctx1
                            .get_mut(len - 1)
                            .map(|e| e.take())
                            .expect("item should be found")
                            .expect(" and taken only once");
                        bring_back.bind_stored(&mut ctx1.output);
                        S::placeholder().to_string()
                    }
                }
            }

            impl<S, E> ExpressionToFragment<'static, E> for defered_binder<S>
            where
                E: Expression<'static, Self>,
                S: Database,
            {
                fn expression_to_fragment(&mut self, t: E) -> <Self as QueryBuilder>::Fragment {
                    let fnitem = t.expression(self);
                    DeferedFragment(Box::new(fnitem))
                }
            }

            impl<S> SanitzingMechanisim for defered_binder<S>
            where
                S: SanitzingMechanisim,
            {
                type SanitzingMechanisim = S::SanitzingMechanisim;
            }

            #[cfg(test)]
            mod tests {
                use sqlx::{
                    Any as SqlxAny, Database,
                    any::{AnyArguments, AnyTypeInfo},
                };
                use std::{marker::PhantomData, sync::Mutex};

                use sqlx::{
                    Encode, Sqlite, Type,
                    encode::IsNull,
                    sqlite::{SqliteArgumentValue, SqliteArguments, SqliteTypeInfo, SqliteValue},
                };

                use crate::{
                    Buildable,
                    EncodeExtention,
                    Expression,
                    PositionalPlaceholder,
                    QueryBuilder,
                    SanitzingMechanisim,
                    defered_builder::defered_binder,
                    sanitize::{SanitizeAndHardcode, by_double_quote},
                    statements::select_st::SelectSt, // prelude::{col, stmt::SelectSt},
                };

                struct StrButCountOrder(&'static str);

                static BIND_ORDER: Mutex<Vec<String>> = Mutex::new(Vec::new());

                impl<'q> Encode<'q, SqlxAny> for StrButCountOrder {
                    fn encode_by_ref(
                        &self,
                        buf: &mut <SqlxAny as Database>::ArgumentBuffer<'q>,
                    ) -> Result<IsNull, sqlx::error::BoxDynError> {
                        panic!("should be called")
                    }
                    fn encode(
                        self,
                        buf: &mut <SqlxAny as Database>::ArgumentBuffer<'q>,
                    ) -> Result<IsNull, sqlx::error::BoxDynError>
                    where
                        Self: Sized,
                    {
                        let mut s = BIND_ORDER.lock().unwrap();
                        s.push(self.0.to_owned());
                        drop(s);
                        <String as Encode<'q, SqlxAny>>::encode(self.0.to_owned(), buf)
                    }
                }

                impl<'q, Q> Expression<'q, Q> for StrButCountOrder
                where
                    Q: QueryBuilder,
                    Q::SqlxDb: Database,
                    Q: EncodeExtention<'q, &'static str>,
                {
                    fn expression(
                        self,
                        query_builder: &mut Q,
                    ) -> impl FnOnce(&mut <Q>::Context) -> String + 'q + use<'q, Q>
                    where
                        Q: QueryBuilder,
                    {
                        let s = EncodeExtention::encode(query_builder, self.0);
                        move |ctx| {
                            let q = s(ctx);
                            q
                        }
                    }
                }

                impl Type<SqlxAny> for StrButCountOrder {
                    fn type_info() -> AnyTypeInfo {
                        todo!()
                    }
                }

                impl SanitzingMechanisim for SqlxAny {
                    type SanitzingMechanisim = by_double_quote;
                }

                use crate::expressions::*;
                impl PositionalPlaceholder for SqlxAny {
                    fn placeholder() -> &'static str {
                        "?"
                    }
                }

                #[test]
                fn positional_query_figure_out_order() {
                    let mut st = SelectSt::init(
                        "Todo",
                        defered_binder {
                            stored: Default::default(),
                            db: PhantomData::<SqlxAny>,
                        },
                    );

                    st.select(col("some_col"));
                    st.where_(col("id").to_eq(StrButCountOrder("where")));
                    st.offset(StrButCountOrder("offset"));

                    let (str, arg) = st.build();

                    drop(arg);

                    assert_eq!(
                        str,
                        "SELECT 'some_col' FROM 'Todo' WHERE 'id' = ? OFFSET ?;"
                    );

                    let bind_order = BIND_ORDER.lock().unwrap().clone();

                    assert_eq!(bind_order, vec!["where".to_string(), "offset".to_string()]);

                    // even when we call offset before where,
                    // PositionalQuery should know to reorder them
                    BIND_ORDER.lock().unwrap().drain(..);
                    let mut st = SelectSt::init(
                        "Todo",
                        defered_binder {
                            stored: Default::default(),
                            db: PhantomData::<SqlxAny>,
                        },
                    );

                    st.select(col("some_col"));
                    st.offset(StrButCountOrder("offset"));
                    st.where_(col("id").to_eq(StrButCountOrder("where")));

                    let (str, arg) = st.build();

                    drop(arg);

                    assert_eq!(
                        str,
                        "SELECT 'some_col' FROM 'Todo' WHERE 'id' = ? OFFSET ?;"
                    );

                    let bind_order = BIND_ORDER.lock().unwrap().clone();

                    assert_eq!(bind_order, vec!["where".to_string(), "offset".to_string()]);
                }
            }
        }
    }
}
