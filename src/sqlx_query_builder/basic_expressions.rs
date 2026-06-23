use std::marker::PhantomData;

use sqlx::Database;

use crate::{
    database_extention::DatabaseExt,
    sqlx_query_builder::{
        Expression, IsOpExpression, ManyExpressions, OpExpression, PossibleExpression,
        SanitizeManyTupleSpec, StatementBuilder,
    },
    tuple_trait::Tuple,
    update_mod::Update,
};
use sqlx::{Encode, Type};

// impl<'q, S, T> ManyExpressions<'q, S> for Option<T>
// where
//     T: OpExpression,
//     T: 'q + ManyExpressions<'q, S>,
// {
//     fn expression(self, start: &'static str, join: &'static str, ctx: &mut StatementBuilder<'q, S>)
//     where
//         S: DatabaseExt,
//     {
//         if let Some(this) = self {
//             this.expression(start, join, ctx);
//         }
//     }
// }

#[derive(Clone)]
pub struct PossibleImplExpression<T>(T);

impl<T> PossibleImplExpression<T>
where
    T: IsOpExpression,
{
    pub fn new(expr: T) -> Option<Self> {
        if expr.is_op() { Some(Self(expr)) } else { None }
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

#[derive(Clone)]
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

#[derive(Clone)]
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

pub struct TypeAsSyntax<T>(pub PhantomData<T>);

impl<T> Clone for TypeAsSyntax<T> {
    fn clone(&self) -> Self {
        Self(PhantomData)
    }
}

impl<T> OpExpression for TypeAsSyntax<T> {}
impl<'q, S, T: 'q> Expression<'q, S> for TypeAsSyntax<T>
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

#[derive(Clone)]
pub struct AliasedScopedColumn<T, C, A> {
    pub table: T,
    pub column: C,
    pub alias: A,
}

impl<T, C, A> OpExpression for AliasedScopedColumn<T, C, A> {}
impl<'q, T, C, A, S> Expression<'q, S> for AliasedScopedColumn<T, C, A>
where
    S: DatabaseExt,
    C: 'q + for<'s> Tuple<SanitizeManyTupleSpec<'s, 'q, S>>,
    T: 'q + for<'s> Tuple<SanitizeManyTupleSpec<'s, 'q, S>>,
    A: 'q + for<'s> Tuple<SanitizeManyTupleSpec<'s, 'q, S>>,
{
    fn expression(self, arg: &mut StatementBuilder<'q, S>)
    where
        S: DatabaseExt,
    {
        arg.sanitize_many(self.table);
        arg.syntax(".");
        arg.sanitize_many(self.column);
        arg.syntax(" AS ");
        arg.sanitize_many(self.alias);
    }
}

#[derive(Clone)]
pub struct ScopedColumn<T, C> {
    pub table: T,
    pub col: C,
}

impl<T, C> OpExpression for ScopedColumn<T, C> {}
impl<'q, T, C, S> Expression<'q, S> for ScopedColumn<T, C>
where
    T: 'q + for<'s> Tuple<SanitizeManyTupleSpec<'s, 'q, S>>,
    C: 'q + for<'s> Tuple<SanitizeManyTupleSpec<'s, 'q, S>>,
{
    fn expression(self, ctx: &mut StatementBuilder<'q, S>)
    where
        S: DatabaseExt,
    {
        ctx.sanitize_many(self.table);
        ctx.syntax(".");
        ctx.sanitize_many(self.col);
    }
}

#[derive(Clone)]
pub struct UpdatingColumn<C, T> {
    pub col: C,
    pub set: T,
}

impl<C, T> OpExpression for UpdatingColumn<C, Option<T>> {}
impl<'a, C, T, S> Expression<'a, S> for UpdatingColumn<C, Option<T>>
where
    S: DatabaseExt,
    T: sqlx::Type<S> + sqlx::Encode<'a, S> + 'a,
    C: 'a + for<'s> Tuple<SanitizeManyTupleSpec<'s, 'a, S>>,
{
    fn expression(self, ctx: &mut StatementBuilder<'a, S>) {
        ctx.sanitize_many(self.col);
        ctx.syntax(&" = ");
        match self.set {
            Some(value) => {
                ctx.bind(value);
            }
            None => {
                ctx.syntax(&" NULL");
            }
        }
    }
}

impl<C, T> IsOpExpression for UpdatingColumn<C, Update<T>> {
    fn is_op(&self) -> bool {
        matches!(self.set, Update::Set(_))
    }
}

impl<'a, C, T, S> PossibleExpression<'a, S> for UpdatingColumn<C, Update<T>>
where
    S: DatabaseExt,
    T: sqlx::Type<S> + sqlx::Encode<'a, S> + 'a,
    C: for<'s> Tuple<SanitizeManyTupleSpec<'s, 'a, S>>,
    C: 'a,
{
    fn expression(self, ctx: &mut StatementBuilder<'a, S>)
    where
        S: DatabaseExt,
    {
        match self.set {
            Update::Set(value) => {
                ctx.sanitize_many(self.col);
                ctx.syntax(&" = ");
                ctx.bind(value);
            }
            Update::Keep => {
                // do nothing
            }
        }
    }

    fn expression_starting(self, start: &'static str, ctx: &mut StatementBuilder<'a, S>)
    where
        S: DatabaseExt,
    {
        match self.set {
            Update::Set(value) => {
                ctx.syntax(start);
                ctx.sanitize_many(self.col);
                ctx.syntax(&" = ");
                ctx.bind(value);
            }
            Update::Keep => {
                // do nothing
            }
        }
    }
}

macro_rules! column_compare {
    ($name:ident, $field_name:ident, $op:literal) => {
        #[derive(Clone)]
        pub struct $name<Col, Val> {
            pub col: Col,
            pub $field_name: Val,
        }

        impl<Col, Val> OpExpression for $name<Col, Val> {}

        impl<'q, S, Col, Val> Expression<'q, S> for $name<Col, Val>
        where
            S: DatabaseExt,
            Col: Expression<'q, S> + 'q,
            Val: 'q + Encode<'q, S> + Type<S>,
        {
            fn expression(self, arg: &mut StatementBuilder<'q, S>) {
                self.col.expression(arg);
                arg.syntax($op);
                arg.bind(self.$field_name);
            }
        }
    };
}

macro_rules! column_is {
    ($name:ident, $op:literal) => {
        #[derive(Clone)]
        pub struct $name<Col> {
            pub col: Col,
        }

        impl<Col> OpExpression for $name<Col> {}

        impl<'q, S, Col> Expression<'q, S> for $name<Col>
        where
            S: DatabaseExt,
            Col: 'q + for<'s> Tuple<SanitizeManyTupleSpec<'s, 'q, S>>,
        {
            fn expression(self, arg: &mut StatementBuilder<'q, S>) {
                arg.sanitize_many(self.col);
                arg.syntax($op);
            }
        }
    };
}

macro_rules! group_with {
    ($name:ident, $op:literal) => {
        #[derive(Clone)]
        pub struct $name<T>(pub T);

        impl<T> IsOpExpression for $name<T>
        where
            T: IsOpExpression,
        {
            fn is_op(&self) -> bool {
                self.0.is_op()
            }
        }

        impl<'q, S, T> PossibleExpression<'q, S> for $name<T>
        where
            S: DatabaseExt,
            T: ManyExpressions<'q, S> + 'q,
        {
            fn expression(self, arg: &mut StatementBuilder<'q, S>) {
                if self.is_op() {
                    arg.syntax("(");
                    self.0.expression("", $op, arg);
                    arg.syntax(")");
                }
                arg.syntax($op);
            }
            fn expression_starting(self, start: &'static str, ctx: &mut StatementBuilder<'q, S>) {
                if self.is_op() {
                    ctx.syntax(start);
                    ctx.syntax("(");
                    self.0.expression("", $op, ctx);
                    ctx.syntax(")");
                }
            }
        }
    };
}

column_compare!(ColumnEqual, eq, " = ");
column_compare!(ColumnNotEqual, ne, " != ");
column_compare!(ColumnGreaterThan, gt, " > ");
column_compare!(ColumnGreaterThanOrEqual, ge, " >= ");
column_compare!(ColumnLessThan, lt, " < ");
column_compare!(ColumnLessThanOrEqual, le, " <= ");
column_compare!(ColumnContains, like, " LIKE ");

column_is!(ColumnIsNotNull, " IS NOT NULL");
column_is!(ColumnIsNull, " IS NULL");

group_with!(ExpressionsWithAnd, " AND ");
group_with!(ExpressionsWithOr, " OR ");

#[derive(Clone)]
pub struct ColumnIn<Col, V> {
    pub col: Col,
    pub values: V,
}

impl<Col, V> IsOpExpression for ColumnIn<Col, V>
where
    V: IsOpExpression,
{
    fn is_op(&self) -> bool {
        self.values.is_op()
    }
}

impl<'q, S, Col, V> PossibleExpression<'q, S> for ColumnIn<Col, V>
where
    S: DatabaseExt,
    Col: Expression<'q, S> + 'q,
    V: ManyExpressions<'q, S> + 'q,
{
    fn expression(self, ctx: &mut StatementBuilder<'q, S>) {
        if self.is_op() {
            ctx.syntax("(");
            self.col.expression(ctx);
            self.values.expression("IN (", ", ", ctx);
            ctx.syntax(")");
        }
    }

    fn expression_starting(self, start: &'static str, ctx: &mut StatementBuilder<'q, S>)
    where
        S: DatabaseExt,
    {
        if self.is_op() {
            ctx.syntax(start);
            self.col.expression(ctx);
            self.values.expression("IN (", ", ", ctx);
            ctx.syntax(")");
        }
    }
}

#[derive(Clone)]
pub struct ManyColumnsLargerOrEqual<Ids, Values> {
    pub ids: Ids,
    pub values: Values,
}

impl<Ids, Values> OpExpression for ManyColumnsLargerOrEqual<Ids, Values> {}

impl<'q, S, Ids, Values> Expression<'q, S> for ManyColumnsLargerOrEqual<Ids, Values>
where
    S: DatabaseExt,
    Ids: ManyExpressions<'q, S>,
    Values: ManyExpressions<'q, S>,
{
    fn expression(self, arg: &mut StatementBuilder<'q, S>) {
        arg.syntax("(");
        self.ids.expression("", ",", arg);
        arg.syntax(")");
        arg.syntax(" >= ");
        arg.syntax("(");
        self.values.expression("", ",", arg);
        arg.syntax(")");
    }
}

// pub mod tuple_many_expressions {
//     use crate::{
//         database_extention::DatabaseExt,
//         sqlx_query_builder::{Expression, ManyExpressions, OpExpression, StatementBuilder},
//         tuple_trait::{Tuple, TupleSpec},
//     };

//     pub struct TupleManyExpressions<T>(pub T);

//     pub struct TupleManyExpressionsSpec<'a, 's, S: DatabaseExt>(
//         &'static str,
//         &'static str,
//         &'a mut StatementBuilder<'s, S>,
//     );

//     impl<'a, 's, T, S> TupleSpec<T> for TupleManyExpressionsSpec<'a, 's, S>
//     where
//         S: DatabaseExt,
//         T: Expression<'s, S> + 's,
//     {
//         type Output = ();

//         fn on_each<const LAST_INDEX: usize, const INDEX: usize>(
//             &mut self,
//             member: T,
//         ) -> Self::Output {
//             if INDEX == 0 {
//                 self.2.syntax(self.0);
//             } else {
//                 self.2.syntax(self.1);
//             }
//             member.expression(&mut self.2);
//         }
//     }

//     impl<T> OpExpression for TupleManyExpressions<T> {}

//     impl<'s, S, T> ManyExpressions<'s, S> for TupleManyExpressions<T>
//     where
//         T: for<'m> Tuple<TupleManyExpressionsSpec<'m, 's, S>>,
//     {
//         fn expression(
//             self,
//             start: &'static str,
//             join: &'static str,
//             ctx: &mut crate::sqlx_query_builder::StatementBuilder<'s, S>,
//         ) where
//             S: crate::database_extention::DatabaseExt,
//         {
//             self.0
//                 .on_all_only_mut(TupleManyExpressionsSpec(start, join, ctx));
//         }
//     }
// }

#[derive(Clone)]
pub struct ManyFlat<T>(pub T);

impl<T> IsOpExpression for ManyFlat<Vec<T>>
where
    T: IsOpExpression,
{
    fn is_op(&self) -> bool {
        !self.0.is_empty() && self.0.iter().all(|e| e.is_op())
    }
}

impl<'s, T, S> ManyExpressions<'s, S> for ManyFlat<Vec<T>>
where
    T: ManyExpressions<'s, S>,
{
    fn expression(self, start: &'static str, join: &'static str, ctx: &mut StatementBuilder<'s, S>)
    where
        S: DatabaseExt,
    {
        let mut need_start = true;
        for each in self.0 {
            match (each.is_op(), need_start) {
                (false, _) => {}
                (true, true) => {
                    each.expression(start, join, ctx);
                    need_start = false;
                }
                (true, false) => each.expression(join, join, ctx),
            }
        }
    }
}

impl<T0> IsOpExpression for ManyFlat<(T0,)>
where
    T0: IsOpExpression,
{
    fn is_op(&self) -> bool {
        self.0.0.is_op()
    }
}

impl<'s, T0, S> ManyExpressions<'s, S> for ManyFlat<(T0,)>
where
    T0: ManyExpressions<'s, S>,
{
    fn expression(self, start: &'static str, join: &'static str, ctx: &mut StatementBuilder<'s, S>)
    where
        S: DatabaseExt,
    {
        if self.0.0.is_op() {
            ctx.syntax(start);
        }
        self.0.0.expression("", join, ctx);
    }
}

impl<T0, T1> IsOpExpression for ManyFlat<(T0, T1)>
where
    T0: IsOpExpression,
    T1: IsOpExpression,
{
    fn is_op(&self) -> bool {
        self.0.0.is_op() || self.0.1.is_op()
    }
}

impl<'s, T0, T1, S> ManyExpressions<'s, S> for ManyFlat<(T0, T1)>
where
    T0: ManyExpressions<'s, S>,
    T1: ManyExpressions<'s, S>,
{
    fn expression(self, start: &'static str, join: &'static str, ctx: &mut StatementBuilder<'s, S>)
    where
        S: DatabaseExt,
    {
        let this = self.0;
        let mut need_start = true;

        if this.0.is_op() {
            this.0.expression(start, join, ctx);
            need_start = false;
        }
        match (this.1.is_op(), need_start) {
            (false, _) => {}
            (true, true) => this.1.expression(start, join, ctx),
            (true, false) => this.1.expression(join, join, ctx),
        };
    }
}

impl<T0, T1, T2> IsOpExpression for ManyFlat<(T0, T1, T2)>
where
    T0: IsOpExpression,
    T1: IsOpExpression,
    T2: IsOpExpression,
{
    fn is_op(&self) -> bool {
        self.0.0.is_op() || self.0.1.is_op() || self.0.2.is_op()
    }
}

impl<'s, T0, T1, T2, S> ManyExpressions<'s, S> for ManyFlat<(T0, T1, T2)>
where
    T0: ManyExpressions<'s, S>,
    T1: ManyExpressions<'s, S>,
    T2: ManyExpressions<'s, S>,
{
    fn expression(self, start: &'static str, join: &'static str, ctx: &mut StatementBuilder<'s, S>)
    where
        S: DatabaseExt,
    {
        let this = self.0;
        let mut need_start = true;

        if this.0.is_op() {
            this.0.expression(start, join, ctx);
            need_start = false;
        }

        // swith the two bool
        match (this.1.is_op(), need_start) {
            (false, _) => {}
            (true, true) => {
                this.1.expression(start, join, ctx);
                need_start = false;
            }
            (true, false) => this.1.expression(join, join, ctx),
        };

        match (this.2.is_op(), need_start) {
            (false, _) => {}
            (true, true) => {
                this.2.expression(start, join, ctx);
            }
            (true, false) => this.2.expression(join, join, ctx),
        };
    }
}

#[cfg(test)]
mod tests {
    use crate::sqlx_query_builder::basic_expressions::ManyFlat;
    use crate::sqlx_query_builder::{Expression, ManyExpressions, OpExpression, StatementBuilder};
    use sqlx::Sqlite;

    struct ManyToImplExpression<T>(pub T);
    impl<T> OpExpression for ManyToImplExpression<T> {}
    impl<'q, T> Expression<'q, Sqlite> for ManyToImplExpression<T>
    where
        T: ManyExpressions<'q, Sqlite>,
    {
        fn expression(self, ctx: &mut StatementBuilder<'q, Sqlite>) {
            self.0.expression(&"START ", &", ", ctx);
        }
    }

    #[test]
    fn test_many_flat() {
        let stmt = StatementBuilder::<Sqlite>::new_many(
            ManyFlat((
                vec!["id", "email"],
                vec!["name", "age"],
                vec!["job", "job_description"],
            )),
            "START ",
            ", ",
        );

        pretty_assertions::assert_eq!(
            stmt.stmt().replace("\"", "'"),
            "START 'id', 'email', 'name', 'age', 'job', 'job_description'"
        );
        // test 3
        let stmt = StatementBuilder::<'_, Sqlite>::new(ManyToImplExpression(ManyFlat((
            vec!["id", "email"],
            vec!["name", "age"],
            vec!["job", "job_description"],
        ))));
        pretty_assertions::assert_eq!(
            stmt.stmt().replace("\"", "'"),
            "START 'id', 'email', 'name', 'age', 'job', 'job_description'"
        );
        // test 2
        let stmt = StatementBuilder::<'_, Sqlite>::new(ManyToImplExpression(ManyFlat((
            vec!["id", "email"],
            vec!["name", "age"],
        ))));
        pretty_assertions::assert_eq!(
            stmt.stmt().replace("\"", "'"),
            "START 'id', 'email', 'name', 'age'"
        );

        // test 1
        let stmt = StatementBuilder::<'_, Sqlite>::new(ManyToImplExpression(ManyFlat((vec![
            "id", "email",
        ],))));
        pretty_assertions::assert_eq!(stmt.stmt().replace("\"", "'"), "START 'id', 'email'");

        // test vec
        let stmt = StatementBuilder::<'_, Sqlite>::new(ManyToImplExpression(ManyFlat(vec![
            vec!["id", "email"],
            vec!["name", "age"],
            vec!["job", "job_description"],
        ])));
        pretty_assertions::assert_eq!(
            stmt.stmt().replace("\"", "'"),
            "START 'id', 'email', 'name', 'age', 'job', 'job_description'"
        );
    }
}

#[claw_ql_macros::skip]
mod old_code {
    #![allow(non_camel_case_types)]
    use crate::collections::{CollectionBasic, Member};
    use crate::database_extention::DatabaseExt;
    use crate::prelude::sql::AliasAndExpr;
    use crate::sqlx_query_builder::{Expression, ManyExpressions, OpExpression, StatementBuilder};
    use sqlx::{Database, Encode, Type};
    use std::marker::PhantomData;
    use std::ops::Not;

    pub struct member_as_expression<T>(pub T);
    impl<T> OpExpression for member_as_expression<T> {}
    impl<'q, S, T> Expression<'q, S> for member_as_expression<T>
    where
        T: Member,
        T: 'q,
    {
        fn expression(self, ctx: &mut StatementBuilder<'q, S>)
        where
            S: DatabaseExt,
        {
            ctx.sanitize(self.0.name());
        }
    }

    impl<T> member_as_expression<T> {
        pub fn eq<V>(self, eq: V) -> ColumnEqual<Self, V> {
            ColumnEqual { col: self, eq }
        }
    }

    pub struct table_as_expression<T>(pub T);
    impl<T> OpExpression for table_as_expression<T> {}
    impl<'q, S, T> Expression<'q, S> for table_as_expression<T>
    where
        T: CollectionBasic,
        T: 'q,
    {
        fn expression(self, ctx: &mut StatementBuilder<'q, S>)
        where
            S: DatabaseExt,
        {
            ctx.sanitize(self.0.table_name());
        }
    }

    pub mod single_col_expressions {
        use std::marker::PhantomData;

        use crate::{
            database_extention::DatabaseExt,
            sqlx_query_builder::{
                Expression, IsOpExpression, OpExpression, PossibleExpression, StatementBuilder,
            },
            update_mod::Update,
        };

        pub struct AliasedCol<T, C, A> {
            pub table: T,
            pub col: C,
            pub alias: A,
        }

        impl<T, C, A> OpExpression for AliasedCol<T, C, A> {}
        impl<'q, T, C, A, S> Expression<'q, S> for AliasedCol<T, C, A>
        where
            T: Expression<'q, S>,
            C: Expression<'q, S>,
            A: Expression<'q, S>,
        {
            fn expression(self, ctx: &mut StatementBuilder<'q, S>)
            where
                S: DatabaseExt,
            {
                self.table.expression(ctx);
                ctx.syntax(&".");
                self.col.expression(ctx);
                ctx.syntax(&" AS ");
                self.alias.expression(ctx);
            }
        }

        pub struct UpdatingCol<C, T> {
            pub col: C,
            pub set: T,
        }

        impl<C, T> OpExpression for UpdatingCol<C, Option<T>> {}
        impl<'a, C, T, S> Expression<'a, S> for UpdatingCol<C, Option<T>>
        where
            S: DatabaseExt,
            T: sqlx::Type<S> + sqlx::Encode<'a, S> + 'a,
            C: Expression<'a, S>,
        {
            fn expression(self, ctx: &mut StatementBuilder<'a, S>) {
                self.col.expression(ctx);
                ctx.syntax(&" = ");
                match self.set {
                    Some(value) => {
                        ctx.bind(value);
                    }
                    None => {
                        ctx.syntax(&" NULL");
                    }
                }
            }
        }

        impl<C, T> IsOpExpression for UpdatingCol<C, Update<T>> {
            fn is_op(&self) -> bool {
                matches!(self.set, Update::Set(_))
            }
        }
        impl<'a, C, T, S> PossibleExpression<'a, S> for UpdatingCol<C, Update<T>>
        where
            S: DatabaseExt,
            T: sqlx::Type<S> + sqlx::Encode<'a, S> + 'a,
            C: Expression<'a, S>,
        {
            fn expression(self, ctx: &mut StatementBuilder<'a, S>)
            where
                S: DatabaseExt,
            {
                match self.set {
                    Update::Set(value) => {
                        self.col.expression(ctx);
                        ctx.syntax(&" = ");
                        ctx.bind(value);
                    }
                    Update::Keep => {
                        // do nothing
                    }
                }
            }
            fn expression_starting(self, start: &'static str, ctx: &mut StatementBuilder<'a, S>)
            where
                S: DatabaseExt,
            {
                match self.set {
                    Update::Set(value) => {
                        ctx.syntax(start);
                        self.col.expression(ctx);
                        ctx.syntax(&" = ");
                        ctx.bind(value);
                    }
                    Update::Keep => {
                        // do nothing
                    }
                }
            }
        }

        pub struct MigratingCol<C, T> {
            pub col: C,
            pub phantom: PhantomData<T>,
        }

        impl<C, T> OpExpression for MigratingCol<C, T> {}
        impl<'a, C, T, S> Expression<'a, S> for MigratingCol<C, T>
        where
            S: DatabaseExt,
            T: sqlx::Type<S> + sqlx::Encode<'a, S> + 'a,
            C: AsRef<str> + 'a,
        {
            fn expression(self, ctx: &mut StatementBuilder<'a, S>)
            where
                S: DatabaseExt,
            {
                ctx.sanitize(self.col.as_ref());
                ctx.syntax(&" ");
                ctx.type_as_syntax::<T>();
            }
        }
    }

    pub mod multi_col_expressions_stack_heavy {
        use crate::{
            database_extention::DatabaseExt,
            sqlx_query_builder::{IsOpExpression, ManyExpressions, StatementBuilder},
        };

        pub struct ScopedCols<'q> {
            pub table: &'q str,
            pub cols: &'q [&'q str],
        }

        impl IsOpExpression for ScopedCols<'_> {
            fn is_op(&self) -> bool {
                self.cols.len() != 0
            }
        }
        impl<'q, S> ManyExpressions<'q, S> for ScopedCols<'q>
        where
            S: DatabaseExt,
        {
            fn expression(
                self,
                start: &'static str,
                join: &'static str,
                ctx: &mut StatementBuilder<'q, S>,
            ) where
                S: DatabaseExt,
            {
                let len = self.cols.len();
                if len == 0 {
                    return;
                }
                ctx.syntax(start);
                for (i, col) in self.cols.iter().enumerate() {
                    ctx.sanitize(self.table);
                    ctx.syntax(&".");
                    ctx.sanitize(col);
                    if i < len - 1 {
                        ctx.syntax(join);
                    }
                }
            }
        }

        pub struct AliasedCols<'q> {
            pub table: &'q str,
            pub cols: &'q [&'q str],
            pub alias: &'q str,
        }

        impl IsOpExpression for AliasedCols<'_> {
            fn is_op(&self) -> bool {
                self.cols.len() != 0
            }
        }
        impl<'q, S> ManyExpressions<'q, S> for AliasedCols<'static>
        where
            S: DatabaseExt,
        {
            fn expression(
                self,
                start: &'static str,
                join: &'static str,
                ctx: &mut StatementBuilder<'q, S>,
            ) where
                S: DatabaseExt,
            {
                let len = self.cols.len();
                if len == 0 {
                    return;
                }
                ctx.syntax(start);
                // panic!("problem, stmt {:?}", self.cols);
                for (i, item) in self.cols.into_iter().enumerate() {
                    ctx.sanitize(self.table);
                    ctx.syntax(&".");
                    ctx.sanitize(item);
                    ctx.syntax(&" AS ");
                    ctx.sanitize_strings((self.alias, *item));
                    if i < len - 1 {
                        ctx.syntax(join);
                    }
                }
                println!("problem, stmt {:?}", ctx.stmt());
            }
        }

        pub struct NumAliasedCols<'q> {
            pub table: &'q str,
            pub cols: &'q [&'q str],
            pub num: usize,
            pub alias: &'q str,
        }

        impl IsOpExpression for NumAliasedCols<'_> {
            fn is_op(&self) -> bool {
                self.cols.len() != 0
            }
        }
        impl<'q, S> ManyExpressions<'q, S> for NumAliasedCols<'static>
        where
            S: DatabaseExt,
        {
            fn expression(
                self,
                start: &'static str,
                join: &'static str,
                ctx: &mut StatementBuilder<'q, S>,
            ) {
                let len = self.cols.len();
                if len == 0 {
                    return;
                }
                ctx.syntax(start);
                for (i, item) in self.cols.into_iter().enumerate() {
                    ctx.sanitize(self.table);
                    ctx.syntax(&".");
                    ctx.sanitize(item);
                    ctx.syntax(&" AS ");
                    ctx.sanitize_strings((self.alias, self.num, *item));
                    if i < len - 1 {
                        ctx.syntax(join);
                    }
                }
            }
        }
    }

    pub struct ColumnDefinition<Name, Type, Constraints> {
        pub name: Name,
        pub ty: PhantomData<Type>,
        pub constraints: Constraints,
    }

    impl<Name, Type, Constraints> OpExpression for ColumnDefinition<Name, Type, Constraints> {}

    impl<'q, S, Name, Type, Constraints> Expression<'q, S> for ColumnDefinition<Name, Type, Constraints>
    where
        S: Database,
        Name: Expression<'q, S>,
        Type: is_null::IsNull + sqlx::Type<S> + 'q,
        Constraints: ManyExpressions<'q, S> + 'q,
    {
        fn expression(self, ctx: &mut StatementBuilder<'q, S>)
        where
            S: DatabaseExt,
        {
            self.name.expression(ctx);
            ctx.syntax(" ");

            ctx.type_as_syntax::<Type>();
            if <Type as is_null::IsNull>::is_null().not() {
                ctx.syntax(" NOT NULL");
            }
            self.constraints.expression(" ", ", ", ctx);
        }
    }

    pub struct col_def_for_collection_member<T>(pub T);

    impl<T> OpExpression for col_def_for_collection_member<T> {}

    impl<'q, S, T> Expression<'q, S> for col_def_for_collection_member<T>
    where
        S: Database,
        T: Member + 'q,
        T::Data: sqlx::Type<S> + 'static,
        T::Data: is_null::IsNull,
    {
        fn expression(self, ctx: &mut StatementBuilder<'q, S>)
        where
            S: DatabaseExt,
        {
            ctx.sanitize(self.0.name());
            ctx.syntax(" ");
            ctx.type_as_syntax::<T::Data>();
            let s = <T::Data as is_null::IsNull>::is_null();
            if s.not() {
                ctx.syntax(&" NOT NULL");
            }
        }
    }

    pub struct col<T>(pub T);

    impl<Column> col<Column> {
        pub fn pre_alias<Alias>(self, alias: Alias) -> PreAlias<col<Column>, Alias> {
            PreAlias { on: self, alias }
        }
        pub fn eq<Eq>(self, eq: Eq) -> ColumnEqual<Self, Eq> {
            ColumnEqual { col: self, eq }
        }
    }

    impl<T> OpExpression for col<T> {}

    impl<'q, S, T> Expression<'q, S> for col<T>
    where
        T: 'q,
        T: AsRef<str>,
    {
        fn expression(self, ctx: &mut StatementBuilder<'q, S>)
        where
            S: DatabaseExt,
        {
            ctx.sanitize(self.0.as_ref());
        }
    }

    pub struct pre_alias<T>(pub T, pub &'static str);

    impl<T> OpExpression for pre_alias<T> {}

    impl<'q, S, T: 'q + Expression<'q, S>> Expression<'q, S> for pre_alias<T> {
        fn expression(self, ctx: &mut StatementBuilder<'q, S>)
        where
            S: DatabaseExt,
        {
            self.0.expression(ctx);
            ctx.syntax(&" AS ");
            todo!()
        }
    }

    pub struct left_join {
        pub ft: String,
        pub fc: String,
        pub lt: String,
        pub lc: String,
    }

    impl OpExpression for left_join {}

    impl<'a, S> Expression<'a, S> for left_join {
        fn expression(self, ctx: &mut StatementBuilder<'a, S>)
        where
            S: DatabaseExt,
        {
            ctx.syntax(&"LEFT JOIN ");
            ctx.sanitize(self.ft.as_str());
            ctx.syntax(&" ON ");
            ctx.sanitize(self.lt.as_str());
            ctx.syntax(&".");
            ctx.sanitize(self.lc.as_str());
            ctx.syntax(&" = ");
            ctx.sanitize(self.ft.as_str());
            ctx.syntax(&".");
            ctx.sanitize(self.fc.as_str());
        }
    }

    // redundant: use prealias
    // pub struct local_col<T>(pub T);
    // impl<'q, S, T> Expression<'q, S> for local_col<T>
    // where
    //     T: SqlSanitize<S>,
    // {
    //     fn expression(self, ctx: &mut QueryBuilder<'q, S>)
    //     where
    //         S: DatabaseExt,
    //     {
    //         tracing::warn!(
    //             "todo: have better implementation for local_col 1.create col(..).alias(..) 2. handle namming conflicts"
    //         );
    //         ctx.sanitize(self.0);
    //         ctx.syntax(".");
    //         ctx.sanitize("id");
    //         ctx.syntax(" AS ");
    //         ctx.sanitize("local_id");
    //     }
    // }

    #[derive(Clone, Debug)]
    #[cfg_attr(feature = "serde", derive(serde::Deserialize))]
    pub struct ColumnEqual<Col, Eq> {
        pub col: Col,
        pub eq: Eq,
    }

    impl<A, E> AliasAndExpr<A, E> for ColumnEqual<A, E> {
        fn aliase_and_expr(alias: A, expr: E) -> Self {
            ColumnEqual {
                col: alias,
                eq: expr,
            }
        }
    }

    impl<Col, Eq> OpExpression for ColumnEqual<Col, Eq> {}

    impl<'q, S, Col, Eq> Expression<'q, S> for ColumnEqual<Col, Eq>
    where
        S: DatabaseExt,
        Eq: 'q + Encode<'q, S> + Type<S>,
        Col: Expression<'q, S> + 'q,
    {
        fn expression(self, arg: &mut StatementBuilder<'q, S>) {
            self.col.expression(arg);
            arg.syntax(" = ");
            arg.bind(self.eq);
        }
    }

    pub struct table<T>(pub T);

    impl<T> OpExpression for table<T> {}

    impl<'q, S, Table> Expression<'q, S> for table<Table>
    where
        S: DatabaseExt,
        Table: AsRef<str> + 'q,
    {
        fn expression(self, arg: &mut StatementBuilder<'q, S>) {
            arg.sanitize(self.0.as_ref());
        }
    }

    impl<Table> table<Table> {
        pub fn col<Column>(self, column: Column) -> scoped_column<Table, Column> {
            scoped_column {
                table: self.0,
                column,
            }
        }
    }

    pub struct scoped_column<Table, Column> {
        pub table: Table,
        pub column: Column,
    }

    impl<Table, Column> scoped_column<Table, Column> {
        pub fn eq<V>(self, eq: V) -> ColumnEqual<Self, V> {
            ColumnEqual { col: self, eq }
        }
        pub fn pre_alias<Alias>(
            self,
            alias: Alias,
        ) -> PreAlias<scoped_column<Table, Column>, Alias> {
            PreAlias { on: self, alias }
        }
    }

    impl<Table, Column> OpExpression for scoped_column<Table, Column> {}

    impl<'q, S, Column, Table> Expression<'q, S> for scoped_column<Table, Column>
    where
        S: DatabaseExt,
        Table: AsRef<str> + 'q,
        Column: AsRef<str> + 'q,
    {
        fn expression(self, arg: &mut StatementBuilder<'q, S>) {
            arg.sanitize(self.table.as_ref());
            arg.syntax(&".");
            arg.sanitize(self.column.as_ref());
        }
    }

    // to deprecate: aliasing is only relevant to to links and should use scoped_column_with_num instead
    pub struct PreAlias<On, Alias> {
        pub on: On,
        pub alias: Alias,
    }

    impl<On, Alias> OpExpression for PreAlias<On, Alias> {}

    // todo: replace &str with generics
    impl<'q, S, T, C, A> Expression<'q, S> for PreAlias<scoped_column<T, C>, A>
    where
        Self: 'q,
        S: DatabaseExt,
        T: AsRef<str> + 'q,
        C: AsRef<str> + 'q,
        A: AsRef<str> + 'q,
    {
        fn expression(self, arg: &mut StatementBuilder<'q, S>) {
            let alias = format!(
                "{}{}{}",
                self.alias.as_ref(),
                self.on.table.as_ref(),
                self.on.column.as_ref(),
            );
            arg.sanitize(self.on.table.as_ref());
            arg.syntax(&".");
            arg.sanitize(self.on.column.as_ref());
            arg.syntax(&" AS ");
            arg.sanitize(alias.as_str());
        }
    }

    // impl<'q, S> Expression<'q, S> for PreAlias<scoped_column<&str, &str>, &str>
    // where
    //     Self: 'q,
    //     S: DatabaseExt,
    // {
    //     fn expression(self, arg: &mut QueryBuilder<'q, S>) {
    //         let alias = format!("{}{}", self.alias, self.on.column);
    //         arg.sanitize(self.on.table);
    //         arg.syntax(".");
    //         arg.sanitize(self.on.column);
    //         arg.syntax(" AS ");
    //         arg.sanitize(alias);
    //     }
    // }

    impl<'q, S> Expression<'q, S> for PreAlias<col<&str>, &str>
    where
        Self: 'q,
        S: DatabaseExt,
    {
        fn expression(self, arg: &mut StatementBuilder<'q, S>) {
            let alias = format!("{}{}", self.alias, self.on.0);
            arg.sanitize(self.on.0);
            arg.syntax(&" AS ");
            arg.sanitize(alias.as_str());
        }
    }

    pub struct id_constraint<Id, Constraint>(pub Id, pub Constraint);

    impl<Id, Constraint> OpExpression for id_constraint<Id, Constraint> {}

    mod imp_id_constraint_for_sqlite {
        use sqlx::Sqlite;

        use crate::{
            database_extention::DatabaseExt,
            sqlx_query_builder::{Expression, StatementBuilder},
        };

        impl<'q, Id, Constraint> Expression<'q, Sqlite> for super::id_constraint<Id, Constraint>
        where
            Id: AsRef<str> + 'q,
            Constraint: Expression<'q, Sqlite>,
        {
            fn expression(self, ctx: &mut StatementBuilder<'q, Sqlite>)
            where
                Sqlite: DatabaseExt,
            {
                ctx.sanitize(self.0.as_ref());
                ctx.syntax(" ");
                self.1.expression(ctx);
            }
        }
    }

    pub struct foriegn_key<Ons> {
        pub references_table: String,
        pub references_col: String,
        pub ons: Ons,
    }

    impl<Ons> OpExpression for foriegn_key<Ons> {}

    mod imp_foriegn_key_for_sqlite {
        use sqlx::Sqlite;

        use crate::{
            database_extention::DatabaseExt,
            sqlx_query_builder::{
                Expression, ManyExpressions, StatementBuilder,
                essential_syntax::{CLOSE_PARANTHESIS, OPEN_PARANTHESIS},
            },
        };

        impl<'q, Ons> Expression<'q, Sqlite> for super::foriegn_key<Ons>
        where
            Ons: ManyExpressions<'q, Sqlite> + 'q,
        {
            fn expression(self, ctx: &mut StatementBuilder<'q, Sqlite>)
            where
                Sqlite: DatabaseExt,
            {
                ctx.syntax(&" REFERENCES ");

                ctx.sanitize(self.references_table.as_str());
                ctx.syntax(OPEN_PARANTHESIS);
                ctx.sanitize(self.references_col.as_str());
                ctx.syntax(CLOSE_PARANTHESIS);
                self.ons.expression(" ", ", ", ctx);
            }
        }
    }

    pub struct on_delete_set_null;

    impl OpExpression for on_delete_set_null {}

    mod imp_on_delete_set_null_for_sqlite {
        use sqlx::Sqlite;

        use crate::{
            database_extention::DatabaseExt,
            sqlx_query_builder::{Expression, StatementBuilder},
        };

        impl<'q> Expression<'q, Sqlite> for super::on_delete_set_null {
            fn expression(self, ctx: &mut StatementBuilder<'q, Sqlite>)
            where
                Sqlite: DatabaseExt,
            {
                ctx.syntax(&"ON DELETE SET NULL");
            }
        }
    }
}
