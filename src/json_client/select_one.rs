use super::JsonClient;
use crate::{
    QueryBuilder,
    execute::Execute,
    operations::{LinkedOutput, select_one_op::SelectOneFragment},
    prelude::{col, stmt::SelectSt},
};
use serde::{Deserialize, Serialize};
use serde_json::{Map, Value};
use sqlx::{ColumnIndex, Database, Decode, Encode, Pool, prelude::Type};
use std::ops::Not;
use std::{collections::HashMap, pin::Pin};

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
pub struct SelectOneInput {
    pub collection: String,
    #[allow(unused)]
    #[serde(default)]
    pub filters: Map<String, Value>,
    #[serde(default)]
    pub links: Map<String, Value>,
}

impl<S> JsonClient<S>
where
    S: QueryBuilder<Output = <S as Database>::Arguments<'static>>,
    for<'c> &'c mut S::Connection: sqlx::Executor<'c, Database = S>,
    for<'e> i64: Encode<'e, S> + Type<S> + Decode<'e, S>,
    for<'e> &'e str: ColumnIndex<S::Row>,
{
    pub async fn select_one(&self, input: Value) -> Result<Value, String> {
        let input: SelectOneInput =
            serde_json::from_value(input).map_err(|e| format!("invalid input: {e:?}"))?;
        self.select_one_serialized(input).await
    }
    pub async fn select_one_serialized(&self, input: SelectOneInput) -> Result<Value, String> {
        let c = self
            .collections
            .get(&input.collection)
            .ok_or(format!("collection {} was not found", input.collection))?;

        let mut st = SelectSt::init(c.table_name());

        let mut link_errors = Vec::default();

        let mut links = self
            .links
            .iter()
            .filter_map(|e| {
                let name = e.1.json_entry();
                let input = input.links.get(*e.0)?.clone();
                let s = e.1.on_select_one(c.clone(), input, self.any_set.clone());

                match s {
                    Ok(Some(s)) => Some((name, s)),
                    Ok(None) => None,
                    Err(e) => {
                        link_errors.push(e);
                        None
                    }
                }
            })
            .collect::<Vec<_>>();

        if link_errors.is_empty().not() {
            return Err(format!("{link_errors:?}"));
        }

        #[rustfmt::skip]
        st.select(
            col("id").
            table(c.table_name()).
            alias("local_id")
        );

        c.on_select(&mut st);
        for link in links.iter_mut() {
            link.1.on_select(&mut st);
        }

        let res = st
            .fetch_one(&self.db, |r| {
                use sqlx::Row;
                let id: i64 = r.get("local_id");
                let attr = c.from_row_scoped(&r);

                for link in links.iter_mut() {
                    link.1.from_row(&r);
                }

                Ok(LinkedOutput {
                    id,
                    attr,
                    links: HashMap::new(),
                })
            })
            .await;

        let mut res = match res {
            Err(sqlx::Error::RowNotFound) => return Ok(serde_json::Value::Null),
            Err(err) => panic!("bug: {err}"),
            Ok(ok) => ok,
        };

        for link in links.iter_mut() {
            link.1.sub_op(self.db.clone()).await;
        }

        res.links = links.into_iter().map(|e| (e.0, e.1.take())).collect();

        Ok(serde_json::to_value(res).unwrap())
    }
}

pub trait SelectOneJsonFragment<S: QueryBuilder>: Send + Sync + 'static {
    fn on_select(&mut self, st: &mut SelectSt<S>);
    fn from_row(&mut self, row: &S::Row);
    fn sub_op<'this>(
        &'this mut self,
        pool: Pool<S>,
    ) -> Pin<Box<dyn Future<Output = ()> + Send + 'this>>;
    fn take(self: Box<Self>) -> serde_json::Value;
}

impl<S: QueryBuilder, T> SelectOneJsonFragment<S> for (T, T::Inner)
where
    T::Output: Serialize,
    T: SelectOneFragment<S> + 'static,
{
    #[inline]
    fn on_select(&mut self, st: &mut SelectSt<S>) {
        self.0.on_select(&mut self.1, st)
    }

    #[inline]
    fn from_row(&mut self, row: &<S>::Row) {
        self.0.from_row(&mut self.1, row)
    }

    #[inline]
    fn sub_op<'this>(
        &'this mut self,
        pool: Pool<S>,
    ) -> Pin<Box<dyn Future<Output = ()> + Send + 'this>> {
        Box::pin(async { self.0.sub_op(&mut self.1, pool).await })
    }

    #[inline]
    fn take(self: Box<Self>) -> serde_json::Value {
        let taken = self.0.take(self.1);
        serde_json::to_value(taken).unwrap()
    }
}

impl<S: QueryBuilder, T> SelectOneJsonFragment<S> for Box<T>
where
    T: ?Sized,
    T: SelectOneJsonFragment<S>,
{
    fn on_select(&mut self, st: &mut SelectSt<S>) {
        T::on_select(self, st)
    }

    fn from_row(&mut self, row: &<S>::Row) {
        T::from_row(self, row)
    }

    fn sub_op<'this>(
        &'this mut self,
        pool: Pool<S>,
    ) -> Pin<Box<dyn Future<Output = ()> + Send + 'this>> {
        T::sub_op(self, pool)
    }

    fn take(self: Box<Self>) -> serde_json::Value {
        T::take(*self)
    }
}

impl<S: QueryBuilder> SelectOneJsonFragment<S> for () {
    fn on_select(&mut self, _st: &mut SelectSt<S>) {
        ()
    }

    fn from_row(&mut self, _row: &<S>::Row) {}

    fn sub_op<'this>(
        &'this mut self,
        _pool: Pool<S>,
    ) -> Pin<Box<dyn Future<Output = ()> + Send + 'this>> {
        Box::pin(async {})
    }

    fn take(self: Box<Self>) -> serde_json::Value {
        Value::Null
    }
}
