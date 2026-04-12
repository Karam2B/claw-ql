//! I often find lifetimes to take extra effort to make sense of and test
//! this guide helps going through all the lifetimes in sqlx and here and explain them
//!
//! think of lifetimes as 'generics over let statements' as oppose of 'generics over types' like in `T` in `Vec<T>`, to understand a lifetime, you have to kind of `let` statement the lifetime refers to, and what types of restriction that impose (from the time a ref created to the time the lifetime drops)
//!

mod expression_lifetime {
    use sqlx::{FromRow, Sqlite};

    use crate::{
        connect_in_memory::ConnectInMemory,
        query_builder::{Expression, OpExpression, QueryBuilder},
        use_executor,
    };

    struct Str<'a>(&'a str);

    impl<'q> OpExpression for Str<'q> {}

    impl<'q> Expression<'q, Sqlite> for Str<'q> {
        fn expression(self, ctx: &mut QueryBuilder<'q, Sqlite>) {
            ctx.syntax(&"SELECT ");
            ctx.bind(self.0);
            ctx.syntax(&";");
        }
    }

    #[tokio::test]
    async fn main() {
        let pool = Sqlite::connect_in_memory().await;

        let mut statment = String::from("hello world");

        // lifetime created and held by QueryBuilder<'statement, _>
        let holding_lifetime = QueryBuilder::new(Str(&/*'statment*/ statment));

        // let _ = &mut statment;

        // until qb drops (here) there are restrictions on what to do with statment let variable
        // for example uncommenting the above code will result in error
        let s = use_executor!(fetch_one(&pool, holding_lifetime)).unwrap();

        let _ = &mut statment;

        assert_eq!(
            <(String,) as FromRow<_>>::from_row(&s).unwrap(),
            (String::from("hello world"),)
        );
    }
}

mod operation_lifetime {}
