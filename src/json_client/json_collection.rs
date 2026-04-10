pub trait JsonCollection<S>: Send + Sync + 'static {
    fn clone(&self) -> Box<dyn JsonCollection<S>>;
    fn table_name(&self) -> &str;
    fn table_name_lowercase(&self) -> &str;
    fn members(&self) -> Vec<String>;
    fn on_select(&self, stmt: &mut SelectSt<S>)
    where
        S: QueryBuilder;
    fn on_insert(&self, this: Value, stmt: &mut InsertOneSt<S>) -> Result<(), FailedToParse>
    where
        S: sqlx::Database;
    fn on_update(&self, this: Value, stmt: &mut UpdateSt<S>) -> Result<(), FailedToParse>
    where
        S: QueryBuilder;
    fn from_row_noscope(&self, row: &S::Row) -> Value
    where
        S: Database;
    fn from_row_scoped(&self, row: &S::Row) -> Value
    where
        S: Database;
}

impl<S: 'static> Clone for Box<dyn JsonCollection<S>> {
    fn clone(&self) -> Self {
        JsonCollection::<S>::clone(&**self)
    }
}

impl<S: 'static> CollectionBasic for Box<dyn JsonCollection<S>> {
    fn table_name(&self) -> &str {
        JsonCollection::<S>::table_name(&**self)
    }

    fn table_name_lower_case(&self) -> &str {
        JsonCollection::<S>::table_name_lowercase(&**self)
    }

    fn members(&self) -> Vec<String> {
        JsonCollection::<S>::members(&**self)
    }
}

impl<S: 'static> CollectionHandler for Box<dyn JsonCollection<S>> {
    type LinkedData = Value;
}

impl<S, T> JsonCollection<S> for T
where
    T: Clone,
    S: QueryBuilder,
    T: Collection<S> + 'static,
    T::Data: Serialize + DeserializeOwned,
    T::Partial: DeserializeOwned,
{
    fn clone(&self) -> Box<dyn JsonCollection<S>> {
        Box::new(Clone::clone(self))
    }

    #[inline]
    fn on_select(&self, stmt: &mut SelectSt<S>)
    where
        S: QueryBuilder,
    {
        Collection::<S>::on_select(self, stmt)
    }

    #[inline]
    fn on_insert(&self, this: Value, stmt: &mut InsertOneSt<S>) -> Result<(), FailedToParse>
    where
        S: sqlx::Database,
    {
        let input = from_value::<T::Data>(this)?;
        Collection::<S>::on_insert(self, input, stmt);
        Ok(())
    }

    #[inline]
    fn on_update(&self, this: Value, stmt: &mut UpdateSt<S>) -> Result<(), FailedToParse>
    where
        S: QueryBuilder,
    {
        let input = from_value::<T::Partial>(this)?;
        Collection::<S>::on_update(self, input, stmt);
        Ok(())
    }

    #[inline]
    fn from_row_scoped(&self, row: &S::Row) -> serde_json::Value
    where
        S: Database,
    {
        let row = Collection::<S>::from_row_scoped(self, row);
        serde_json::to_value(row)
            .expect("data integrity bug indicate the bug is within `claw_ql` code")
    }
    #[inline]
    fn from_row_noscope(&self, row: &S::Row) -> Value
    where
        S: Database,
    {
        let row = Collection::<S>::from_row_noscope(self, row);
        serde_json::to_value(row)
            .expect("data integrity bug indicate the bug is within `claw_ql` code")
    }

    fn table_name(&self) -> &str {
        CollectionBasic::table_name(self)
    }

    fn table_name_lowercase(&self) -> &str {
        CollectionBasic::table_name_lower_case(self)
    }

    fn members(&self) -> Vec<String> {
        CollectionBasic::members(self)
    }
}

impl<S> JsonCollection<S> for DynamicCollection<S>
where
    for<'a> &'a str: ColumnIndex<S::Row>,
    S: QueryBuilder + Sync,
{
    fn clone(&self) -> Box<dyn JsonCollection<S>> {
        Box::new(Clone::clone(self))
    }
    fn table_name(&self) -> &str {
        todo!()
        // &self.name
    }

    fn members(&self) -> Vec<String> {
        todo!()
        // self.fields.iter().map(|e| e.name.to_string()).collect()
    }

    fn on_select(&self, stmt: &mut crate::prelude::stmt::SelectSt<S>)
    where
        S: crate::QueryBuilder,
    {
        for field in self.fields.iter() {
            stmt.select(
                crate::prelude::col(&field.name)
                    .table(&self.name)
                    .alias(&format!("{}_{}", self.table_name_lowercase(), field.name)),
            );
        }
    }

    fn on_insert(
        &self,
        this: serde_json::Value,
        stmt: &mut crate::prelude::stmt::InsertOneSt<S>,
    ) -> Result<(), FailedToParse>
    where
        S: sqlx::Database,
    {
        let this_obj = this.as_object().ok_or("failed to parse to object")?;
        for field in self.fields.iter() {
            field.type_info.on_insert(
                this_obj
                    .get(&field.name)
                    .cloned()
                    .ok_or(format!("object doesn't contain keys {}", field.name))?,
                stmt,
                &field.name,
            )?;
        }
        todo!()
    }

    fn on_update(
        &self,
        this: serde_json::Value,
        stmt: &mut crate::prelude::macro_derive_collection::UpdateSt<S>,
    ) -> Result<(), FailedToParse>
    where
        S: crate::QueryBuilder,
    {
        todo!()
    }

    fn from_row_noscope(&self, row: &<S>::Row) -> serde_json::Value
    where
        S: Database,
    {
        use sqlx::Row;
        panic!("rows{:?}", row.columns());
        for field in self.fields.iter() {
            let typei = &field.type_info;
            let ret = field.type_info.from_row_optional(&field.name, row);
        }
        todo!()
    }

    #[track_caller]
    fn from_row_scoped(&self, row: &<S>::Row) -> serde_json::Value
    where
        S: Database,
    {
        use sqlx::Row;
        let table_name = &self.name;
        let mut map = serde_json::Map::default();
        for field in self.fields.iter() {
            let name = &field.name;
            let typei = &field.type_info;
            let ret = field
                .type_info
                .from_row_optional(&format!("{}_{name}", table_name), row);
            let inserted = map.insert(field.name.clone(), ret);
            if inserted.is_some() {
                panic!("map should be empty")
            }
        }
        serde_json::to_value(map).unwrap()
    }

    fn table_name_lowercase(&self) -> &str {
        todo!()
    }
}
