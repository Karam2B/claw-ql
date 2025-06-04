// identity management is aimed to:
// 1. prevent sql injection by rejecting unrecognized input
// 2. reduce errors either at a runtime or buildtime
// 3. help in migration from one version of the schema to another
pub trait AcceptNoneBind<I: IdentManager, Table, Column> {
    fn accept(self) -> String;
    fn runtime_check_table(&self, _: &I::Context) {}
    fn runtime_check_column(&self, _: &I::Context) {}
}

pub trait IdentManager {
    type Context;
}

impl IdentManager for () {
    type Context = ();
}

trait ColumnBelongToTable<C, T>: IdentManager {}
impl ColumnBelongToTable<&str, &str> for () {}
impl ColumnBelongToTable<String, String> for () {}
impl ColumnBelongToTable<String, &str> for () {}
impl ColumnBelongToTable<&str, String> for () {}

#[allow(non_camel_case_types)]
pub struct col<C>(pub C);

impl<I, C, T> AcceptNoneBind<I, T, C> for col<C>
where
    I: ColumnBelongToTable<C, T>,
    C: ToString,
{
    fn accept(self) -> String {
        self.0.to_string()
    }
    fn runtime_check_table(&self, _: &I::Context) {}
    fn runtime_check_column(&self, _: &I::Context) {}
}

#[allow(non_camel_case_types)]
pub struct table<T>(pub T);

impl<I, C, T> AcceptNoneBind<I, T, C> for table<C>
where
    I: ColumnBelongToTable<C, T>,
    C: ToString,
{
    fn accept(self) -> String {
        self.0.to_string()
    }
    fn runtime_check_table(&self, _: &I::Context) {}
    fn runtime_check_column(&self, _: &I::Context) {}
}
