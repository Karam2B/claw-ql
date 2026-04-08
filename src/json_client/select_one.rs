use super::JsonClient;
use crate::{
    QueryBuilder,
    collections::{FailedToParseBody, FilterError, FilterIsNotApplicableForCollection},
    execute::Execute,
    json_client::{RuntimeResult, axum_router_mod::HttpError, from_map, map_is_empty},
    operations::{CollectionOutput, LinkedOutput, select_one_op::SelectOneFragment},
    prelude::{col, stmt::SelectSt},
};
use serde::{Deserialize, Serialize};
use serde_json::{Map, Value};
use sqlx::{ColumnIndex, Database, Decode, Encode, Pool, prelude::Type};
use std::{any::Any, collections::HashMap, pin::Pin};
use std::{ops::Not, sync::Arc};

#[derive(Debug)]
#[cfg_attr(feature = "http", derive(serde::Serialize))]
pub struct TypeidIsNotRegistered {
    pub requested: String,
    pub to_impl: String,
}

impl HttpError for TypeidIsNotRegistered {
    fn status_code(&self) -> hyper::StatusCode {
        hyper::StatusCode::BAD_REQUEST
    }
}

#[derive(Debug)]
#[cfg_attr(feature = "http", derive(serde::Serialize))]
pub enum SelectOneError {
    TypeidIsNotRegistered(TypeidIsNotRegistered),
    FailedToParseBody(FailedToParseBody),
    FilterIsNotApplicableForCollection(FilterIsNotApplicableForCollection),
    Other(String),
}

impl From<FilterError> for SelectOneError {
    fn from(value: FilterError) -> Self {
        match value {
            FilterError::FailedToParseBody(failed_to_parse_body) => {
                Self::FailedToParseBody(failed_to_parse_body)
            }
            FilterError::FilterIsNotApplicableForCollection(
                filter_is_not_applicable_for_collection,
            ) => Self::FilterIsNotApplicableForCollection(filter_is_not_applicable_for_collection),
            _ => Self::Other("unenumerated filter error".to_string()),
        }
    }
}

// impl From<FilterIsNotApplicableForCollection> for SelectOneError {
//     fn from(value: FilterIsNotApplicableForCollection) -> Self {
//         SelectOneError::FilterIsNotApplicableForCollection(value)
//     }
// }
// impl From<FailedToParseBody> for SelectOneError {
//     fn from(value: FailedToParseBody) -> Self {
//         SelectOneError::FailedToParseBody(value)
//     }
// }

impl From<String> for SelectOneError {
    fn from(value: String) -> Self {
        Self::Other(value)
    }
}
impl From<&str> for SelectOneError {
    fn from(value: &str) -> Self {
        Self::Other(value.to_string())
    }
}

impl From<TypeidIsNotRegistered> for SelectOneError {
    fn from(value: TypeidIsNotRegistered) -> Self {
        Self::TypeidIsNotRegistered(value)
    }
}

impl HttpError for SelectOneError {
    fn status_code(&self) -> hyper::StatusCode {
        match self {
            SelectOneError::TypeidIsNotRegistered(typeid_is_not_registered) => {
                typeid_is_not_registered.status_code()
            }
            SelectOneError::FailedToParseBody(s) => hyper::StatusCode::BAD_REQUEST,
            SelectOneError::FilterIsNotApplicableForCollection(s) => hyper::StatusCode::BAD_REQUEST,
            // other is used temporarly for now
            Self::Other(_) => hyper::StatusCode::INTERNAL_SERVER_ERROR,
        }
    }
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
pub struct SelectOneInput {
    pub collection: String,
    #[allow(unused)]
    #[serde(default)]
    pub filters: Vec<WithTypeid>,
    #[serde(default)]
    pub links: Map<String, Value>,
}

#[derive(Deserialize, Serialize)]
pub struct WithTypeid {
    #[serde(rename = "$typeid")]
    pub typeid: String,
    #[serde(flatten)]
    pub rest: Value,
}

#[cfg(feature = "sdf")]
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

        let output = self
            .select_one_serialized_2(input)
            .await
            .map_err(|e| format!("{e:?}"))?;

        Ok(serde_json::to_value(output).expect("should not go through any error, no?"))
    }

    pub async fn select_one_serialized_2(
        &self,
        mut input: SelectOneInput,
    ) -> Result<LinkedOutput<Value, Map<String, Value>>, SelectOneError> {
        let c = self
            .collections
            .get(&input.collection)
            .ok_or(format!("collection {} was not found", input.collection))?;

        let mut st = SelectSt::init(c.table_name());

        for WithTypeid { typeid, rest } in input.filters {
            let filter = if let Some(found) = self.filter_extentions.get(&typeid) {
                found
            } else {
                let existing_filters = self.filter_extentions.keys().collect::<Vec<_>>();
                // self.debugger_error()

                return Err(SelectOneError::TypeidIsNotRegistered(
                    TypeidIsNotRegistered {
                        requested: typeid,
                        to_impl: "filter".to_string(),
                    },
                ));
            };

            filter.on_select(rest, &**c, &mut st)?;
            // st.where_("");
        }

        // for WithTypeid { typeid, rest } in input.links {
        //     let link = if let Some(found) = self.links.get(&typeid) {
        //         let input = rest;
        //         found.on_select_one(base_col, input, client)
        //     } else {
        //         return Err(SelectOneError::TypeidIsNotRegistered(
        //             TypeidIsNotRegistered {
        //                 requested: typeid,
        //                 to_impl: "link_data".to_string(),
        //             },
        //         ));
        //     };
        // }

        let mut links = {
            let mut link_errors = Vec::default();
            let links = self
                .links
                .iter()
                .filter_map(|e| {
                    let name = e.1.json_selector();
                    let input = from_map(&mut input.links, &e.0.body)?;
                    // TODO: it is possible to have bugs where you drop Arc<Self> (mutate it) and then use the next Box
                    let s = e.1.on_select_one(c.table_name().to_string(), input, &self);

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
                return Err(format!("{link_errors:?}").into());
            }

            if map_is_empty(&mut input.links).not() {
                return Err("unused input".into());
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
            Err(sqlx::Error::RowNotFound) => return Err("nothing found".into()),
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

    #[cfg(feature = "unstable")]
    pub async fn select_one_serialized(
        &self,
        mut input: SelectOneInput,
    ) -> Result<LinkedOutput<Value, Map<String, Value>>, SelectOneError> {
        let c = self
            .collections
            .get(&input.collection)
            .ok_or(format!("collection {} was not found", input.collection))?;

        let mut st = SelectSt::init(c.table_name());

        for WithTypeid { typeid, rest } in input.filters {
            let filter = if let Some(found) = self.filter_extentions.get(&typeid) {
                found
            } else {
                let existing_filters = self.filter_extentions.keys().collect::<Vec<_>>();
                // self.debugger_error()

                return Err(SelectOneError::TypeidIsNotRegistered(
                    TypeidIsNotRegistered {
                        requested: typeid,
                        to_impl: "filter".to_string(),
                    },
                ));
            };

            filter.on_select(rest, &**c, &mut st)?;
            // st.where_("");
        }

        for WithTypeid { typeid, rest } in input.links {
            let link = if let Some(found) = self.links.get(&typeid) {
                let input = rest;
                found.on_select_one(base_col, input, client)
            } else {
                return Err(SelectOneError::TypeidIsNotRegistered(
                    TypeidIsNotRegistered {
                        requested: typeid,
                        to_impl: "link_data".to_string(),
                    },
                ));
            };
        }

        let mut links = {
            let mut link_errors = Vec::default();
            let links = self
                .links
                .iter()
                .filter_map(|e| {
                    let name = e.1.json_selector();
                    let input = from_map(&mut input.links, &e.0.body)?;
                    // TODO: it is possible to have bugs where you drop Arc<Self> (mutate it) and then use the next Box
                    let s = e.1.on_select_one(c.table_name().to_string(), input, &self);

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
                return Err(format!("{link_errors:?}").into());
            }

            if map_is_empty(&mut input.links).not() {
                return Err("unused input".into());
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
            Err(sqlx::Error::RowNotFound) => return Err("nothing found".into()),
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
