#![allow(non_camel_case_types)]
#![allow(unexpected_cfgs)]

use sqlx::{Encode, Type};

use crate::database_extention::DatabaseExt;
pub mod basic_expressions;
pub mod statements;
pub mod std_impls;
pub mod trait_objects;

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

impl<'q, S: DatabaseExt> StatementBuilder<'q, S> {
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

pub trait IsOpExpression {
    fn is_op(&self) -> bool;
}

impl<T> IsOpExpression for T
where
    T: OpExpression,
{
    fn is_op(&self) -> bool {
        true
    }
}

impl IsOpExpression for () {
    fn is_op(&self) -> bool {
        false
    }
}

impl<T: OpExpression> IsOpExpression for Option<T> {
    fn is_op(&self) -> bool {
        self.is_some()
    }
}

pub trait PossibleExpression<'q, S>: IsOpExpression + 'q {
    fn expression_starting(self, start: &'static str, ctx: &mut StatementBuilder<'q, S>)
    where
        S: DatabaseExt;
    fn expression(self, ctx: &mut StatementBuilder<'q, S>)
    where
        S: DatabaseExt;
}

impl<'q, S, T> PossibleExpression<'q, S> for T
where
    T: Expression<'q, S> + 'q,
{
    fn expression_starting(self, start: &'static str, ctx: &mut StatementBuilder<'q, S>)
    where
        S: DatabaseExt,
    {
        ctx.syntax(start);
        Expression::expression(self, ctx);
    }
    fn expression(self, ctx: &mut StatementBuilder<'q, S>)
    where
        S: DatabaseExt,
    {
        Expression::expression(self, ctx);
    }
}

impl<'q, S> PossibleExpression<'q, S> for () {
    fn expression_starting(self, _: &'static str, _: &mut StatementBuilder<'q, S>)
    where
        S: DatabaseExt,
    {
    }
    fn expression(self, _: &mut StatementBuilder<'q, S>)
    where
        S: DatabaseExt,
    {
    }
}

impl<'q, S, T> PossibleExpression<'q, S> for Option<T>
where
    T: Expression<'q, S> + 'q,
{
    fn expression_starting(self, start: &'static str, ctx: &mut StatementBuilder<'q, S>)
    where
        S: DatabaseExt,
    {
        if let Some(inner) = self {
            ctx.syntax(start);
            Expression::expression(inner, ctx);
        }
    }
    fn expression(self, ctx: &mut StatementBuilder<'q, S>)
    where
        S: DatabaseExt,
    {
        if let Some(inner) = self {
            Expression::expression(inner, ctx);
        }
    }
}

pub trait ManyExpressions<'q, S>: IsOpExpression + 'q {
    fn expression(self, start: &'static str, join: &'static str, ctx: &mut StatementBuilder<'q, S>)
    where
        S: DatabaseExt;
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

impl<'q, S> ManyExpressions<'q, S> for () {
    fn expression(self, _: &'static str, _: &'static str, _: &mut StatementBuilder<'q, S>)
    where
        S: DatabaseExt,
    {
    }
}

impl<'q, S, T> ManyExpressions<'q, S> for Option<T>
where
    T: Expression<'q, S> + 'q,
{
    fn expression(self, start: &'static str, join: &'static str, ctx: &mut StatementBuilder<'q, S>)
    where
        S: DatabaseExt,
    {
        if let Some(inner) = self {
            ctx.syntax(start);
            Expression::expression(inner, ctx);
        }
    }
}

impl<'q, S: DatabaseExt> StatementBuilder<'q, S> {
    pub fn new_many<Expr>(expr: Expr, start: &'static str, join: &'static str) -> Self
    where
        Expr: ManyExpressions<'q, S>,
    {
        let mut this = Self::default();

        expr.expression(start, join, &mut this);

        this
    }

    pub fn new_many_no_data<Expr>(
        expr: Expr,
        start: &'static str,
        join: &'static str,
    ) -> Option<String>
    where
        Expr: ManyExpressions<'q, S>,
    {
        let mut this = Self::default();

        expr.expression(start, join, &mut this);

        if this.count == 0 {
            Some(this.stmt)
        } else {
            None
        }
    }
}

pub use __sanitize_many::SanitizeMany;
pub use __sanitize_many::SanitizeManyTupleSpec;
mod __sanitize_many {
    use crate::{
        database_extention::DatabaseExt,
        sqlx_query_builder::{Expression, OpExpression, StatementBuilder},
        tuple_trait::{Tuple, TupleSpec},
    };

    pub struct SanitizeManyTupleSpec<'expression_mut_context, 'expression_parameter, S: DatabaseExt>(
        &'expression_mut_context mut StatementBuilder<'expression_parameter, S>,
    );

    impl<'c, 'a, 'q, S> TupleSpec<usize> for SanitizeManyTupleSpec<'a, 'q, S>
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

    impl<'c, 'a, 'q, S> TupleSpec<&'c str> for SanitizeManyTupleSpec<'a, 'q, S>
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

    impl<'a, 'q, S> TupleSpec<String> for SanitizeManyTupleSpec<'a, 'q, S>
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

    impl<'a, 'q, S, AsR> TupleSpec<(AsR,)> for SanitizeManyTupleSpec<'a, 'q, S>
    where
        AsR: AsRef<str>,
        S: DatabaseExt,
    {
        type Output = ();

        fn on_each<const LAST_INDEX: usize, const INDEX: usize>(
            &mut self,
            member: (AsR,),
        ) -> Self::Output {
            S::sanitize(member.0.as_ref(), &mut self.0.stmt);
        }
    }

    impl<'q, S> StatementBuilder<'q, S>
    where
        S: DatabaseExt,
    {
        pub fn sanitize_many<'a, T>(&mut self, data: T)
        where
            T: for<'s> Tuple<SanitizeManyTupleSpec<'s, 'q, S>>,
        {
            S::sanitize_start(&mut self.stmt);
            T::on_all_only_mut(data, SanitizeManyTupleSpec(self));
            S::sanitize_end(&mut self.stmt);
        }
    }

    pub struct SanitizeMany<T>(pub T);

    impl<T> OpExpression for SanitizeMany<T> {}
    impl<'q, S, T> Expression<'q, S> for SanitizeMany<T>
    where
        T: 'q,
        T: for<'a> Tuple<SanitizeManyTupleSpec<'a, 'q, S>>,
        S: DatabaseExt,
    {
        fn expression(self, ctx: &mut StatementBuilder<'q, S>) {
            ctx.sanitize_many(self.0);
        }
    }

    #[cfg(test)]
    mod tests {
        use crate::{
            database_extention::DatabaseExt,
            sqlx_query_builder::{Expression, OpExpression, StatementBuilder},
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
                ctx.sanitize_many((self.0, " world", local.as_str()));
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
