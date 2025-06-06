use std::collections::HashMap;
use std::ops::Not;

use crate::QueryBuilder;
use crate::any_set::AnySet;
use crate::collections::{Collection, OnMigrate};
use crate::dynamic_client::json_client::{JsonCollection, SelectOneJsonFragment};
use crate::links::relation::RelationEntries;
use crate::prelude::col;
use crate::prelude::macro_relation::OptionalToMany;
use crate::verbatim::verbatim;
use crate::{
    collections::CollectionBasic,
    operations::select_one::SelectOneFragment,
    prelude::{join::join, stmt::SelectSt},
};
use convert_case::{Case, Casing};
use serde::Serialize;
use sqlx::{ColumnIndex, Executor};
use sqlx::{Sqlite, sqlite::SqliteRow};

use super::relation::Relation;
use super::relation_many_to_many::ManyToMany;
use super::{DynamicLink, LinkData};

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

    fn on_select(&self, data: &mut Self::Inner, st: &mut SelectSt<Sqlite>) {
        let column_name_in_junction = format!("{}_id", self.from.table_name().to_case(Case::Snake));
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
        _data: &'this mut Self::Inner,
        _pool: sqlx::Pool<Sqlite>,
    ) -> impl Future<Output = ()> + Send + use<'this, From, To> {
        async { /* no op */ }
    }

    fn take(self, data: Self::Inner) -> Self::Output {
        CountResult(data.unwrap())
    }
}

struct CountDynamic {
    from_table_name: String,
    to_table_name: String,
    alias: String,
    junction: String,
}

impl SelectOneFragment<Sqlite> for CountDynamic {
    type Inner = Option<i64>;

    type Output = CountResult;

    fn on_select(&self, data: &mut Self::Inner, st: &mut SelectSt<Sqlite>) {
        let column_name_in_junction = format!("{}_id", self.from_table_name.to_case(Case::Snake));
        let foriegn_table = self.to.table_name().to_string();
        let junction = format!("{}{}", self.to_table_name, self.from_table_name);
        st.select(verbatim(format!(
            "COUNT({junction}.{column_name_in_junction}) AS {alias}",
            alias = self.alias
        )));
        st.join(join {
            foriegn_table: self.junction,
            foriegn_column: column_name_in_junction,
            local_column: "id".to_string(),
        });
        st.group_by(col("id").table(&self.from_table_name));
    }

    fn from_row(&self, data: &mut Self::Inner, row: &SqliteRow) {
        todo!()
    }

    fn sub_op<'this>(
        &'this self,
        data: &'this mut Self::Inner,
        pool: sqlx::Pool<Sqlite>,
    ) -> impl Future<Output = ()> + Send + use<'this> {
        async { /* no op */ }
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

impl DynamicLink<Sqlite> for count<()> {
    fn on_finish(&self, build_ctx: &AnySet) -> Result<(), String> {
        let dynamic = build_ctx.get::<RelationEntries>().ok_or(
            "count: there was no relation added to the client, are you sure this is not a mistake",
        )?;
        if dynamic.entries.iter().any(|e| e.ty == "many_to_many").not() {
            Err("count: there was no many_to_many relation, is this an error")?;
        }

        Ok(())
    }
    type Entry = ();
    fn init_entry() -> Self::Entry {}
    fn on_register(&self, _entry: &mut Self::Entry) {}
    fn json_entry() -> &'static str {
        "count"
    }
    fn on_each_json_request(
        &self,
        base_col: &dyn JsonCollection<Sqlite>,
        input: serde_json::Value,
        ctx: &AnySet,
    ) -> Option<Result<Box<dyn SelectOneJsonFragment<Sqlite>>, String>> {
        let input = serde_json::from_value::<Vec<String>>(input).ok()?;
        let base = base_col.table_name().to_case(Case::Snake);

        let rels = ctx
            .get::<RelationEntries>()
            .unwrap()
            .entries
            .iter()
            .filter(|e| e.from == base)
            .filter(|e| e.ty == "many_to_many");

        let mut error_collector = Vec::default();

        let s = input
            .into_iter()
            .map(|to| {
                // let rel = rels.clone().find(|rel rel.to == to);

                if rels.clone().any(|rel| rel.to == to).not() {
                    error_collector.push(format!(
                        "{base} is not related to {to} with many_to_many relation",
                    ))
                }

                let boxed = Box::new((
                    CountDynamic {
                        from_table_name: base_col.table_name().to_string(),
                        to_table_name: to.to_case(Case::Camel),
                        alias: format!("count_{}_s", to),
                        // in ManyToMany___Inverse* these are reversed
                        junction: format!(
                            "{first}{second}",
                            first = base_col.table_name().to_string(),
                            second = to
                        ),
                    },
                    Default::default(),
                )) as Box<dyn SelectOneJsonFragment<Sqlite>>;

                return (to, boxed);
            })
            .collect::<Vec<_>>();

        return Some(Ok(Box::new(s)));
    }
}
