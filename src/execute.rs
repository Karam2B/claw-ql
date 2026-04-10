use sqlx::{Database, Execute, Executor};
use std::{
    marker::PhantomData,
    mem::{self, take},
};

pub struct Executable<'q, S: Database, A> {
    pub string: &'q str,
    pub arguments: A,
    pub db: PhantomData<S>,
}

impl<'q, S: Database> Execute<'q, S> for Executable<'q, S, S::Arguments<'q>> {
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
