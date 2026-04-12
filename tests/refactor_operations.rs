//! refactoring seq
//! - [x] change to generic trait
//! - [ ] mimic LinkInsertOne
//! - [ ] provide implementations for optional_to_many and insert_one
//!
#![allow(unused)]
#![deny(unused_must_use)]
#![allow(dead_code)]
use std::{any::Any, pin::Pin};

use claw_ql::{
    collections::{Collection, SingleIncremintalInt},
    connect_in_memory::ConnectInMemory,
    database_extention::DatabaseExt,
    from_row::row_helpers::{AliasRowHelper, ManyRowHelper, OneRowHelper},
    links::{Link, relation_optional_to_many::optional_to_many, set_id_mod::SetIdSpec},
    query_builder::{
        ExpressionAsBind, IsOpExpression, ManyExpressions, QueryBuilder,
        functional_expr::{ManyFlat, ManyImplPossible},
        syntax::{comma_join, empty},
    },
    statements::insert_one_statement::{InsertStatement, OneDefault},
    test_module::{self, Todo, TodoPartial, category},
    use_executor,
};
use sqlx::{ColumnIndex, Database, Executor, Sqlite, SqliteConnection, Type};

#[tokio::test]
async fn row_op() {
    let p = Sqlite::connect_in_memory().await;

    sqlx::query(
        "
        CREATE TABLE IF NOT EXISTS Category (
            id INTEGER PRIMARY KEY AUTOINCREMENT,
            title TEXT NOT NULL
        );
        CREATE TABLE IF NOT EXISTS Todo (
            id INTEGER PRIMARY KEY AUTOINCREMENT,
            title TEXT NOT NULL,
            category_id INTEGER,
            FOREIGN KEY (category_id) REFERENCES Category(id)
        );
    ",
    )
    .execute(&p)
    .await
    .unwrap();

    let mut tx = p.begin().await.unwrap();

    let sub_op: Pin<Box<dyn Future<Output = (i64,)>>> = Box::pin(async {
        sqlx::query("INSERT INTO Category (title) VALUES ('cat_1') RETURNING id;")
            .fetch_one(tx.as_mut())
            .await
            .unwrap()
            .from_row::<(i64,)>()
            .unwrap()
    });

    let r = sub_op.await;

    let r = sqlx::query(
        "
        INSERT INTO Todo (title, category_id) VALUES ('new_todo', $1) RETURNING id;
    ",
    )
    .bind(r.0)
    .fetch_one(tx.as_mut())
    .await
    .unwrap();

    tx.commit().await.unwrap();

    let output = sqlx::query("SELECT title, category_id FROM Todo;")
        .fetch_all(&p)
        .await
        .unwrap()
        .from_rows::<(String, i64)>()
        .unwrap();

    pretty_assertions::assert_eq!(output, vec![("new_todo".to_string(), 1)]);
}

pub trait LinkInsertOne<S> {
    type PreOp;
    fn pre_op(&self) -> Self::PreOp;

    type InsertExtentionIdents;
    type InsertExtentionValues;
    type NextStage;
    fn insert_extention(
        self,
        output: <Self::PreOp as Operation<S>>::Output,
    ) -> (
        Self::InsertExtentionIdents,
        Self::InsertExtentionValues,
        Self::NextStage,
    )
    where
        Self::PreOp: Operation<S>;
}

pub struct LinkB;

impl<An> Link<An> for LinkB {
    type Spec = LinkB;
    fn spec(self, base: &An) -> Self::Spec {
        self
    }
}

impl LinkInsertOne<Sqlite> for LinkB {
    type PreOp = Box<dyn BoxedOperation<Sqlite>>;
    fn pre_op(&self) -> Self::PreOp {
        Box::new(PreInsert)
    }
    type InsertExtentionIdents = String;
    type InsertExtentionValues = ExpressionAsBind<i64>;
    type NextStage = ();
    fn insert_extention(
        self,
        output: <Self::PreOp as Operation<Sqlite>>::Output,
    ) -> (String, ExpressionAsBind<i64>, ())
    where
        Self::PreOp: Operation<Sqlite>,
    {
        (
            "category_id".to_string(),
            ExpressionAsBind(
                *output
                    .downcast::<i64>()
                    .expect("links should be consistant"),
            ),
            (),
        )
    }
}

impl<From, To> LinkInsertOne<Sqlite> for SetIdSpec<optional_to_many<String, From, To>, i64> {
    type PreOp = ();

    fn pre_op(&self) -> Self::PreOp {}

    // type CurrentTable = ExpressionAsBind<i64>;
    type InsertExtentionIdents = Vec<String>;
    type InsertExtentionValues = ExpressionAsBind<i64>;
    type NextStage = ();

    fn insert_extention(
        self,
        output: <Self::PreOp as Operation<Sqlite>>::Output,
    ) -> (Vec<String>, Self::InsertExtentionValues, ())
    where
        Self::PreOp: Operation<Sqlite>,
    {
        (
            vec![self.og_spec.foriegn_key.clone()],
            ExpressionAsBind(self.input),
            (),
        )
    }
}

pub struct PreInsert;
impl Operation<Sqlite> for PreInsert {
    type Output = i64;
    async fn exec_op(self, conn: &mut SqliteConnection) -> Self::Output {
        sqlx::query("INSERT INTO Category (title) VALUES ('cat_1') RETURNING id;")
            .fetch_one(conn)
            .await
            .unwrap()
            .from_row::<(i64,)>()
            .unwrap()
            .0
    }
}

pub trait Operation<S> {
    type Output;
    fn exec_op(self, conn: &mut S::Connection) -> impl Future<Output = Self::Output> + Send
    where
        S: Database;
}

impl<S> Operation<S> for () {
    type Output = ();
    async fn exec_op(self, conn: &mut <S>::Connection) -> Self::Output
    where
        S: Database,
    {
    }
}

pub struct SimpleOp<L>(pub L);

impl<L> Operation<Sqlite> for SimpleOp<L>
where
    L: LinkInsertOne<
            Sqlite,
            PreOp: Operation<Sqlite>,
            InsertExtentionValues = ExpressionAsBind<i64>,
            InsertExtentionIdents = String,
            NextStage: Send,
        > + Send,
{
    type Output = ();
    async fn exec_op(self, conn: &mut SqliteConnection) -> Self::Output {
        sqlx::query(
            "
        CREATE TABLE IF NOT EXISTS Category (
            id INTEGER PRIMARY KEY AUTOINCREMENT,
            title TEXT NOT NULL
        );
        CREATE TABLE IF NOT EXISTS Todo (
            id INTEGER PRIMARY KEY AUTOINCREMENT,
            title TEXT NOT NULL,
            category_id INTEGER,
            FOREIGN KEY (category_id) REFERENCES Category(id)
        );
    ",
        )
        .execute(&mut *conn)
        .await
        .unwrap();

        let output = self.0.pre_op().exec_op(&mut *conn).await;

        let output = self.0.insert_extention(output);

        assert_eq!(output.0, String::from("category_id"));

        sqlx::query(
            "
        INSERT INTO Todo (title, category_id) VALUES ('new_todo', $1) RETURNING id;
    ",
        )
        .bind(output.1.0)
        .fetch_one(&mut *conn)
        .await
        .unwrap();
    }
}

pub struct InsertTodo<L> {
    partial: Todo,
    links: L,
}

pub trait NamedExpressions<'q, S> {
    type Names: ManyExpressions<'q, S>;
    type Values: ManyExpressions<'q, S>;
    fn name_values(self) -> (Self::Names, Self::Values);
}

impl<S, L> Operation<S> for InsertTodo<L>
where
    S: DatabaseExt,
    usize: ColumnIndex<S::Row>,
    i64: Type<S> + for<'q> sqlx::Decode<'q, S>,
    for<'c> &'c mut S::Connection: sqlx::Executor<'c, Database = S>,
    L: Link<test_module::todo> + Send,
    L::Spec: Send
        + LinkInsertOne<
            S,
            PreOp: Operation<S>,
            InsertExtentionIdents: for<'q> ManyExpressions<'q, S>,
            InsertExtentionValues: for<'q> ManyExpressions<'q, S>,
            NextStage: Send,
        >,
    Todo: for<'q> ManyExpressions<'q, S>,
    test_module::todo: Collection<Id = SingleIncremintalInt>,
{
    type Output = ();
    async fn exec_op(self, conn: &mut <S>::Connection) -> Self::Output
    where
        S: Database,
    {
        let links = self.links.spec(&test_module::todo);

        let pre_op = links.pre_op().exec_op(&mut *conn).await;
        let (idents, values, links) = links.insert_extention(pre_op);

        let qb = QueryBuilder::<'_, S>::new(InsertStatement {
            table_name: "Todo".to_string(),
            identifiers: ManyFlat((idents, test_module::todo)),
            values: OneDefault(ManyFlat((values, self.partial))),
            returning: vec!["id"],
        });

        let id = use_executor!(fetch_one(&mut *conn, qb))
            .unwrap()
            .from_row::<(i64,)>()
            .unwrap()
            .0;
    }
}

pub trait BoxedOperation<S: Database>: Send {
    fn exec_boxed<'c>(
        self: Box<Self>,
        conn: &'c mut S::Connection,
    ) -> Pin<Box<dyn Future<Output = Box<dyn Any>> + Send + 'c>>;
}

impl<S: Database, T> BoxedOperation<S> for T
where
    T: Operation<S> + Send + 'static,
{
    fn exec_boxed<'c>(
        self: Box<Self>,
        conn: &'c mut S::Connection,
    ) -> Pin<Box<dyn Future<Output = Box<dyn Any>> + Send + 'c>> {
        Box::pin(async { Box::new(self.exec_op(conn).await) as Box<dyn Any> })
    }
}

impl<S: Database> Operation<S> for Box<dyn BoxedOperation<S>> {
    type Output = Box<dyn Any>;
    async fn exec_op(self, conn: &mut S::Connection) -> Self::Output {
        self.exec_boxed(conn).await
    }
}

#[tokio::test]
async fn test_simple_op() {
    let p = Sqlite::connect_in_memory().await;
    let mut tx = p.begin().await.unwrap();

    SimpleOp(LinkB).exec_op(&mut tx).await;

    tx.commit().await.unwrap();

    // test output
    let output = sqlx::query("SELECT title, category_id FROM Todo;")
        .fetch_all(&p)
        .await
        .unwrap()
        .from_rows::<(String, i64)>()
        .unwrap();

    pretty_assertions::assert_eq!(output, vec![("new_todo".to_string(), 1)]);
}

#[cfg(test)]
mod test {
    use super::Operation;
    use crate::InsertTodo;
    use crate::LinkB;
    use crate::test_module::*;
    use claw_ql::connect_in_memory::ConnectInMemory;
    use claw_ql::from_row::FromRowAlias;
    use claw_ql::from_row::pre_alias;
    use claw_ql::from_row::row_helpers::AliasRowHelper;
    use claw_ql::links::set_id_mod::set_id;
    use claw_ql::links::set_new_mod::set_new;
    use sqlx::Row;
    use sqlx::Sqlite;

    #[tokio::test]
    async fn test_insert_one() {
        let pool = Sqlite::connect_in_memory().await;

        sqlx::query(
            "
            CREATE TABLE IF NOT EXISTS Category (
                id INTEGER PRIMARY KEY AUTOINCREMENT,
                title TEXT NOT NULL
            );
            CREATE TABLE IF NOT EXISTS Todo (
                id INTEGER PRIMARY KEY AUTOINCREMENT,
                title TEXT NOT NULL,
                done BOOLEAN NOT NULL,
                description TEXT,
                category_id INTEGER,
                FOREIGN KEY (category_id) REFERENCES Category(id)
            );
            INSERT INTO Category (title) VALUES ('category_1');
            ",
        )
        .execute(&pool)
        .await
        .unwrap();

        let mut tx = pool.begin().await.unwrap();

        InsertTodo {
            partial: Todo {
                title: "first_todo".to_string(),
                done: true,
                description: Some("description_1".to_string()),
            },
            links: set_id {
                to: category,
                id: 1,
            },
        }
        .exec_op(tx.as_mut())
        .await;

        tx.commit().await.unwrap();

        // move to query
        let row = sqlx::query(
            "SELECT 
                    t.title as todo_title, 
                    t.done as todo_done, 
                    t.description as todo_description, 
                    t.category_id, 
                    c.title as category_title
                FROM Todo t LEFT JOIN Category c ON t.category_id = c.id;",
        )
        .fetch_one(&pool)
        .await
        .unwrap();

        let result = row.row_pre_alias(&todo, "todo_").unwrap();

        pretty_assertions::assert_eq!(
            result,
            Todo {
                title: "first_todo".to_string(),
                done: true,
                description: Some("description_1".to_string()),
            }
        );

        let category_id: i64 = row.get("category_id");

        pretty_assertions::assert_eq!(category_id, 1);

        let result = row.row_pre_alias(&category, "category_").unwrap();

        pretty_assertions::assert_eq!(
            result,
            Category {
                title: "category_1".to_string(),
            }
        );
    }
}

// pub trait Operation2<'q, S>: Send {
//     type Output: Send;
//     fn exec_conn(
//         self,
//         conn: &'q mut S::Connection,
//     ) -> impl Future<Output = Self::Output> + Send + 'q
//     where
//         S: Database;
// }
// pub trait LifetimedOperation<'c, S: Database>: Send + 'static {
//     fn exec_boxed(
//         self: Box<Self>,
//         pool: &'c mut S::Connection,
//     ) -> Pin<Box<dyn Future<Output = Box<dyn Any + Send>> + Send + 'c>>;
// }

// impl<'c, T, S> LifetimedOperation<'c, S> for T
// where
//     T: Send + Operation2<'c, S, Output: Send> + 'static,
//     S: Database,
//     for<'e> &'e mut <S as sqlx::Database>::Connection: Executor<'e, Database = S>,
// {
//     fn exec_boxed(
//         self: Box<Self>,
//         pool: &'c mut S::Connection,
//     ) -> Pin<Box<dyn Future<Output = Box<dyn Any + Send>> + Send + 'c>> {
//         Box::pin(async move {
//             todo!()
//             // Operation2::exec_conn(*self, pool)
//             //     .map(|f| Box::new(f) as Box<dyn Any + Send>)
//             //     .await
//         })
//     }
// }

// impl<'q, S: 'static> Operation2<'q, S> for Vec<Box<dyn LifetimedOperation<'q, S>>>
// // where
// //     for<'q> T: LifetimedOperation<'q, S> + Send,
// {
//     type Output = Vec<Box<dyn Any + Send>>;
//     fn exec_conn(
//         self,
//         conn: &'q mut <S>::Connection,
//     ) -> impl Future<Output = Self::Output> + Send + 'q
//     where
//         S: Database,
//     {
//         async move {
//             let mut v = vec![];
//             for each in self {
//                 // v.push(each.exec_boxed(conn).await);
//             }
//             v
//         }
//     }

//     // async fn exec_operation(self, pool: sqlx::Pool<S>) -> Self::Output
//     // where
//     //     S: sqlx::Database,
//     // {
//     // }
// }

// impl<S, T> Operation2<S> for Vec<T>
// where
//     T: Operation2<S, Output: Send> + Send,
// {
//     type Output = Vec<T::Output>;

//     fn exec_conn(self, conn: &mut <S>::Connection) -> impl Future<Output = Self::Output> + Send
//     where
//         S: Database,
//     {
//         async move {
//             let mut v = vec![];
//             for each in self {
//                 v.push(each.exec_conn(conn).await);
//             }
//             v
//         }
//     }
// async fn exec_operation(self, pool: sqlx::Pool<S>) -> Self::Output
// where
//     S: sqlx::Database,
// {
//     let mut v = vec![];
//     for each in self {
//         v.push(each.exec_operation(pool.clone()).await);
//     }
//     v
// }

// impl<'q, S, $($t,)* > Operation2<'q, S> for ($($t,)*)
// where
//     $($t: Operation2<'q, S>,)*
// {
//     type Output = (
//         $($t::Output,)*
//     );

//     async fn exec_conn(self, pool: &'q mut S::Connection) -> Self::Output
//     where
//         S: sqlx::Database,
//     {
//         ($(
//             paste!(self.$part).exec_conn(pool).await,
//         )*)
//     }
// }
