use super::LinkData;
use super::relation::Relation;
use super::relation_many_to_many::ManyToMany;
use crate::QueryBuilder;
use crate::collections::{Collection, OnMigrate};
use crate::prelude::col;
use crate::prelude::macro_relation::OptionalToMany;
use crate::{
    collections::CollectionBasic,
    operations::select_one_op::SelectOneFragment,
    prelude::{join::join, stmt::SelectSt},
};
use convert_case::{Case, Casing};
use serde::Serialize;
use sqlx::{ColumnIndex, Executor};
use sqlx::{Sqlite, sqlite::SqliteRow};

#[allow(non_camel_case_types)]
pub struct count<T>(pub T);

pub struct CountSpec<From, To> {
    from: From,
    to: To,
    alias: String,
    junction: String,
}

trait CountingSupportedIn {}
impl<T0, T1> CountingSupportedIn for ManyToMany<T0, T1> {}
impl<From, To> CountingSupportedIn for OptionalToMany<From, To> {}

impl<From, To> LinkData<From> for count<To>
where
    From: CollectionBasic,
    To: CollectionBasic,
    Relation<From, To>: LinkData<From, Spec: CountingSupportedIn>,
{
    type Spec = CountSpec<From, To>;
    fn spec(self, from: From) -> Self::Spec
    where
        Self: Sized,
    {
        let junction = format!("{}{}", self.0.table_name(), from.table_name());
        CountSpec {
            from,
            alias: format!("count_{}_s", self.0.table_name().to_case(Case::Snake)),
            to: self.0,
            junction,
        }
    }
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize)]
pub struct CountResult(pub i64);

impl<From, To> SelectOneFragment<Sqlite> for CountSpec<From, To>
where
    From: Send + Sync + Collection<Sqlite>,
    To: Send + Sync + Collection<Sqlite>,
    // sqlx gasim
    for<'s> &'s str: ColumnIndex<SqliteRow>,
{
    type Inner = Option<i64>;

    type Output = CountResult;

    fn on_select(&mut self, _data: &mut Self::Inner, st: &mut SelectSt<Sqlite>) {
        let column_name_in_junction = format!("{}_id", self.from.table_name().to_case(Case::Snake));
        let junction = format!("{}{}", self.to.table_name(), self.from.table_name());
        st.select(format!(
            "COUNT({junction}.{column_name_in_junction}) AS {alias}",
            alias = self.alias
        ));
        st.join(join {
            foriegn_table: self.junction.clone(),
            foriegn_column: column_name_in_junction,
            local_column: "id".to_string(),
        });
        st.group_by(col("id").table(&self.from.table_name()));
    }

    fn from_row(&mut self, data: &mut Self::Inner, row: &SqliteRow) {
        use sqlx::Row;
        *data = Some(row.get(self.alias.as_str()));
    }

    fn sub_op<'this>(
        &'this mut self,
        _data: &'this mut Self::Inner,
        _pool: sqlx::Pool<Sqlite>,
    ) -> impl Future<Output = ()> + Send + use<'this, From, To> {
        async { /* no_op: count has no sub_op */ }
    }

    fn take(self, data: Self::Inner) -> Self::Output {
        CountResult(data.unwrap())
    }
}

// no op
impl<S> OnMigrate<S> for count<()> {
    async fn custom_migration<'e>(&self, _: impl for<'q> Executor<'q, Database = S> + Clone)
    where
        S: QueryBuilder,
    {
        // no-op count is on-request only
    }
}

impl count<()> {
    pub fn dynamic_link() -> count<()> {
        count(())
    }
}
