#![allow(non_camel_case_types)]
#![allow(unexpected_cfgs)]

use std::marker::PhantomData;

use sqlx::{Database, Encode, Type};

use crate::database_extention::DatabaseExt;
pub mod functional_expr;
pub mod sanitize;

pub mod essential_syntax {
    // I have these, because bad LSP can be confused by paranthesis inside strings
    pub const OPEN_PARANTHESIS: &str = "(";
    pub const CLOSE_PARANTHESIS: &str = ")";
}

pub struct StatementBuilder<'q, S>
where
    S: DatabaseExt,
{
    pub(crate) stmt: String,
    count: usize,
    arg: S::Arguments<'q>,
}

impl<'q, S: DatabaseExt> StatementBuilder<'q, S> {
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

    pub fn new_no_data<Expr>(expr: Expr) -> Option<String>
    where
        Expr: Expression<'q, S>,
    {
        let mut this = Self::default();

        expr.expression(&mut this);

        if this.count == 0 {
            Some(this.stmt)
        } else {
            None
        }
    }
}

impl<'q, S: DatabaseExt> Default for StatementBuilder<'q, S> {
    fn default() -> Self {
        StatementBuilder {
            stmt: String::new(),
            count: 0,
            arg: S::Arguments::default(),
        }
    }
}

/// assert is not Expression is not empty
/// prevent downstream crate from `impl Expression<LocalType> for ForiegnTypes`
pub trait OpExpression {}

/// Representing an sql string/expression/statement that is not empty.
///
/// Possibly non-operational expressions implements `PossibleExpression`, multiple expressions implements `ManyExpressions`, both traits are extentions of this trait (will be automatically implemented for all types that implement `Expression`).
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
/// the only reason why there is lifetime in expression interface is: because in Sqlite you can send a string reference to an in-memory-database (instead of serializing the ref to an owned String and sending it over the netword like MySQL and PostgreQL), so you would have to wait for the lifetime (impl Expression<Sqlite, 'q> for &'q str) to be droped before you can mutate or move the referenced string
///
/// in fact all impelentation of `sqlx::Encode` (used internally by `QueryBuilder::bind`) are static expect for `impl<'s> Encode<'s, Sqlite> for &'s str`
///
/// this lifetime itroduce restriction on all references made between `holding_lifetime` and `lifetime_droped`
///
/// Don't feel the need to abstract over this lifetime, look for statics instead of introducing a new lifetime (i.e. where T: Expression<'static, S>), especially when you are building over-the-netword backends where everything is static anyway, lifetime here is to have a perfect API that I don't need to refactor later.
pub trait Expression<'q, S>: OpExpression + 'q {
    fn expression(self, ctx: &mut StatementBuilder<'q, S>)
    where
        S: DatabaseExt;
}

impl OpExpression for String {}
impl<'q, S> Expression<'q, S> for String
where
    S: DatabaseExt,
{
    fn expression(self, ctx: &mut StatementBuilder<'q, S>)
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
    fn expression(self, ctx: &mut StatementBuilder<'q, S>)
    where
        S: DatabaseExt,
    {
        ctx.sanitize(self);
    }
}

pub struct Bind<T>(pub T);
impl<T> OpExpression for Bind<T> {}
impl<'q, S, T> Expression<'q, S> for Bind<T>
where
    S: sqlx::Database,
    T: 'q + sqlx::Type<S> + sqlx::Encode<'q, S>,
{
    fn expression(self, ctx: &mut StatementBuilder<'q, S>)
    where
        S: DatabaseExt,
    {
        ctx.bind(self.0);
    }
}

pub struct SyntaxAsType<T>(pub PhantomData<T>);

impl<T> OpExpression for SyntaxAsType<T> {}
impl<'q, S, T: 'q> Expression<'q, S> for SyntaxAsType<T>
where
    S: DatabaseExt,
    T: sqlx::Type<S> + 'q,
{
    fn expression(self, ctx: &mut StatementBuilder<'q, S>)
    where
        S: Database,
    {
        ctx.type_as_syntax::<T>();
    }
}

pub trait IsOpExpression {
    fn is_op(&self) -> bool;
}

pub trait PossibleExpression<'q, S>: IsOpExpression + 'q {
    fn expression_starting(self, start: &'static str, ctx: &mut StatementBuilder<'q, S>)
    where
        S: DatabaseExt;
    fn expression(self, ctx: &mut StatementBuilder<'q, S>)
    where
        S: DatabaseExt;
}

pub struct PossibleImplExpression<T>(T);
impl<T> PossibleImplExpression<T>
where
    T: IsOpExpression,
{
    pub fn new(expr: T) -> Result<Self, ()> {
        if expr.is_op() {
            Ok(Self(expr))
        } else {
            Err(())
        }
    }
}

impl<T> OpExpression for PossibleImplExpression<T> {}
impl<'q, S, T> Expression<'q, S> for PossibleImplExpression<T>
where
    T: PossibleExpression<'q, S> + 'q,
{
    fn expression(self, ctx: &mut StatementBuilder<'q, S>)
    where
        S: DatabaseExt,
    {
        self.0.expression(ctx);
    }
}

pub trait ManyExpressions<'q, S>: IsOpExpression + 'q {
    fn expression(self, start: &'static str, join: &'static str, ctx: &mut StatementBuilder<'q, S>)
    where
        S: DatabaseExt;
}

impl<'q> IsOpExpression for &'q [&'q str] {
    fn is_op(&self) -> bool {
        self.len() != 0
    }
}
impl<'q, S> ManyExpressions<'q, S> for &'static [&'static str] {
    fn expression(self, start: &'static str, join: &'static str, ctx: &mut StatementBuilder<'q, S>)
    where
        S: DatabaseExt,
    {
        for (i, each) in self.into_iter().enumerate() {
            if i == 0 {
                ctx.syntax(start);
            } else {
                ctx.syntax(join);
            }
            ctx.sanitize(each);
        }
    }
}

pub trait ManyBoxedExpressions<S> {
    fn is_op(&self) -> bool;
    fn boxed_expression<'q>(
        self: Box<Self>,
        start: &'static str,
        join: &'static str,
        ctx: &mut StatementBuilder<'q, S>,
    ) where
        S: DatabaseExt;
}

impl<S, T> ManyBoxedExpressions<S> for T
where
    T: for<'q> ManyExpressions<'q, S> + Send,
{
    fn is_op(&self) -> bool {
        T::is_op(self)
    }
    fn boxed_expression<'q>(
        self: Box<Self>,
        start: &'static str,
        join: &'static str,
        ctx: &mut StatementBuilder<'q, S>,
    ) where
        S: DatabaseExt,
    {
        self.expression(start, join, ctx);
    }
}

impl<S> IsOpExpression for Box<dyn ManyBoxedExpressions<S> + Send> {
    fn is_op(&self) -> bool {
        ManyBoxedExpressions::is_op(&**self)
    }
}

impl<'q, S: 'q> ManyExpressions<'q, S> for Box<dyn ManyBoxedExpressions<S> + Send> {
    fn expression(self, start: &'static str, join: &'static str, ctx: &mut StatementBuilder<'q, S>)
    where
        S: DatabaseExt,
    {
        self.boxed_expression(start, join, ctx);
    }
}

impl<'q, S> ManyExpressions<'q, S> for () {
    fn expression(self, _: &'static str, _: &'static str, _: &mut StatementBuilder<'q, S>)
    where
        S: DatabaseExt,
    {
    }
}

impl<'q, S, T> ManyExpressions<'q, S> for Option<T>
where
    T: 'q + ManyExpressions<'q, S>,
{
    fn expression(self, start: &'static str, join: &'static str, ctx: &mut StatementBuilder<'q, S>)
    where
        S: DatabaseExt,
    {
        if let Some(this) = self {
            this.expression(start, join, ctx);
        }
    }
}

impl<'q, S, T> ManyExpressions<'q, S> for T
where
    T: Expression<'q, S> + 'q,
{
    fn expression(self, start: &'static str, _: &'static str, ctx: &mut StatementBuilder<'q, S>)
    where
        S: DatabaseExt,
    {
        ctx.syntax(start);

        Expression::expression(self, ctx);
    }
}

pub struct PossibleImplMany<T>(pub T);

impl<T> IsOpExpression for PossibleImplMany<T>
where
    T: IsOpExpression,
{
    fn is_op(&self) -> bool {
        self.0.is_op()
    }
}

impl<'q, S, T> ManyExpressions<'q, S> for PossibleImplMany<T>
where
    T: PossibleExpression<'q, S> + 'q,
{
    fn expression(self, start: &'static str, _: &'static str, ctx: &mut StatementBuilder<'q, S>)
    where
        S: DatabaseExt,
    {
        if self.0.is_op() {
            ctx.syntax(start);
            self.0.expression(ctx);
        }
    }
}

impl<'q, S> StatementBuilder<'q, S>
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
        S::sanitize_start(&mut self.stmt);
        S::sanitize(display, &mut self.stmt);
        S::sanitize_end(&mut self.stmt);
    }

    /// push str that is known to not cause sql injection,
    ///
    /// the type for the syntax is `&'static str`, because
    /// it is less likely to be created by a network request
    /// (unless you maliciously leak a string like `String::from(from_network).leak()`)
    /// that is fine because sql injection targets bad code and not malicious code
    pub fn syntax(&mut self, str: &'static str) {
        self.stmt.push_str(str);
    }

    pub fn type_as_syntax<T: Type<S>>(&mut self) {
        use sqlx::TypeInfo;
        self.stmt.push_str(T::type_info().name());
    }

    pub fn unwrap(self) -> (String, S::Arguments<'q>) {
        (self.stmt, self.arg)
    }
}

pub use sanitize_and_build_3::SanitizeMany;
mod sanitize_and_build_3 {
    use crate::{
        database_extention::DatabaseExt,
        query_builder::{Expression, OpExpression, StatementBuilder},
        tuple_trait::{Tuple, TupleSpec},
    };

    pub struct SanitizeAndBuild<'expression_mut_context, 'expression_parameter, S: DatabaseExt>(
        &'expression_mut_context mut StatementBuilder<'expression_parameter, S>,
    );

    impl<'c, 'a, 'q, S> TupleSpec<usize> for SanitizeAndBuild<'a, 'q, S>
    where
        S: DatabaseExt,
    {
        type Output = ();

        fn on_each<const LAST_INDEX: usize, const INDEX: usize>(
            &mut self,
            member: usize,
        ) -> Self::Output {
            S::sanitize(member.to_string().as_str(), &mut self.0.stmt);
        }
    }

    impl<'c, 'a, 'q, S> TupleSpec<&'c str> for SanitizeAndBuild<'a, 'q, S>
    where
        S: DatabaseExt,
    {
        type Output = ();

        fn on_each<const LAST_INDEX: usize, const INDEX: usize>(
            &mut self,
            member: &'c str,
        ) -> Self::Output {
            S::sanitize(member, &mut self.0.stmt);
        }
    }

    impl<'a, 'q, S> TupleSpec<String> for SanitizeAndBuild<'a, 'q, S>
    where
        S: DatabaseExt,
    {
        type Output = ();

        fn on_each<const LAST_INDEX: usize, const INDEX: usize>(
            &mut self,
            member: String,
        ) -> Self::Output {
            S::sanitize(member.as_str(), &mut self.0.stmt);
        }
    }

    impl<'q, S> StatementBuilder<'q, S>
    where
        S: DatabaseExt,
    {
        pub fn sanitize_strings<'a, T>(&mut self, data: T)
        where
            T: for<'s> Tuple<SanitizeAndBuild<'s, 'q, S>>,
        {
            S::sanitize_start(&mut self.stmt);
            T::on_all_only_mut(data, SanitizeAndBuild(self));
            S::sanitize_end(&mut self.stmt);
        }
    }

    pub struct SanitizeMany<T>(pub T);

    impl<T> OpExpression for SanitizeMany<T> {}
    impl<'q, S, T> Expression<'q, S> for SanitizeMany<T>
    where
        T: 'q,
        T: for<'a> Tuple<SanitizeAndBuild<'a, 'q, S>>,
        S: DatabaseExt,
    {
        fn expression(self, ctx: &mut StatementBuilder<'q, S>) {
            ctx.sanitize_strings(self.0);
        }
    }

    #[cfg(test)]
    mod tests {
        use crate::{
            database_extention::DatabaseExt,
            query_builder::{Expression, OpExpression, StatementBuilder},
        };
        use sqlx::Sqlite;

        pub struct TestSanitizeAndBuild<'q>(&'q str);

        impl OpExpression for TestSanitizeAndBuild<'_> {}
        impl<'q, 'a, S> Expression<'q, S> for TestSanitizeAndBuild<'a>
        where
            'a: 'q,
            S: DatabaseExt,
        {
            fn expression(self, ctx: &mut StatementBuilder<'q, S>) {
                let local = String::from(" local");
                ctx.sanitize_strings((self.0, " world", local.as_str()));
            }
        }

        #[test]
        fn main() {
            let borrow = String::from("hello");
            let str = StatementBuilder::<Sqlite>::new(TestSanitizeAndBuild(&borrow))
                .unwrap()
                .0;

            assert_eq!(str, "\"hello world local\"");
        }
    }
}

#[claw_ql_macros::skip]
mod sanitize_and_build_2 {

    use crate::{
        database_extention::DatabaseExt,
        query_builder::StatementBuilder,
        tuple_trait::{Tuple, TupleSpec},
    };

    pub struct SanitizeAndBuild<'expression_mut_context, 'expression_parameter, S: DatabaseExt>(
        &'expression_mut_context mut StatementBuilder<'expression_parameter, S>,
    );

    impl<'a, 'q, S> TupleSpec<&'q str> for SanitizeAndBuild<'a, 'q, S>
    where
        S: DatabaseExt,
    {
        type Output = ();

        fn on_each<const LAST_INDEX: usize, const INDEX: usize>(
            &mut self,
            member: &'q str,
        ) -> Self::Output {
            S::sanitize(member, &mut self.0.stmt);
        }
    }

    impl<'q, S> StatementBuilder<'q, S>
    where
        S: DatabaseExt,
    {
        pub fn sanitize_strings<T>(&mut self, data: T)
        where
            T: for<'s> Tuple<SanitizeAndBuild<'s, 'q, S>>,
        {
            S::sanitize_start(&mut self.stmt);
            T::on_all_only_mut(data, SanitizeAndBuild(self));
            S::sanitize_end(&mut self.stmt);
        }
    }

    #[cfg(test)]
    mod tests {
        use crate::{
            database_extention::DatabaseExt,
            query_builder::{Expression, OpExpression, StatementBuilder},
        };
        use sqlx::Sqlite;

        pub struct TestSanitizeAndBuild<'q>(&'q str);

        impl OpExpression for TestSanitizeAndBuild<'_> {}
        impl<'q, S> Expression<'q, S> for TestSanitizeAndBuild<'q>
        where
            S: DatabaseExt,
        {
            fn expression(self, ctx: &mut StatementBuilder<'q, S>) {
                ctx.sanitize_strings((self.0, "world"));
            }
        }

        #[test]
        fn main() {
            let borrow = String::from("hello");
            let str = StatementBuilder::<Sqlite>::new(TestSanitizeAndBuild(&borrow))
                .unwrap()
                .0;

            assert_eq!(str, "\"helloworld\"");
        }
    }
}

#[claw_ql_macros::skip]
mod sanitize_and_build {
    use std::marker::PhantomData;

    use crate::{
        database_extention::DatabaseExt,
        query_builder::StatementBuilder,
        tuple_trait::{TupleLast, TupleLastSpec},
    };

    pub struct SanitizeAndBuild<'q, S>(&'q mut String, PhantomData<S>);

    impl<'q, S> TupleLastSpec<&'static str, &'static str> for SanitizeAndBuild<'q, S>
    where
        S: DatabaseExt,
    {
        type Output = ();
        type LastOutput = ();

        fn on_each<const LAST_INDEX: usize, const INDEX: usize>(
            &mut self,
            member: &'static str,
        ) -> Self::Output {
            S::sanitize(member, &mut self.0);
        }

        fn on_last<const INDEX: usize>(mut self, member: &'static str) -> Self::LastOutput {
            S::sanitize(member, &mut self.0);
            S::sanitize_end(&mut self.0);
        }
    }

    impl<'q, S> StatementBuilder<'q, S>
    where
        S: DatabaseExt,
    {
        pub fn sanitize_strings<T>(&mut self, data: T)
        where
            T: for<'s> TupleLast<SanitizeAndBuild<'s, S>>,
        {
            S::sanitize_start(&mut self.stmt);
            T::on_all(data, SanitizeAndBuild(&mut self.stmt, PhantomData));
        }
    }

    #[cfg(test)]
    mod tests {
        use crate::{
            database_extention::DatabaseExt,
            query_builder::{Expression, OpExpression, StatementBuilder},
        };
        use sqlx::Sqlite;

        pub struct TestSanitizeAndBuild;

        impl OpExpression for TestSanitizeAndBuild {}
        impl<'q, S> Expression<'q, S> for TestSanitizeAndBuild
        where
            S: DatabaseExt,
        {
            fn expression(self, ctx: &mut StatementBuilder<'q, S>) {
                ctx.sanitize_strings(("hello", "world"));
            }
        }

        #[test]
        fn main() {
            let str = StatementBuilder::<Sqlite>::new(TestSanitizeAndBuild)
                .unwrap()
                .0;

            assert_eq!(str, "\"helloworld\"");
        }
    }
}

#[cfg(feature = "skip_without_comments")]
mod mut_query_builder {
    use std::hash::BuildHasher;

    use crate::{
        OpExpression,
        expressions::{ColumnEqual, col},
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

        pub trait ExpressionToFragment<'q, T>: StatementBuilder {
            fn expression_to_fragment(
                &mut self,
                expression: T,
            ) -> <Self as StatementBuilder>::Fragment;
        }

        // trait to extend sqlx's Encode trait -- adapted to fit the need of this library
        pub trait EncodeExtention<'q, T>: StatementBuilder {
            fn encode(
                &mut self,
                val: T,
            ) -> impl FnOnce(&mut Self::Context) -> String + 'q + use<'q, T, Self>;
        }

        pub trait Buildable: Sized {
            type QueryBuilder: StatementBuilder;
            fn build(self) -> (String, <Self::QueryBuilder as StatementBuilder>::Output);
        }

        pub trait Expression<'q, Q> {
            fn expression(
                self,
                query_builder: &mut Q,
            ) -> impl FnOnce(&mut Q::Context) -> String + 'q + use<'q, Q, Self>
            where
                Q: StatementBuilder;
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
                Q: StatementBuilder + SanitzingMechanisim,
                T: SanitizeAndHardcode<Q::SanitzingMechanisim> + 'q,
            {
                fn expression(
                    self,
                    ctx: &mut Q,
                ) -> impl FnOnce(&mut <Q>::Context) -> String + 'q + use<'q, Q, T>
                where
                    Q: StatementBuilder,
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

            impl<'q, S: Database> StatementBuilder for direct_bind<'q, S> {
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
                fn expression_to_fragment(&mut self, t: E) -> <Self as StatementBuilder>::Fragment {
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

            impl<S: Database> StatementBuilder for defered_binder<S> {
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
                fn expression_to_fragment(&mut self, t: E) -> <Self as StatementBuilder>::Fragment {
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
                    Q: StatementBuilder,
                    Q::SqlxDb: Database,
                    Q: EncodeExtention<'q, &'static str>,
                {
                    fn expression(
                        self,
                        query_builder: &mut Q,
                    ) -> impl FnOnce(&mut <Q>::Context) -> String + 'q + use<'q, Q>
                    where
                        Q: StatementBuilder,
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
