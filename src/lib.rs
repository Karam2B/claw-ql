use sqlx::Database;

pub mod select_st;

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

pub trait BindItem<S: QueryBuilder> {
    fn bind_item(self, ctx: &mut S::Context1) -> impl FnOnce(&mut S::Context2) -> String + 'static;
}

pub trait Accept<This>: QueryBuilder + Send {
    fn accept(
        this: This,
        ctx1: &mut Self::Context1,
    ) -> impl FnOnce(&mut Self::Context2) -> String + 'static + Send;
}
