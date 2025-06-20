use super::JsonClient;
use crate::{
    QueryBuilder,
    execute::Execute,
    json_client::{from_map, map_is_empty},
    operations::{LinkedOutput, update_one_op::UpdateOneFragment},
    statements::update_st::UpdateSt,
};
use serde::{Deserialize, Serialize};
use serde_json::{Map, Value, from_value};
use sqlx::{ColumnIndex, Database, Decode, Encode, Executor, Pool, prelude::Type};
use std::{collections::HashMap, ops::Not, pin::Pin};

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
pub struct UpdateOneInput {
    pub collection: String,
    pub data: Value,
    #[serde(default)]
    pub links: Map<String, Value>,
    pub filters: Map<String, Value>,
}

impl<S> JsonClient<S>
where
    S: QueryBuilder<Output = <S as Database>::Arguments<'static>>,
    for<'c> &'c mut S::Connection: sqlx::Executor<'c, Database = S>,
    for<'e> i64: Encode<'e, S> + Type<S> + Decode<'e, S>,
    for<'e> &'e str: ColumnIndex<S::Row>,
{
    pub async fn update_one(&self, input: Value) -> Result<Value, String> {
        let input: UpdateOneInput =
            from_value(input).map_err(|e| format!("invalid input: {e:?}"))?;
        self.update_one_serialized(input).await
    }

    #[inline]
    pub async fn update_one_serialized(&self, mut input: UpdateOneInput) -> Result<Value, String> {
        use sqlx::Row;

        let c = self
            .collections
            .get(&input.collection)
            .ok_or(format!("collection {} was not found", input.collection))?
            .clone();

        let mut st = UpdateSt::init(c.table_name().to_string());

        c.on_update(input.data, &mut st)
            .map_err(|e| format!("invalid {}: {e:?}", c.table_name()))?;

        (|| {
            // self.filters.on_update(&handler, &mut st);
            if true {
                panic!("nice todo")
            }
        })();

        let mut links = {
            let mut link_errors = Vec::default();
            let links = self
                .links
                .iter()
                .filter_map(|e| {
                    let name = e.1.json_entry();
                    let input = from_map(&mut input.links, e.0)?;
                    let s = e.1.on_update_one(c.table_name().to_string(), input, &());
                    // let s = e.1.on_update_one(c.clone(), input, self.any_set.clone());

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
            if map_is_empty(&mut input.links).not() {
                return Err("unused input")?;
            }
            links
        };

        for link in links.iter_mut() {
            link.1.on_update(&mut st);
        }

        let mut s: Vec<String> = c.members();

        for link in links.iter_mut() {
            s.extend(link.1.returning());
        }

        s.push(String::from("id"));

        let mut res = st
            .returning(s)
            .fetch_optional(&self.db, |r| {
                let id: i64 = r.get("id");
                let attr = c.from_row_noscope(&r);
                for link in links.iter_mut() {
                    link.1.from_row(&r);
                }
                Ok(LinkedOutput {
                    id,
                    attr,
                    links: HashMap::new(),
                })
            })
            .await
            .unwrap()
            .ok_or("update performed no action")?;

        for link in links.iter_mut() {
            link.1.second_sub_op(self.db.clone()).await;
        }

        res.links = links.into_iter().map(|e| (e.0, e.1.take())).collect();

        return Ok(serde_json::to_value(res).unwrap());
    }
}

pub trait UpdateOneJsonFragment<S: QueryBuilder>: Send + Sync + 'static {
    fn on_update(&mut self, st: &mut UpdateSt<S>);
    fn from_row(&mut self, row: &S::Row);
    fn returning(&mut self) -> Vec<String>;
    fn second_sub_op<'this>(
        &'this mut self,
        pool: Pool<S>,
    ) -> Pin<Box<dyn Future<Output = ()> + Send + 'this>>;
    fn take(self: Box<Self>) -> serde_json::Value;
}

impl<S: QueryBuilder> UpdateOneJsonFragment<S> for () {
    fn on_update(&mut self, _st: &mut UpdateSt<S>) {}

    fn returning(&mut self) -> Vec<String> {
        vec![]
    }

    fn from_row(&mut self, _row: &<S>::Row) {}

    fn second_sub_op<'this>(
        &'this mut self,
        _pool: Pool<S>,
    ) -> Pin<Box<dyn Future<Output = ()> + Send + 'this>> {
        Box::pin(async {})
    }

    fn take(self: Box<Self>) -> serde_json::Value {
        Value::Null
    }
}

impl<S: QueryBuilder, T> UpdateOneJsonFragment<S> for (T, T::Inner)
where
    T: 'static,
    T::Output: Serialize,
    T: UpdateOneFragment<S>,
    for<'c> &'c mut S::Connection: Executor<'c, Database = S>,
{
    fn returning(&mut self) -> Vec<String> {
        UpdateOneFragment::returning(&mut self.0)
    }
    fn on_update(&mut self, st: &mut UpdateSt<S>) {
        self.0.on_update(&mut self.1, st)
    }

    #[inline]
    fn from_row(&mut self, row: &<S>::Row) {
        self.0.from_row(&mut self.1, row)
    }

    #[inline]
    fn second_sub_op<'this>(
        &'this mut self,
        pool: Pool<S>,
    ) -> Pin<Box<dyn Future<Output = ()> + Send + 'this>> {
        Box::pin(async move { self.0.second_sub_op(&mut self.1, &pool).await })
    }

    #[inline]
    fn take(self: Box<Self>) -> serde_json::Value {
        let taken = self.0.take(self.1);
        serde_json::to_value(taken).unwrap()
    }
}
