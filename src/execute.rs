use sqlx::{Database, Execute};
use std::mem;

pub struct Executable<'q, A> {
    pub string: &'q str,
    pub arguments: A,
}

impl<'q, S: Database> Execute<'q, S> for Executable<'q, S::Arguments<'q>> {
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

/// this macro maynot be always be needed, as sometime you don't need to break lifetimes
/// ,valid use examples
/// `use_executor(fetch_optional(&Sqlite::connect_in_memory().await, b))`
/// `use_executor(fetch_many(pool, b).map(|e| "").collect::<Vec<&str>>())`
#[macro_export]
macro_rules! use_executor {
    ($fn_name:ident ($executor: expr, $execute:ident) $($rest:tt)*) => {

        {
            let (stmt, arg): (String, _) = $execute.unwrap();
            let query_result = ::sqlx::Executor::$fn_name(
                $executor,
                #[allow(unused_doc_comments)]
                $crate::execute::Executable {

                    /// safer alternative is to leak
                    /// string: Box::<str>::leak(stmt.into_boxed_str()),
                    string: unsafe {
                        /// breaking lifetimes! because I'm dropping `stmt` after static query_result
                        /// I will not have any use-after-free bug, which is the bug lifetimes are solving

                        /// the problem is in the strict signature of sqlx::Execute::sql function
                        /// the only `safe` way to avoid it is to rewrite sqlx::Executor and sqlx::Execute traits
                        &*(stmt.as_str() as *const _)
                    },


                    arguments: arg,
                },
            )$($rest)*.await;

            ::std::mem::drop(stmt);

            query_result
        }
    }
}

// // failed attempt
// #[allow(unused)]
// struct BreakLifetimeExecutable<'q, S: Database, A> {
//     pub string: &'q str,
//     pub arguments: A,
//     pub db: PhantomData<S>,
// }

// impl<'a, S: Database> Execute<'static, S>
//     for BreakLifetimeExecutable<'a, S, S::Arguments<'static>>
// {
//     fn sql(&self) -> &'static str {
//         unsafe { &*(self.string as *const _) }
//     }
//     fn take_arguments(
//         &mut self,
//     ) -> Result<Option<<S as Database>::Arguments<'static>>, sqlx::error::BoxDynError> {
//         Ok(Some(mem::take(&mut self.arguments)))
//     }

//     fn statement(&self) -> Option<&<S as Database>::Statement<'static>> {
//         None
//     }

//     fn persistent(&self) -> bool {
//         false
//     }
// }
