use std::ops::Not;
use std::process::Output;

use convert_case::{Case, Casing};
use serde::Serialize;
use sqlx::{ColumnIndex, Decode, Executor, Row, Sqlite, prelude::Type};

use sqlx::Pool;

use crate::dynamic_client::json_client::SelectOneJsonFragment;
use crate::{
    QueryBuilder,
    operations::{
        SimpleOutput,
        collections::{Collection, OnMigrate},
        select_one::SelectOneFragment,
    },
    prelude::{col, join, stmt::SelectSt},
};

use super::relation::DynamicLinkForRelation;

#[derive(Clone)]
pub struct OptionalToMany<F, T> {
    pub foriegn_key: String,
    pub from: F,
    pub to: T,
}

#[derive(Clone)]
pub struct OptionalToManyInverse<F, T> {
    pub foriegn_key: String,
    pub from: F,
    pub to: T,
}

// todo add generic implementaion
impl<From, To> OnMigrate<Sqlite> for OptionalToMany<From, To>
where
    From: Collection<Sqlite>,
    To: Collection<Sqlite>,
{
    async fn custom_migration<'e>(
        &self,
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
            from_table_name = self.from.table_name(),
            to_table_name = self.to.table_name(),
            col_name = format!("{}_id", self.to.table_name().to_case(Case::Snake)),
            dio = ""
        ))
        .execute(exec.clone())
        .await
        .unwrap();
    }
}

impl<S, From, To> SelectOneFragment<S> for OptionalToMany<From, To>
where
    S: QueryBuilder,
    To: Send + Sync + Collection<S>,
    To::Yeild: Send + Sync,
    From: Send + Sync + Collection<S>,
    From::Yeild: Send + Sync,
    for<'c> &'c str: ColumnIndex<S::Row>,
    for<'q> i64: Decode<'q, S>,
    i64: Type<S>,
{
    type Output = Option<SimpleOutput<To::Yeild>>;
    type Inner = Option<(i64, To::Yeild)>;

    fn on_select(&self, _: &mut Self::Inner, st: &mut SelectSt<S>) {
        st.join(join::left_join {
            foriegn_table: self.to.table_name().to_string(),
            foriegn_column: "id".to_string(),
            local_column: self.foriegn_key.to_string(),
        });
        st.select(col(&self.foriegn_key).table(self.from.table_name()));
        self.to.on_select(st);
    }

    fn from_row(&self, data: &mut Self::Inner, row: &S::Row) {
        let id: Option<i64> = row.get(self.foriegn_key.as_str());
        if let Some(id) = id {
            let value = self.to.from_row_scoped(row);
            *data = Some((id, value));
        }
    }

    async fn sub_op<'this>(&'this self, _: &'this mut Self::Inner, _: Pool<S>) {
        // no sub_op for optional_to_many
    }

    fn take(self, data: Self::Inner) -> Self::Output {
        data.map(|(id, attr)| SimpleOutput { id, attr })
    }
}

impl<S, F, T> DynamicLinkForRelation<S> for OptionalToMany<F, T>
where
    F: 'static,
    T: 'static,
    Self: Clone,
    Self: SelectOneFragment<S, Output: Serialize, Inner: 'static>,
    S: QueryBuilder,
{
    fn global_ident(&self) -> &'static str {
        "optional_to_many"
    }
    fn on_each_select_one_request(
        &self,
        input: serde_json::Value,
    ) -> Result<Box<dyn SelectOneJsonFragment<S>>, String> {
        if input.is_object().not() {
            return Err("many_to_many relation is only input is {}".to_string());
        }
        let this = self.clone();

        Ok(Box::new((this, Default::default())))
    }
}
