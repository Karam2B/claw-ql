// run
// cargo watch -- cargo run --example id_concept --features="experimental_id_trait"
#![allow(unused)]
#![warn(unused_must_use)]
use ::claw_ql::prelude::macro_derive_collection::*;
use claw_ql::{
    ConnectInMemory,
    collections::{Id, SingleIncremintalInt},
    update_mod::update,
};
use sqlx::{Pool, Sqlite};

pub struct Todo {
    pub title: String,
    pub done: bool,
    pub description: Option<String>,
}

#[derive(Default, Debug)]
pub struct TodoPartial {
    pub title: update<String>,
    pub done: update<bool>,
    pub description: update<Option<String>>,
}

#[allow(non_camel_case_types)]
#[derive(Clone, Default)]
pub struct todo;

impl CollectionBasic for todo {
    type LinkedData = Todo;
    fn table_name_lower_case(&self) -> &'static str {
        "todo"
    }
    fn table_name(&self) -> &'static str {
        "Todo"
    }
    fn members(&self) -> Vec<String> {
        vec![
            String::from("title"),
            String::from("done"),
            String::from("description"),
        ]
    }
}

impl<S> OnMigrate<S> for todo
where
    S: QueryBuilder<Output = <S as Database>::Arguments<'static>>,
    for<'q> S::Arguments<'q>: IntoArguments<'q, S>,
    S: QueryBuilder + DatabaseDefaultPrimaryKey,
    <S as DatabaseDefaultPrimaryKey>::KeyType:
        Type<S> + for<'c> Decode<'c, S> + for<'e> Encode<'e, S>,
    String: Type<S> + for<'c> Decode<'c, S> + for<'e> Encode<'e, S>,
    bool: Type<S> + for<'c> Decode<'c, S> + for<'e> Encode<'e, S>,
    Option<String>: Type<S> + for<'c> Decode<'c, S> + for<'e> Encode<'e, S>,
{
    fn custom_migration<'e>(
        &self,
        exec: impl for<'q> Executor<'q, Database = S> + Clone,
    ) -> impl Future<Output = ()> {
        async move {
            let mut stmt = CreateTableSt::init(header::create, self.table_name());
            stmt.column_def("id", primary_key::<S>());
            stmt.column_def("title", col_type_check_if_null::<String>());
            stmt.column_def("done", col_type_check_if_null::<bool>());
            stmt.column_def("description", col_type_check_if_null::<Option<String>>());
            stmt.execute(exec).await.unwrap();
        }
    }
}

impl HasHandler for Todo {
    type Handler = todo;
}

impl HasHandler for TodoPartial {
    type Handler = todo;
}

impl<S> Collection<S> for todo
where
    S: QueryBuilder,
    for<'s> &'s str: sqlx_::ColumnIndex<<S as Database>::Row>,
    String: Type<S> + for<'c> Decode<'c, S> + for<'e> Encode<'e, S>,
    S: Accept<String>,
    bool: Type<S> + for<'c> Decode<'c, S> + for<'e> Encode<'e, S>,
    S: Accept<bool>,
    Option<String>: Type<S> + for<'c> Decode<'c, S> + for<'e> Encode<'e, S>,
    S: Accept<Option<String>>,
    SingleIncremintalInt: Id<S>,
    <SingleIncremintalInt as Id<S>>::Data: Type<S> + for<'c> Decode<'c, S> + for<'c> Encode<'c, S>,
{
    type Partial = TodoPartial;
    type Data = Todo;
    type IdData = <SingleIncremintalInt as Id<S>>::Data;
    type Id = SingleIncremintalInt;
    fn on_select(&self, stmt: &mut SelectSt<S>) {
        stmt.select(col("title").table("Todo").alias("todo_title"));
        stmt.select(col("done").table("Todo").alias("todo_done"));
        stmt.select(col("description").table("Todo").alias("todo_description"));
    }
    fn on_insert(&self, this: Self::Data, stmt: &mut InsertOneSt<S>)
    where
        S: sqlx::Database,
    {
        stmt.col("title".to_string(), this.title);
        stmt.col("done".to_string(), this.done);
        stmt.col("description".to_string(), this.description);
    }
    fn on_update(&self, this: Self::Partial, stmt: &mut UpdateSt<S>)
    where
        S: claw_ql::QueryBuilder,
    {
        match this.title {
            update::keep => {}
            update::set(set) => stmt.set_col("title".to_string(), set),
        }
        match this.done {
            update::keep => {}
            update::set(set) => stmt.set_col("done".to_string(), set),
        }
        match this.description {
            update::keep => {}
            update::set(set) => stmt.set_col("description".to_string(), set),
        }
    }

    fn from_row_noscope(&self, row: &<S as Database>::Row) -> Self::Data {
        Self::Data {
            title: row.get("title"),
            done: row.get("done"),
            description: row.get("description"),
        }
    }
    fn from_row_scoped(&self, row: &<S as Database>::Row) -> Self::Data {
        Self::Data {
            title: row.get("todo_title"),
            done: row.get("todo_done"),
            description: row.get("todo_description"),
        }
    }
}

async fn select_one_op_exec_op<C>(this: C, db: Pool<Sqlite>)
where
    C: Collection<Sqlite>,
{
    let mut st = SelectSt::init(this.table_name().to_string());

    st.select(
        col(C::Id::ident())
            .table(this.table_name())
            .alias("local_id"),
    );

    let s = st
        .fetch_optional(&db, |r| {
            let id: C::IdData = r.get("local_id");

            Ok(())
        })
        .await
        .unwrap();
}

#[tokio::main]
async fn main() {
    let pool = Sqlite::connect_in_memory().await;

    select_one_op_exec_op(todo, pool).await;
}
