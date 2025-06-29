use super::JsonClient;
use crate::{
    QueryBuilder,
    execute::Execute,
    json_client::{RuntimeResult, from_map, map_is_empty},
    operations::{CollectionOutput, LinkedOutput, select_one_op::SelectOneFragment},
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
    #[inline]
    pub async fn select_one(&self, input: Value) -> Result<Value, String> {
        let input: SelectOneInput =
            serde_json::from_value(input).map_err(|e| format!("invalid input: {e:?}"))?;
        Ok(serde_json::to_value(self.select_one_serialized(input).await?).unwrap())
    }

    pub async fn select_one_serialized(
        &self,
        mut input: SelectOneInput,
    ) -> Result<LinkedOutput<Value, Map<String, Value>>, String> {
        let c = self
            .collections
            .get(&input.collection)
            .ok_or(format!("collection {} was not found", input.collection))?;

        let mut st = SelectSt::init(c.table_name());

        let mut links = {
            let mut link_errors = Vec::default();
            let links = self
                .links
                .iter()
                .filter_map(|e| {
                    let name = e.1.json_selector();
                    let input = from_map(&mut input.links, &e.0.body)?;
                    let s = e.1.on_select_one(c.table_name().to_string(), input);

                    match s {
                        Ok(s) => Some((name, s)),
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

            if map_is_empty(&mut input.links).not() {
                return Err("unused input")?;
            }
            links
        };

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

                Ok(CollectionOutput { id, attr })
            })
            .await;

        let mut res = match res {
            Err(sqlx::Error::RowNotFound) => return Err("nothing found".to_string()),
            Err(err) => panic!("bug: {err}"),
            Ok(ok) => ok,
        };

        for link in links.iter_mut() {
            link.1.sub_op(self.db.clone()).await;
        }

        fn build_back_as_map(input: HashMap<Vec<&'static str>, ()>) {}

        let links = {
            let mut map = Map::new();
            for (mut keys, value) in links.into_iter().map(|e| (e.0, e.1.take())) {
                let last = keys.body.pop().unwrap();
                let mut reff = None;
                keys.body.reverse();
                while let Some(next) = keys.body.pop() {
                    map.insert(next.to_string(), Value::Object(Default::default()));
                    let s = map.get_mut(next).unwrap().as_object_mut().unwrap();
                    reff = Some(s)
                }
                if let Some(reff) = reff {
                    reff.insert(last.to_string(), value);
                } else {
                    map.insert(last.to_string(), value);
                }
            }
            map
        };
        let res = LinkedOutput {
            id: res.id,
            attr: res.attr,
            links,
        };

        Ok(res)
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
