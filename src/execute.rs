use _private::InnerExecutable;
use sqlx::{Database, Executor};
use std::mem::take;

use crate::{Buildable, QueryBuilder};

pub trait Execute<S: Database>: Sized {
    fn build(self) -> (String, S::Arguments<'static>);
    fn execute<E>(
        self,
        executor: E,
    ) -> impl Future<Output = Result<S::QueryResult, sqlx::Error>> + Send
    where
        E: for<'e> sqlx::Executor<'e, Database = S>,
    {
        let (query, args) = self.build();
        async move {
            InnerExecutable {
                stmt: query.as_str(),
                buffer: args,
                persistent: false,
            }
            .execute(executor)
            .await
        }
    }
    fn fetch_one<E, O, F>(
        self,
        executor: E,
        with: F,
    ) -> impl Future<Output = Result<O, sqlx::Error>> + Send
    where
        E: for<'e> sqlx::Executor<'e, Database = S>,
        F: FnOnce(S::Row) -> Result<O, sqlx::Error> + Send,
    {
        let (query, args) = self.build();
        async move {
            InnerExecutable {
                stmt: query.as_str(),
                buffer: args,
                persistent: false,
            }
            .fetch_one_with(executor, with)
            .await
        }
    }
    fn fetch_optional<E, O, F>(
        self,
        executor: E,
        with: F,
    ) -> impl Future<Output = Result<Option<O>, sqlx::Error>> + Send
    where
        E: for<'e> sqlx::Executor<'e, Database = S>,
        F: FnOnce(S::Row) -> Result<O, sqlx::Error> + Send,
    {
        let (query, args) = self.build();
        async move {
            InnerExecutable {
                stmt: query.as_str(),
                buffer: args,
                persistent: false,
            }
            .fetch_optional_with(executor, with)
            .await
        }
    }
    #[track_caller]
    fn fetch_all<E, O, F>(
        self,
        executor: E,
        with: F,
    ) -> impl Future<Output = Result<Vec<O>, sqlx::Error>> + Send
    where
        E: for<'e> sqlx::Executor<'e, Database = S>,
        F: FnMut(S::Row) -> Result<O, sqlx::Error> + Send,
    {
        let (query, args) = self.build();
        async move {
            InnerExecutable {
                stmt: query.as_str(),
                buffer: args,
                persistent: false,
            }
            .fetch_all_with(executor, with)
            .await
        }
    }
}

impl<T, S> Execute<S> for T
where
    T: Buildable<Database = S> + Sized,
    S: QueryBuilder<Output = <S as Database>::Arguments<'static>>,
{
    fn build(self) -> (String, <S as Database>::Arguments<'static>) {
        self.build()
    }
}

mod _private {
    use sqlx::Database;

    pub struct InnerExecutable<'s, 'q, DB: Database> {
        pub stmt: &'s str,
        pub buffer: DB::Arguments<'q>,
        pub persistent: bool,
    }
}

impl<'s, 'q, S: Database> InnerExecutable<'s, 'q, S> {
    pub async fn execute<E>(self, executor: E) -> Result<S::QueryResult, sqlx::Error>
    where
        for<'c> E: Executor<'q, Database = S>,
    {
        #[cfg(feature = "trace")]
        tracing::debug!("execute");
        executor
            .execute(InnerExecutable {
                // SAFETY: the output of execute is free of
                // any reference of self, which means that
                // self can drop after the await, and the
                // result can live longer
                //
                // I tried Self: 'q, and &'q mut self can
                // be used to solve this issue
                //
                // I saw the same issue before, this is
                // either a problem in sqlx or rust is not
                // advanced enough to catch this pattern, but
                // i'm sure this code is 100% safe
                stmt: unsafe { &*(self.stmt as *const _) },
                ..self
            })
            .await
    }
    pub async fn fetch_one_with<E, O, F>(self, executor: E, with: F) -> Result<O, sqlx::Error>
    where
        F: FnOnce(S::Row) -> Result<O, sqlx::Error>,
        for<'c> E: Executor<'c, Database = S>,
    {
        #[cfg(feature = "trace")]
        tracing::debug!("fetch one");
        let execute = InnerExecutable {
            // SAFETY: the output of execute is free of
            // any reference of self, which means that
            // self can drop after the await, and the
            // result can live longer
            //
            // I tried Self: 'q, and &'q mut self can
            // be used to solve this issue
            //
            // I saw the same issue before, this is
            // either a problem in sqlx or rust is not
            // advanced enough to catch this pattern, but
            // i'm sure this code is 100% safe
            stmt: unsafe { &*(self.stmt as *const _) },
            ..self
        };

        let res = executor.fetch_one(execute).await;

        match res {
            Ok(r) => Ok(with(r)?),
            Err(e) => Err(e),
        }
    }

    pub async fn fetch_all_with<E, O, F>(
        self,
        executor: E,
        mut with: F,
    ) -> Result<Vec<O>, sqlx::Error>
    where
        F: FnMut(S::Row) -> Result<O, sqlx::Error>,
        for<'c> E: Executor<'c, Database = S>,
    {
        #[cfg(feature = "trace")]
        tracing::debug!("fetch all");
        let execute = InnerExecutable {
            // SAFETY: the output of execute is free of
            // any reference of self, which means that
            // self can drop after the await, and the
            // result can live longer
            //
            // I tried Self: 'q, and &'q mut self can
            // be used to solve this issue
            //
            // I saw the same issue before, this is
            // either a problem in sqlx or rust is not
            // advanced enough to catch this pattern, but
            // i'm sure this code is 100% safe
            stmt: unsafe { &*(self.stmt as *const _) },
            ..self
        };

        executor.fetch_all(execute).await.map(|r| {
            r.into_iter()
                .map(|r| with(r).expect("failed to decode"))
                .collect::<Vec<_>>()
        })
    }

    pub async fn fetch_optional_with<E, O, F>(
        self,
        executor: E,
        with: F,
    ) -> Result<Option<O>, sqlx::Error>
    where
        F: FnOnce(S::Row) -> Result<O, sqlx::Error>,
        for<'c> E: Executor<'c, Database = S>,
    {
        #[cfg(feature = "trace")]
        tracing::debug!("fetch optional");
        let execute = InnerExecutable {
            // SAFETY: the output of execute is free of
            // any reference of self, which means that
            // self can drop after the await, and the
            // result can live longer
            //
            // I tried Self: 'q, and &'q mut self can
            // be used to solve this issue
            //
            // I saw the same issue before, this is
            // either a problem in sqlx or rust is not
            // advanced enough to catch this pattern, but
            // i'm sure this code is 100% safe
            stmt: unsafe { &*(self.stmt as *const _) },
            ..self
        };

        let op = executor.fetch_optional(execute).await;

        match op {
            Ok(Some(r)) => Ok(Some(with(r)?)),
            _ => Ok(None),
        }
    }
}

impl<'q, DB: Database> sqlx::Execute<'q, DB> for InnerExecutable<'q, 'q, DB> {
    fn sql(&self) -> &'q str {
        self.stmt
    }

    fn persistent(&self) -> bool {
        self.persistent
    }

    fn statement(&self) -> Option<&DB::Statement<'q>> {
        None
    }

    fn take_arguments(
        &mut self,
    ) -> Result<Option<<DB as Database>::Arguments<'q>>, sqlx::error::BoxDynError> {
        Ok(Some(take(&mut self.buffer)))
    }
}
