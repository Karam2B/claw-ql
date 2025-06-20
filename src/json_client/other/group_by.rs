use crate::json_client::{DynamicLink, JsonCollection, ReturnAsJsonMap, SelectOneJsonFragment};

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
    fn json_entry(&self) -> Vec<&'static str> {
        vec!["count"]
    }
    type SelectOneInput = Vec<String>;
    type SelectOne = ReturnAsJsonMap<CountDynamic>;
    fn on_select_one(
        &self,
        base_col: Arc<dyn JsonCollection<Sqlite>>,
        input: Self::SelectOneInput,
        ctx: Arc<AnySet>,
    ) -> Result<Option<Self::SelectOne>, String> {
        // let input = serde_json::from_value::<Vec<String>>(input).ok()?;
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

                return (
                    to.clone(),
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
                        inner: None,
                    },
                );
            })
            .collect::<Vec<_>>();

        return Ok(Some(ReturnAsJsonMap(s)));
    }
    type InsertOneInput = ();

    type InsertOne = ();

    fn on_insert_one(
        &self,
        _base_col: std::sync::Arc<dyn crate::json_client::JsonCollection<Sqlite>>,
        _input: Self::InsertOneInput,
        _ctx: std::sync::Arc<crate::any_set::AnySet>,
    ) -> Result<Option<Self::InsertOne>, String> {
        todo!()
    }

    type DeleteOneInput = ();

    type DeleteOne = ();

    fn on_delete_one(
        &self,
        _base_col: std::sync::Arc<dyn crate::json_client::JsonCollection<Sqlite>>,
        _input: Self::DeleteOneInput,
        _ctx: std::sync::Arc<crate::any_set::AnySet>,
    ) -> Result<Option<Self::DeleteOne>, String> {
        todo!()
    }

    type UpdateOneInput = ();

    type UpdateOne = ();

    fn on_update_one(
        &self,
        _base_col: std::sync::Arc<dyn crate::json_client::JsonCollection<Sqlite>>,
        _input: Self::UpdateOneInput,
        _ctx: std::sync::Arc<crate::any_set::AnySet>,
    ) -> Result<Option<Self::UpdateOne>, String> {
        todo!()
    }
}

#[derive(Serialize)]
pub struct CountDynamic {
    from_table_name: String,
    to_table_name: String,
    alias: String,
    junction: String,
    inner: Option<i64>,
}

// I can't access Count<F, T> in dynamic code because
// I don't have access to downcase::<T> here
//
// unlike Relation<F,T>
//
// Count is too "dynamic" so CountDynamic is there to solve this issue
impl SelectOneJsonFragment<Sqlite> for CountDynamic {
    fn on_select(&mut self, st: &mut SelectSt<Sqlite>) {
        let column_name_in_junction = format!("{}_id", self.from_table_name.to_case(Case::Snake));
        // let foriegn_table = self.to.table_name().to_string();
        let junction = format!("{}{}", self.to_table_name, self.from_table_name);
        st.select(format!(
            "COUNT({junction}.{column_name_in_junction}) AS {alias}",
            alias = self.alias
        ));
        st.join(join {
            foriegn_table: self.junction.clone(),
            foriegn_column: column_name_in_junction,
            local_column: "id".to_string(),
        });
        st.group_by(col("id").table(&self.from_table_name));
    }

    fn from_row(&mut self, row: &SqliteRow) {
        use sqlx::Row;
        *&mut self.inner = Some(row.get(self.alias.as_str()));
    }

    fn sub_op<'this>(
        &'this mut self,
        _pool: sqlx::Pool<Sqlite>,
    ) -> std::pin::Pin<Box<dyn Future<Output = ()> + Send + 'this>> {
        Box::pin(async {})
    }

    fn take(self: Box<Self>) -> serde_json::Value {
        serde_json::to_value(CountResult(self.inner.unwrap())).unwrap()
    }
}
impl SelectOneFragment<Sqlite> for CountDynamic {
    type Inner = Option<i64>;

    type Output = CountResult;

    fn on_select(&mut self, _data: &mut Self::Inner, st: &mut SelectSt<Sqlite>) {
        let column_name_in_junction = format!("{}_id", self.from_table_name.to_case(Case::Snake));
        // let foriegn_table = self.to.table_name().to_string();
        let junction = format!("{}{}", self.to_table_name, self.from_table_name);
        st.select(format!(
            "COUNT({junction}.{column_name_in_junction}) AS {alias}",
            alias = self.alias
        ));
        st.join(join {
            foriegn_table: self.junction.clone(),
            foriegn_column: column_name_in_junction,
            local_column: "id".to_string(),
        });
        st.group_by(col("id").table(&self.from_table_name));
    }

    fn from_row(&mut self, data: &mut Self::Inner, row: &SqliteRow) {
        use sqlx::Row;
        *data = Some(row.get(self.alias.as_str()));
    }

    fn sub_op<'this>(
        &'this mut self,
        _data: &'this mut Self::Inner,
        _pool: sqlx::Pool<Sqlite>,
    ) -> impl Future<Output = ()> + Send + use<'this> {
        async { /* no_op: count has no sub_op */ }
    }

    fn take(self, data: Self::Inner) -> Self::Output {
        CountResult(data.unwrap())
    }
}
