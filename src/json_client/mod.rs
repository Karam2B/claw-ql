pub mod builder_pattern;
use super::builder_pattern::BuilderPattern;
use crate::{
    QueryBuilder,
    any_set::AnySet,
    collections::Collection,
    execute::Execute,
    links::DynamicLinkTraitObject,
    operations::{LinkedOutput, select_one_op::SelectOneFragment},
    prelude::{col, stmt::SelectSt},
};
use builder_pattern::to_json_client;
use serde::{Deserialize, Serialize};
use serde_json::{Map, Value};
use sqlx::{ColumnIndex, Database, Decode, Encode, Pool, prelude::Type};
use std::ops::Not;
use std::{collections::HashMap, pin::Pin, sync::Arc};

pub struct JsonClient<S: Database> {
    pub(crate) collections: HashMap<String, Arc<dyn JsonCollection<S>>>,
    pub(crate) links: HashMap<&'static str, Arc<dyn DynamicLinkTraitObject<S>>>,
    pub(crate) any_set: AnySet,
    pub(crate) db: Pool<S>,
}

impl<S> JsonClient<S>
where
    S: Database,
{
    pub fn init(db: Pool<S>) -> BuilderPattern<(to_json_client<S>,), (), (), ()> {
        BuilderPattern::default().build_mode(to_json_client(db))
    }
}

pub trait JsonCollection<S>: Send + Sync + 'static {
    fn on_select(&self, stmt: &mut SelectSt<S>)
    where
        S: QueryBuilder;
    fn from_row_scoped(&self, row: &S::Row) -> serde_json::Value
    where
        S: Database;
    fn table_name(&self) -> &'static str;
}

impl<S, T> JsonCollection<S> for T
where
    S: QueryBuilder,
    T: Collection<S> + 'static,
    T::Data: Serialize,
{
    #[inline]
    fn on_select(&self, stmt: &mut SelectSt<S>)
    where
        S: QueryBuilder,
    {
        self.on_select(stmt)
    }
    #[inline]
    fn table_name(&self) -> &'static str {
        self.table_name()
    }
    #[inline]
    fn from_row_scoped(&self, row: &S::Row) -> serde_json::Value
    where
        S: Database,
    {
        let row = <Self as Collection<S>>::from_row_scoped(self, row);
        serde_json::to_value(row).unwrap()
    }
}

pub trait SelectOneJsonFragment<S: QueryBuilder>: Send + Sync {
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
    T: SelectOneFragment<S>,
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

// a common pattern is you have array of fragments and you
// want to build them as a map
impl<S: QueryBuilder> SelectOneJsonFragment<S>
    for Vec<(String, Box<dyn SelectOneJsonFragment<S>>)>
{
    fn on_select(&mut self, st: &mut SelectSt<S>) {
        self.iter_mut().for_each(|e| e.1.on_select(st))
    }

    fn from_row(&mut self, row: &<S>::Row) {
        self.iter_mut().for_each(|e| e.1.from_row(row))
    }

    fn sub_op<'this>(
        &'this mut self,
        pool: Pool<S>,
    ) -> Pin<Box<dyn Future<Output = ()> + Send + 'this>> {
        Box::pin(async move {
            for item in self.iter_mut() {
                item.1.sub_op(pool.clone()).await
            }
        })
    }

    fn take(self: Box<Self>) -> serde_json::Value {
        let mut map = serde_json::Map::new();
        self.into_iter().for_each(|e| {
            map.insert(e.0, e.1.take());
        });
        map.into()
    }
}

impl<S> JsonClient<S>
where
    S: QueryBuilder<Output = <S as Database>::Arguments<'static>>,
    for<'c> &'c mut S::Connection: sqlx::Executor<'c, Database = S>,
    for<'e> i64: Encode<'e, S> + Type<S> + Decode<'e, S>,
    for<'e> &'e str: ColumnIndex<S::Row>,
{
    pub async fn select_one(&self, input: Value) -> Result<Value, String> {
        #[derive(Deserialize)]
        #[serde(deny_unknown_fields)]
        struct Input {
            pub collection: String,
            #[allow(unused)]
            #[serde(default)]
            pub filters: Map<String, Value>,
            #[serde(default)]
            pub links: Map<String, Value>,
        }

        let input: Input =
            serde_json::from_value(input).map_err(|e| format!("invalid input: {e:?}"))?;

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
                let s = e.1.on_each_json_request(c.as_ref(), input, &self.any_set);

                match s {
                    Some(Ok(s)) => Some((name, s)),
                    None => None,
                    Some(Err(e)) => {
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
