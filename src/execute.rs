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
