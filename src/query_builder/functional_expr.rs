use super::StatementBuilder;
use super::{
    DatabaseExt, Expression, IsOpExpression, ManyExpressions, OpExpression, PossibleExpression,
};
use std::ops::Not;

pub trait BoxedExpression<S: DatabaseExt>: Send {
    fn expression<'q>(self: Box<Self>, ctx: &mut StatementBuilder<'q, S>);
}
impl<E, S> BoxedExpression<S> for E
where
    S: DatabaseExt,
    E: for<'e> Expression<'e, S> + Send,
{
    fn expression<'q>(self: Box<Self>, ctx: &mut StatementBuilder<'q, S>) {
        Expression::expression(*self, ctx);
    }
}

impl<S> OpExpression for Box<dyn BoxedExpression<S> + Send> where S: DatabaseExt {}

impl<'q, S> Expression<'q, S> for Box<dyn BoxedExpression<S> + Send>
where
    S: DatabaseExt,
{
    fn expression(self, ctx: &mut StatementBuilder<'q, S>) {
        BoxedExpression::expression(self, ctx);
    }
}

#[cfg(test)]
mod create_boxed_expressions_from_tuple {
    use std::marker::PhantomData;

    use crate::{
        database_extention::DatabaseExt,
        query_builder::{
            Expression, OpExpression, StatementBuilder, functional_expr::BoxedExpression,
        },
        tuple_trait::{Tuple, TupleSpec},
    };
    use sqlx::Sqlite;

    pub struct ToImplExpr;
    impl OpExpression for ToImplExpr {}
    impl<'q> Expression<'q, Sqlite> for ToImplExpr {
        fn expression(self, _: &mut StatementBuilder<'q, Sqlite>) {
            panic!("irrelavant in tests")
        }
    }

    pub trait AsVec<T> {
        fn as_vec(self) -> Vec<T>;
    }

    impl<T> AsVec<T> for (T, T, T) {
        fn as_vec(self) -> Vec<T> {
            [self.0, self.1, self.2].into_iter().collect()
        }
    }

    struct ManyExpressionsCast<S>(PhantomData<S>);
    impl<S> Default for ManyExpressionsCast<S> {
        fn default() -> Self {
            Self(PhantomData)
        }
    }

    impl<T, S> TupleSpec<T> for ManyExpressionsCast<S>
    where
        S: DatabaseExt,
        T: for<'q> Expression<'q, S> + Send,
    {
        type Output = Box<dyn BoxedExpression<S> + Send>;

        fn on_each<const LEN: usize, const INDEX: usize>(&mut self, this: T) -> Self::Output {
            Box::new(this) as Box<dyn BoxedExpression<S> + Send>
        }
    }

    #[test]
    fn test_create_boxed_expressions_from_tuple() {
        let many_expr = (ToImplExpr, ToImplExpr, ToImplExpr);

        let many_expr: Vec<Box<dyn BoxedExpression<Sqlite> + Send>> = many_expr
            .on_all_only_mut(ManyExpressionsCast::default())
            .as_vec();

        assert_eq!(many_expr.len(), 3);
    }
}

impl<T> IsOpExpression for T
where
    T: OpExpression,
{
    fn is_op(&self) -> bool {
        true
    }
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
impl<T> IsOpExpression for Option<T> {
    fn is_op(&self) -> bool {
        self.is_some()
    }
}

impl<'q, T: 'q, S> PossibleExpression<'q, S> for Option<T>
where
    T: Expression<'q, S> + 'q,
{
    fn expression_starting(self, start: &'static str, ctx: &mut StatementBuilder<'q, S>)
    where
        S: DatabaseExt,
    {
        if let Some(this) = self {
            ctx.syntax(start);
            this.expression(ctx);
        }
    }
    fn expression(self, ctx: &mut StatementBuilder<'q, S>)
    where
        S: DatabaseExt,
    {
        if let Some(this) = self {
            this.expression(ctx);
        }
    }
}

impl IsOpExpression for () {
    fn is_op(&self) -> bool {
        false
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

pub struct ManyPossible<T>(pub T);

mod impl_many_expr {
    use super::ManyPossible;
    use crate::database_extention::DatabaseExt;
    use crate::query_builder::StatementBuilder;
    use crate::query_builder::{IsOpExpression, ManyExpressions, PossibleExpression};
    use paste::paste;

    macro_rules! implt {
        (
            $([$first:ident, $first_part:literal])?;
            $([$last:ident,  $last_part:literal ])?;
            $([$each:ident,  $part:literal      ])*
        ) => {

            impl<$($first,)? $($each,)* $($last,)?> IsOpExpression for ManyPossible<($($first,)? $($each,)* $($last,)?)>
            where
                $($first: IsOpExpression,)?
                $($each:  IsOpExpression,)*
                $($last:  IsOpExpression,)?
            {
                fn is_op(&self) -> bool {
                    // if some are op then the entire tuple is op
                    false
                    $(|| paste!(self.0.$first_part).is_op())?
                    $(|| paste!(self.0.$part).is_op())?
                    $(|| paste!(self.0.$last_part).is_op())?
                }
            }

            #[allow(unused)]
            impl<'q, S, $($first,)? $($each,)* $($last,)?> ManyExpressions<'q, S> for ManyPossible<($($first,)? $($each,)* $($last,)?)>
            where
                $($first: PossibleExpression<'q, S> + 'q,)?
                $($each:  PossibleExpression<'q, S> + 'q,)*
                $($last:  PossibleExpression<'q, S> + 'q,)?
            {
                fn expression(self,start: &'static str, join: &'static str, ctx: &mut StatementBuilder<'q, S>)
                    where S: DatabaseExt,
                {
                    let mut need_to_start = true;
                    $(
                        if paste!(self.0.$first_part).is_op() {
                            if need_to_start {
                                ctx.syntax(start);
                                need_to_start = false
                            }
                            paste!(self.0.$first_part).expression(ctx);
                        }
                    )?
                    $(
                        if paste!(self.0.$part).is_op() {
                            if need_to_start {
                                ctx.syntax(start);
                                need_to_start = false
                            } else {
                                ctx.syntax(join)
                            }
                            paste!(self.0.$part).expression(ctx);
                        }
                    )*
                    $(
                        if paste!(self.0.$last_part).is_op() {
                            if need_to_start {
                                ctx.syntax(start);
                            } else {
                                ctx.syntax(join)
                            }
                            paste!(self.0.$last_part).expression(ctx);
                        }
                    )?
                }
            }
        };
    }

    implt!(;;);
    implt!([R0, 0];;);
    implt!([R0, 0]; [R1, 1];);
    implt!([R0, 0]; [R2, 2]; [R1, 1]);
    implt!([R0, 0]; [R3, 3]; [R1, 1] [R2, 2]);
    implt!([R0, 0]; [R4, 4]; [R1, 1] [R2, 2] [R3, 3]);
}

impl<T> IsOpExpression for Vec<T>
where
    T: IsOpExpression,
{
    fn is_op(&self) -> bool {
        self.is_empty().not() && self.iter().all(|e| e.is_op())
    }
}

impl<'q, T, S> ManyExpressions<'q, S> for Vec<T>
where
    T: PossibleExpression<'q, S> + 'q,
{
    fn expression(self, start: &'static str, join: &'static str, ctx: &mut StatementBuilder<'q, S>)
    where
        S: DatabaseExt,
    {
        let mut need_to_start = true;

        for each in self {
            if each.is_op().not() {
                continue;
            }

            if need_to_start {
                ctx.syntax(start);
                need_to_start = false
            } else {
                ctx.syntax(join);
            }
            each.expression(ctx);
        }
    }
    // fn expression(self, start: &'static str, join: &'static str, ctx: &mut QueryBuilder<'q, S>)
    // where
    //     S: DatabaseExt,
    // {

    // }
}

#[cfg(test)]
mod test {
    // use sqlx::Sqlite;
    // use crate::{PossibleExpression, expressions::col, functional_expr::ZeroOrMoreImplPossible};
    // #[test]
    // fn main() {
    //     let zomip = ZeroOrMoreImplPossible {
    //         start: "WHERE ",
    //         join: " JOIN ",
    //         expressions: vec![col("index".to_string()).eq(3)],
    //     };
    //     let mut ctx = crate::QueryBuilder::<'_, Sqlite>::default();
    //     // PossibleExpression::expression(zomip, &mut ctx);
    // }
}

pub struct ManyImplPossible<T> {
    pub start: &'static str,
    pub join: &'static str,
    pub expressions: T,
}

impl<T: IsOpExpression> IsOpExpression for ManyImplPossible<T> {
    fn is_op(&self) -> bool {
        self.expressions.is_op()
    }
}

impl<'q, S, T> PossibleExpression<'q, S> for ManyImplPossible<T>
where
    T: ManyExpressions<'q, S> + 'q,
{
    /// double start! maybe that can be intentional!
    fn expression_starting(self, start: &'static str, ctx: &mut StatementBuilder<'q, S>)
    where
        S: DatabaseExt,
    {
        if self.is_op() {
            ctx.syntax(start);
            self.expressions.expression(&self.start, &self.join, ctx);
        }
    }

    fn expression(self, ctx: &mut StatementBuilder<'q, S>)
    where
        S: DatabaseExt,
    {
        self.expressions.expression(&self.start, &self.join, ctx);
    }
}

pub struct PossibleExprImplExpr<T>(T);

#[derive(Debug, Clone)]
pub struct PossibleExpressionCannotImplExpression;

impl<T> PossibleExprImplExpr<T> {
    pub fn new(possible_expr: T) -> Result<Self, PossibleExpressionCannotImplExpression>
    where
        T: IsOpExpression,
    {
        if IsOpExpression::is_op(&possible_expr) {
            Ok(Self(possible_expr))
        } else {
            Err(PossibleExpressionCannotImplExpression)
        }
    }
}

impl<T> OpExpression for PossibleExprImplExpr<T> {}

impl<'q, S, T> Expression<'q, S> for PossibleExprImplExpr<T>
where
    T: PossibleExpression<'q, S> + 'q,
{
    fn expression(self, ctx: &mut StatementBuilder<'q, S>)
    where
        S: DatabaseExt,
    {
        PossibleExpression::expression(self.0, ctx);
    }
}

pub struct ManyImplExpression<T> {
    item: T,
    start: &'static str,
    join: &'static str,
}

impl<T: IsOpExpression> ManyImplExpression<T> {
    pub fn new(item: T, start: &'static str, join: &'static str) -> Result<Self, ()> {
        if item.is_op() {
            Ok(Self { item, start, join })
        } else {
            Err(())
        }
    }
}

impl<T> OpExpression for ManyImplExpression<T> {}

impl<'q, S, T> Expression<'q, S> for ManyImplExpression<T>
where
    T: ManyExpressions<'q, S> + 'q,
{
    #[track_caller]
    fn expression(self, ctx: &mut StatementBuilder<'q, S>)
    where
        S: DatabaseExt,
    {
        if self.item.is_op().not() {
            panic!(
                "bug: ManyImplExpression is not operational, ManyImplExpression should not be constructed"
            );
        }
        self.item.expression(&self.start, &self.join, ctx);
    }
}

pub struct ManyFlat<T>(pub T);

#[cfg(test)]
mod tests {
    use crate::query_builder::functional_expr::ManyFlat;
    use crate::query_builder::{Expression, ManyExpressions, OpExpression, StatementBuilder};
    use sqlx::Sqlite;

    struct ManyToImplExpression<T>(pub T);
    impl<T> OpExpression for ManyToImplExpression<T> {}
    impl<T> Expression<'static, Sqlite> for ManyToImplExpression<T>
    where
        T: ManyExpressions<'static, Sqlite>,
    {
        fn expression(self, ctx: &mut StatementBuilder<'static, Sqlite>) {
            self.0.expression(&"START ", &", ", ctx);
        }
    }

    #[test]
    fn test_many_flat() {
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

// pub trait ManyBoxedExpressions<S: DatabaseExt> {
//     fn boxed_is_op(self: &Self) -> bool;
//     fn boxed_expression(
//         self: Box<Self>,
//         start: &dyn SqlSyntax,
//         join: &dyn SqlSyntax,
//         ctx: &mut StatementBuilder<'static, S>,
//     );
// }

// impl<S, T> ManyBoxedExpressions<S> for T
// where
//     T: ManyExpressions<'static, S>,
//     T: IsOpExpression,
//     S: DatabaseExt,
// {
//     fn boxed_is_op(self: &Self) -> bool {
//         T::is_op(&*self)
//     }
//     fn boxed_expression(
//         self: Box<Self>,
//         start: &dyn SqlSyntax,
//         join: &dyn SqlSyntax,
//         ctx: &mut StatementBuilder<'static, S>,
//     ) {
//         self.expression(start, join, ctx);
//     }
// }

// impl<S: DatabaseExt> IsOpExpression for Box<dyn ManyBoxedExpressions<S> + Send> {
//     fn is_op(&self) -> bool {
//         // if you have `&*self` rust will stack-overflow, nice tripping rustc!
//         ManyBoxedExpressions::boxed_is_op(&**self)
//     }
// }
// impl<S: DatabaseExt> ManyExpressions<'static, S> for Box<dyn ManyBoxedExpressions<S> + Send> {
//     fn expression<Start: SqlSyntax + ?Sized, Join: SqlSyntax + ?Sized>(
//         self,
//         start: &Start,
//         join: &Join,
//         ctx: &mut StatementBuilder<'static, S>,
//     ) where
//         S: DatabaseExt,
//     {
//         let start = start.as_rc();
//         let join = join.as_rc();
//         self.boxed_expression(&*start, &*join, ctx);
//     }
// }
