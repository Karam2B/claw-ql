use serde::Serialize;
use sqlx::Database;

// pub mod delete_by_id;
// pub mod delete_one;
pub mod fetch_many;
// pub mod fetch_one;
pub mod insert_one;
// pub mod update_one;

pub trait OperationOutput {
    type Output;
}
pub trait Operation<S>: OperationOutput<Output: Send> + Send {
    fn exec_operation(self, pool: &mut S::Connection) -> impl Future<Output = Self::Output> + Send
    where
        S: Database,
        Self: Sized;
}

/// to be removed in favor of macros
///
/// User-facing check, validation, and transormation.
/// Needed to execute Operation without bugs.
///
/// panics or compile error for `Operation` trait is considered a bug.
/// you might forgot to call `safety_check` before calling `exec_operation` or
/// there is a bug in this crate's code
///
/// most of the time these checks can be known inside const context,
/// since `const_trait_impl` is not stable yet, you have to run these at runtime.
/// for now you can use the macro `sql` which mimics this trait using const_blocks.
///
/// example of that is when you try to  check wither a 'where clause'
/// specifies any unique filters, if you have tuple of (T0,T1), there is no
/// way to check if EITHER T0 or T1 is a unique filter, that would be
/// equivalent of this hypothetical rust
///
/// code:
/// ```no_run
///     impl<T0,T1> SafeOperation
///     for SelectOneAndOnlyOne<Wheres = (T0,T1)>
///     where   
///         (T0: AssertUniqueFilter) or (T1: AssertUniqueFilter),
///     {}
/// ```
///
/// or just simply using `const_trait_impl`
///
/// ```no_run
///     impl<T0, T1> const SafeOperation for SelectOneAndOnlyOne<Wheres = (T0,T1)>
///     where
///         T0: [const] UniqueFilter + [const] Destruct,
///         T1: [const] UniqueFilter + [const] Destruct,
///     {
///         fn safety_check(self) -> Result<Self::Ok, Self::Error> {
///             if self.wheres.0.is_unique() || self.wheres.1.is_unique() {
///                 return Ok(self.0);
///             }
///             Err(Self::Error::NonUniqueOperation)
///         }
///     }
/// ```
///
/// if the the checks are "inevitably non-const", consider if the implementation of `Operation`
/// can have an output of `Option<T>` or `Result<T, _>`, if so, no need to
/// implement `SafeOperation`
///
/// in this crate `NeedCheck` is used to force you to use `SafeOperation`
/// before using `Operation` impls by making `Ok = NeedCheck<T>`, and
/// implementing Operation for `NeedCheck<T>`
pub trait SafeOperation {
    type Error;
    type Ok;
    fn safety_check(self) -> Result<Self::Ok, Self::Error>;
}

#[derive(Clone, Debug, PartialEq, Serialize)]
pub struct LinkedOutput<Id, C, L> {
    pub id: Id,
    pub attributes: C,
    pub links: L,
}

#[derive(Clone, Debug, PartialEq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct CollectionOutput<Id, C> {
    pub id: Id,
    pub attributes: C,
}

#[derive(Clone, Debug, PartialEq, Serialize)]
pub struct IdOutput<Id> {
    pub id: Id,
}

impl<I, C, L> From<LinkedOutput<I, C, L>> for CollectionOutput<I, C> {
    fn from(value: LinkedOutput<I, C, L>) -> Self {
        CollectionOutput {
            id: value.id,
            attributes: value.attributes,
        }
    }
}

impl<I, C, L> From<LinkedOutput<I, C, L>> for IdOutput<I> {
    fn from(value: LinkedOutput<I, C, L>) -> Self {
        IdOutput { id: value.id }
    }
}

impl<I, C> From<CollectionOutput<I, C>> for IdOutput<I> {
    fn from(value: CollectionOutput<I, C>) -> Self {
        IdOutput { id: value.id }
    }
}

mod functional_impls {
    use crate::operations::{Operation, OperationOutput};
    use sqlx::Database;

    impl<T> OperationOutput for Vec<T>
    where
        T: OperationOutput,
    {
        type Output = Vec<T::Output>;
    }

    impl<S, T> Operation<S> for Vec<T>
    where
        T: Operation<S, Output: Send> + Send,
    {
        async fn exec_operation(self, pool: &mut S::Connection) -> Self::Output
        where
            S: sqlx::Database,
        {
            let mut v = vec![];
            for each in self {
                v.push(each.exec_operation(pool).await);
            }
            v
        }
    }

    impl OperationOutput for () {
        type Output = ();
    }

    impl<S: Database> Operation<S> for () {
        async fn exec_operation(self, _: &mut S::Connection) -> Self::Output {
            ()
        }
    }
}

pub mod boxed_operation {
    use crate::operations::{Operation, OperationOutput};
    use futures::{Future, FutureExt};
    use sqlx::Database;
    use std::{any::Any, pin::Pin};

    pub trait BoxedOperation<S: Database>: Send + Any {
        fn exec_boxed<'c>(
            self: Box<Self>,
            pool: &'c mut S::Connection,
        ) -> Pin<Box<dyn Future<Output = Box<dyn Any + Send>> + Send + 'c>>;
    }

    impl<T, S> BoxedOperation<S> for T
    where
        T: Send + Operation<S> + 'static,
        S: Database,
    {
        fn exec_boxed<'c>(
            self: Box<Self>,
            pool: &'c mut S::Connection,
        ) -> Pin<Box<dyn Future<Output = Box<dyn Any + Send>> + Send + 'c>> {
            Box::pin(async {
                Operation::exec_operation(*self, pool)
                    .map(|f| Box::new(f) as Box<dyn Any + Send>)
                    .await
            })
        }
    }

    impl<S> OperationOutput for Box<dyn BoxedOperation<S> + Send> {
        type Output = Box<dyn Any + Send>;
    }

    impl<S> Operation<S> for Box<dyn BoxedOperation<S> + Send> {
        fn exec_operation(
            self,
            pool: &mut S::Connection,
        ) -> impl Future<Output = Self::Output> + Send
        where
            S: Database,
        {
            BoxedOperation::exec_boxed(self, pool)
        }
    }
}

pub mod execute_expression {
    use crate::database_extention::DatabaseExt;
    use crate::execute::Executable;
    use crate::fix_executor::ExecutorTrait;
    use crate::operations::OperationOutput;
    use crate::{
        operations::Operation,
        query_builder::{Expression, StatementBuilder},
    };
    use sqlx::Database;

    pub struct ExpressionAsOperation<E>(pub E);

    impl<E: Send> OperationOutput for ExpressionAsOperation<E> {
        type Output = ();
    }

    impl<S, E> Operation<S> for ExpressionAsOperation<E>
    where
        E: for<'q> Expression<'q, S>,
        E: Send,
        S: DatabaseExt,
        S: ExecutorTrait,
    {
        async fn exec_operation(self, pool: &mut S::Connection) -> Self::Output
        where
            S: Database,
        {
            let mut qb = StatementBuilder::default();
            self.0.expression(&mut qb);

            let (stmt, arg) = qb.unwrap();
            S::execute(
                pool,
                Executable {
                    string: stmt.as_str(),
                    arguments: arg,
                },
            )
            .await
            .unwrap();
        }
    }
}
