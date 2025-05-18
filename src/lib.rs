use sqlx::Database;

pub mod build_tuple;
pub mod execute;
pub mod expressions;
pub mod collections;
pub mod quick_query;
pub mod select_one;
pub mod select_st;

pub mod prelude {
    pub use super::execute::Execute;
    pub use crate::expressions::exports::*;
    pub use crate::select_st::join;
    pub use crate::select_st::order_by;
    pub mod stmt {
        pub use crate::select_st::SelectSt;
    }
}

pub trait QueryBuilder: Database {
    type Fragment;
    type Context1: Default + 'static;
    type Context2: From<Self::Context1>;

    fn build_sql_part_back(ctx: &mut Self::Context2, from: Self::Fragment) -> String;

    type Output;

    fn build_query(
        ctx1: Self::Context1,
        f: impl FnOnce(&mut Self::Context2) -> String,
    ) -> (String, Self::Output);

    fn handle_bind_item<T>(t: T, ctx: &mut Self::Context1) -> Self::Fragment
    where
        T: BindItem<Self> + 'static;

    fn handle_accept<T>(t: T, ctx: &mut Self::Context1) -> Self::Fragment
    where
        T: 'static + Send,
        Self: Accept<T>;
}

pub trait IdentSafety {
    #[track_caller]
    fn check(ident: &str);
}
impl IdentSafety for () {
    fn check(_: &str) {}
}

pub(crate) mod unstable {
    pub struct Unsateble;
}

/// very similar signature to ToString, but take care of things
/// like sql-injection and ident-safety
///
/// This trait decide which type can be accepted in the SQL syntax
/// but is not binding data to the SQL buffer. identity safety is
/// also important to consider -- does this type have sql-injection?
/// it is accessing data it should not access or result in runtime
/// error?
///
/// implimenting AcceptNoneBind is not stable for now
/// because its linked to the IdentSafety feature which is not
/// fully implemented or understood so far
pub trait AcceptNoneBind {
    type IdentSafety: IdentSafety;
    fn accept(self, _: unstable::Unsateble) -> String;
}

// impl AcceptNoneBind for &str {
//     fn to_string(self) -> String {
//         <str>::to_string(self)
//     }
// }

pub trait Buildable: Sized {
    type Database: QueryBuilder;
    fn build(self) -> (String, <Self::Database as QueryBuilder>::Output);
}

pub trait BuildableAsRef {
    type Database: QueryBuilder;

    fn build(&self) -> (&str, <Self::Database as QueryBuilder>::Output);
}

impl<St> Buildable for St
where
    St: BuildableAsRef,
{
    type Database = St::Database;
    fn build(self) -> (String, <Self::Database as QueryBuilder>::Output) {
        let (string, output) = <Self as BuildableAsRef>::build(&self);
        (string.to_string(), output)
    }
}

pub trait BindItem<S: QueryBuilder> {
    fn bind_item(self, ctx: &mut S::Context1) -> impl FnOnce(&mut S::Context2) -> String + 'static;
}

pub trait Accept<This>: QueryBuilder + Send {
    fn accept(
        this: This,
        ctx1: &mut Self::Context1,
    ) -> impl FnOnce(&mut Self::Context2) -> String + 'static + Send;
}

pub trait IntoMutArguments<'q, DB>
where
    Self: Sized,
    DB: Database,
{
    const LEN: usize;
    fn into_arguments(self, argument: &mut DB::Arguments<'q>);
}
