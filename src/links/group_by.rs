use std::any::{Any, TypeId};
use std::collections::HashMap;

use crate::dynamic_client::json_client::{JsonCollection, JsonLink};
use crate::QueryBuilder;
use crate::collections::{Collection, OnMigrate};
use crate::prelude::col;
use crate::prelude::macro_relation::OptionalToMany;
use crate::schema::AnySet;
use crate::schema::json_client::JsonCollection;
use crate::verbatim::verbatim;
use crate::{
    collections::CollectionBasic,
    operations::{LinkData, Relation, select_one::SelectOneFragment},
    prelude::{join::join, stmt::SelectSt},
};
use sqlx::{ColumnIndex, Executor};
use sqlx::{Sqlite, sqlite::SqliteRow};

use super::relation_many_to_many::ManyToMany;

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

pub struct DynamicCount;

// no op
impl<S> OnMigrate<S> for DynamicCount {
    async fn custom_migration<'e>(&self, _: impl for<'q> Executor<'q, Database = S> + Clone)
    where
        S: QueryBuilder,
    {
    }
}

impl count<()> {
    pub fn dynamic_link() -> DynamicCount {
        DynamicCount
    }
}

pub trait DynamicLink<S> {
    fn on_finish(&self, link_ctx: &AnySet) -> Result<(), String>;
}
pub trait DynamicLinkAssociatedEntry {
    fn register_entry_in_context(&self) -> Option<Self::Entry>;
    type Entry: Any;
}

pub struct DynamicManyToMany {}

impl<S> DynamicLink<S> for DynamicCount {
    fn on_finish(&self, link_ctx: &AnySet) -> Result<(), String> {
        let dynamic = link_ctx.get::<DynamicManyToMany>().ok_or(
            "cant relation of many_to_many, make sure to register 'count' after all many_to_many_s",
        )?;

        Ok(())
    }
}

impl DynamicLinkAssociatedEntry for DynamicCount {
    type Entry = ();
    fn register_entry_in_context(&self) -> Option<Self::Entry> {
        None
    }
}

impl<S> JsonLink<S> for DynamicCount {
    fn on_each_json_request(&self, link_ctx: AnySet, base_col: &dyn JsonCollection<S>) {}
}

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
            alias: format!("count_{}_s", self.0.table_name().to_lowercase()),
            to: self.0,
            junction,
        }
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
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

    fn on_select(&self, data: &mut Self::Inner, st: &mut SelectSt<Sqlite>) {
        let column_name_in_junction = format!("{}_id", self.from.table_name().to_lowercase());
        let foriegn_table = self.to.table_name().to_string();
        let junction = format!("{}{}", self.to.table_name(), self.from.table_name());
        st.select(verbatim(format!(
            "COUNT({junction}.{column_name_in_junction}) AS {alias}",
            alias = self.alias
        )));
        st.join(join {
            foriegn_table: self.junction.clone(),
            foriegn_column: column_name_in_junction,
            local_column: "id".to_string(),
        });
        st.group_by(col("id").table(&self.from.table_name()));
    }

    fn from_row(&self, data: &mut Self::Inner, row: &SqliteRow) {
        use sqlx::Row;
        *data = Some(row.get(self.alias.as_str()));
    }

    fn sub_op<'this>(
        &'this self,
        data: &'this mut Self::Inner,
        pool: sqlx::Pool<Sqlite>,
    ) -> impl Future<Output = ()> + Send + use<'this, From, To> {
        async { /* no op */ }
    }

    fn take(self, data: Self::Inner) -> Self::Output {
        CountResult(data.unwrap())
    }
}
