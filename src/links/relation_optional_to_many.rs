use sqlx::{ColumnIndex, Decode, Executor, Row, Sqlite, prelude::Type};
use std::marker::PhantomData;

use sqlx::Pool;

use crate::{
    QueryBuilder,
    operations::{
        SimpleOutput,
        collections::{Collection, OnMigrate},
        select_one::GetOneWorker,
    },
    prelude::{col, join, stmt::SelectSt},
};

#[derive(Clone)]
pub struct OptionalToMany<F, T> {
    pub foriegn_key: String,
    pub _pd: PhantomData<(F, T)>,
}

#[derive(Clone)]
pub struct OptionalToManyInverse<F, T> {
    pub foriegn_key: String,
    pub _pd: PhantomData<(F, T)>,
}

// todo add generic implementaion
impl<From, To> OnMigrate<Sqlite> for OptionalToMany<From, To>
where
    From: Collection<Sqlite>,
    To: Collection<Sqlite>,
{
    async fn custom_migration<'e>(
        self,
        exec: impl for<'q> Executor<'q, Database = Sqlite> + Clone,
    ) {

        sqlx::query(&format!(
            "
ALTER TABLE {from_table_name} 
ADD COLUMN {col_name} INT
REFERENCES {to_table_name} (id)
{dio}
ON DELETE SET NULL;
",
        from_table_name = From::table_name(),
        to_table_name = To::table_name(),
        col_name = format!("{}_id", To::table_name().to_lowercase()),
            dio = ""
        ))
        .execute(exec.clone())
        .await
        .unwrap();
    }
}

impl<S, From, To> GetOneWorker<S> for OptionalToMany<From, To>
where
    S: QueryBuilder,
    To: Send + Sync + Collection<S>,
    From: Send + Sync + Collection<S>,
    for<'c> &'c str: ColumnIndex<S::Row>,
    for<'q> i64: Decode<'q, S>,
    i64: Type<S>,
{
    type Output = Option<SimpleOutput<To>>;
    type Inner = Option<(i64, To)>;

    fn on_select(&self, _: &mut Self::Inner, st: &mut SelectSt<S>) {
        st.join(join::left_join {
            foriegn_table: To::table_name().to_string(),
            foriegn_column: "id".to_string(),
            local_column: self.foriegn_key.to_string(),
        });
        st.select(col(&self.foriegn_key).table(From::table_name()));
        To::on_select(st);
    }

    fn from_row(&self, data: &mut Self::Inner, row: &S::Row) {
        let id: Option<i64> = row.get(self.foriegn_key.as_str());
        if let Some(id) = id {
            let value = To::from_row_scoped(row);
            *data = Some((id, value));
        }
    }

    async fn sub_op<'this>(&'this self, _: &'this mut Self::Inner, _: Pool<S>) {}

    fn take(self, data: Self::Inner) -> Self::Output {
        data.map(|(id, attr)| SimpleOutput { id, attr })
    }
}
