use super::JsonClient;
use crate::{
    Accept, QueryBuilder,
    operations::{LinkedOutput, delete_one_op::DeleteOneFragment},
    statements::delete_st::DeleteSt,
};
use serde::{Deserialize, Serialize};
use serde_json::{Map, Value, from_value};
use sqlx::{ColumnIndex, Database, Decode, Encode, Executor, Pool, prelude::Type};
use std::{collections::HashMap, ops::Not, pin::Pin};

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
pub struct DeleteOneInput {
    pub collection: String,
    #[serde(default)]
    pub retrieve: Map<String, Value>,
    pub id: i64,
}

impl<S> JsonClient<S>
where
    S: QueryBuilder<Output = <S as Database>::Arguments<'static>>,
    for<'c> &'c mut S::Connection: sqlx::Executor<'c, Database = S>,
    S: Accept<i64>,
    for<'e> i64: Encode<'e, S> + Type<S> + Decode<'e, S>,
    for<'e> &'e str: ColumnIndex<S::Row>,
{
    pub async fn delete_one(&self, input: Value) -> Result<Value, String> {
        let input: DeleteOneInput =
            from_value(input).map_err(|e| format!("invalid input: {e:?}"))?;
        self.delete_one_serialized(input).await
    }

    pub async fn delete_one_serialized(&self, input: DeleteOneInput) -> Result<Value, String> {
        use crate::execute::Execute;
        use sqlx::Row;

        let handler = self
            .collections
            .get(&input.collection)
            .ok_or(format!("collection {} was not found", input.collection))?
            .clone();

        let st = DeleteSt::init_where_id_eq(handler.table_name().to_string(), input.id);

        let mut links = {
            let mut link_errors = Vec::default();
            let links = self
                .links
                .iter()
                .filter_map(|e| {
                    let name = e.1.json_entry();
                    let input = input.retrieve.get(*e.0)?.clone();
                    let s =
                        e.1.on_delete_one(handler.clone(), input, self.any_set.clone());

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
            };
            links
        };

        for link in links.iter_mut() {
            link.1.first_sub_op(self.db.clone(), input.id).await;
        }

        let mut s: Vec<String> = handler.members();

        for link in links.iter_mut() {
            s.extend(link.1.returning());
        }

        s.push(String::from("id"));

        let mut res = st
            .returning(s)
            .fetch_optional(&self.db, |r| {
                let id: i64 = r.get("id");
                let attr = handler.from_row_noscope(&r);
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

        res.links = links.into_iter().map(|e| (e.0, e.1.take())).collect();

        return Ok(serde_json::to_value(res).unwrap());
    }
}

pub trait DeleteOneJsonFragment<S: QueryBuilder>: Send + Sync + 'static {
    fn first_sub_op<'this>(
        &'this mut self,
        pool: Pool<S>,
        id: i64,
    ) -> Pin<Box<dyn Future<Output = ()> + Send + 'this>>;
    fn returning(&self) -> Vec<String>;
    fn from_row(&mut self, row: &S::Row);
    fn take(self: Box<Self>) -> serde_json::Value;
}

impl<S: QueryBuilder> DeleteOneJsonFragment<S> for () {
    fn returning(&self) -> Vec<String> {
        vec![]
    }

    fn from_row(&mut self, _row: &<S>::Row) {}

    fn first_sub_op<'this>(
        &'this mut self,
        _pool: Pool<S>,
        _id: i64
    ) -> Pin<Box<dyn Future<Output = ()> + Send + 'this>> {
        Box::pin(async {})
    }

    fn take(self: Box<Self>) -> serde_json::Value {
        Value::Null
    }
}

impl<S: QueryBuilder, T> DeleteOneJsonFragment<S> for (T, T::Inner)
where
    T: 'static,
    T::Output: Serialize,
    T: DeleteOneFragment<S>,
    for<'c> &'c mut S::Connection: Executor<'c, Database = S>,
{
    #[inline]
    fn first_sub_op<'this>(
        &'this mut self,
        pool: Pool<S>,
        id: i64,
    ) -> Pin<Box<dyn Future<Output = ()> + Send + 'this>> {
        Box::pin(async move { self.0.first_sup_op(&mut self.1, &pool, id).await })
    }

    #[inline]
    fn returning(&self) -> Vec<String> {
        DeleteOneFragment::<S>::returning(&self.0)
    }

    #[inline]
    fn from_row(&mut self, row: &<S>::Row) {
        self.0.from_row(&mut self.1, row)
    }

    #[inline]
    fn take(self: Box<Self>) -> serde_json::Value {
        let taken = self.0.take(self.1);
        serde_json::to_value(taken).unwrap()
    }
}
