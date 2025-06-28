use super::builder_pattern::BuilderPattern;
use crate::collections::CollectionBasic;
use crate::json_client::builder_pattern::JsonClientBuilding;
use crate::prelude::stmt::InsertOneSt;
use crate::statements::update_st::UpdateSt;
use crate::{QueryBuilder, collections::Collection, prelude::stmt::SelectSt};
use builder_pattern::to_json_client;
use serde::{Serialize, de::DeserializeOwned};
use serde_json::{Map, Value, from_value};
use sqlx::sqlite::SqliteRow;
use sqlx::{Database, Pool, Sqlite};
use std::any::Any;
use std::marker::PhantomData;
use std::{collections::HashMap, pin::Pin, sync::Arc};

pub mod builder_pattern;
pub mod select_one;
pub use select_one::SelectOneJsonFragment;
// pub mod update_one;
// pub use update_one::UpdateOneJsonFragment;
// pub mod delete_one;
// pub use delete_one::DeleteOneJsonFragment;
// pub mod insert_one;
// pub use insert_one::InsertOneJsonFragment;

mod abstract_over_json_client {
    use std::sync::Arc;

    use sqlx::{Database, Sqlite, sqlite::SqliteRow};

    use crate::QueryBuilder;

    pub trait DatabaseForJC: Sized + Database + QueryBuilder {
        fn this_as_jc_actions() -> Arc<dyn JsonClientActions<Self>>;
    }

    /// this trait is to eliminate all the where predicate by doing things
    /// more dynamicly, this is often come at performance cost at the expence
    /// of "cleaner" code, but I think that abstracting over JsonClient is
    /// rare enough to justify this cost
    pub trait JsonClientActions<S>: Send + Sync + 'static {
        fn i64_decode(&self, row: &S::Row, name: &str) -> i64
        where
            S: Database;
        // this method is to remvoe two where predicate, but is it worth creating new heap allocation? in theory that is a shortcumming of Rust, and a cost whoever want to abstract over JsonClient has to pay!
        //
        // fn select_one_op(
        //     &self,
        //     st: SelectSt<S>,
        //     c: &dyn JsonCollection<S>,
        //     links: &mut Vec<(JsonSelector, Box<dyn SelectOneJsonFragment<S>>)>,
        //     pool: Pool<S>,
        // ) -> Box<dyn Future<Output = Result<(), ()>>>
        // where
        //     S: Database + QueryBuilder;
    }

    impl DatabaseForJC for Sqlite {
        fn this_as_jc_actions() -> Arc<dyn JsonClientActions<Self>> {
            Arc::new(Sqlite)
        }
    }

    impl JsonClientActions<Sqlite> for Sqlite {
        #[inline]
        #[track_caller]
        fn i64_decode(&self, row: &SqliteRow, name: &str) -> i64 {
            use sqlx::Row;
            row.get(name)
        }
    }
}

pub struct JsonClient<S: Database> {
    pub collections: HashMap<String, Arc<dyn JsonCollection<S>>>,
    pub links: HashMap<JsonSelector, Arc<dyn DynamicLinkRT<S>>>,
    pub db: Pool<S>,
}

#[derive(Debug, Clone, Eq, Hash, PartialEq)]
pub struct JsonSelector {
    /// mimics how rust infer `D` in `LinkData<D>` trait
    pub collection: FromParameter,
    /// mimics how what follows the token `for` in impl block
    /// but this is limited to how json read json maps
    pub body: Vec<&'static str>,
}

#[derive(Debug, Clone, Hash, PartialEq, Eq)]
pub enum FromParameter {
    /// equivalent to `impl LinkData<specific_collection>`
    Specific(String),
    /// equivalent to `impl<C> LinkData<C>`
    Generic,
}

impl<S> JsonClient<S>
where
    S: QueryBuilder,
{
    pub fn init(
        db: Pool<S>,
    ) -> BuilderPattern<PhantomData<(to_json_client<S>,)>, (JsonClientBuilding<S>,)> {
        BuilderPattern::default()
            .build_component(to_json_client(db))
            .start()
    }
}

pub trait JsonCollection<S>: Send + Sync + 'static {
    fn table_name(&self) -> &'static str;
    fn members(&self) -> Vec<String>;
    fn on_select(&self, stmt: &mut SelectSt<S>)
    where
        S: QueryBuilder;

    fn on_insert(&self, this: Value, stmt: &mut InsertOneSt<S>) -> Result<(), String>
    where
        S: sqlx::Database;
    fn on_update(&self, this: Value, stmt: &mut UpdateSt<S>) -> Result<(), String>
    where
        S: QueryBuilder;

    fn from_row_noscope(&self, row: &S::Row) -> Value
    where
        S: Database;
    fn from_row_scoped(&self, row: &S::Row) -> Value
    where
        S: Database;
}

impl<S, T> JsonCollection<S> for T
where
    S: QueryBuilder,
    T: Collection<S> + 'static,
    T::Data: Serialize + DeserializeOwned,
    T::Partial: DeserializeOwned,
{
    #[inline]
    fn members(&self) -> Vec<String> {
        CollectionBasic::members(self)
    }

    #[inline]
    fn table_name(&self) -> &'static str {
        CollectionBasic::table_name(self)
    }

    #[inline]
    fn on_select(&self, stmt: &mut SelectSt<S>)
    where
        S: QueryBuilder,
    {
        Collection::<S>::on_select(self, stmt)
    }

    #[inline]
    fn on_insert(&self, this: Value, stmt: &mut InsertOneSt<S>) -> Result<(), String>
    where
        S: sqlx::Database,
    {
        let input = from_value::<T::Data>(this).map_err(|r| r.to_string())?;
        Collection::<S>::on_insert(self, input, stmt);
        Ok(())
    }

    #[inline]
    fn on_update(&self, this: Value, stmt: &mut UpdateSt<S>) -> Result<(), String>
    where
        S: QueryBuilder,
    {
        let input = from_value::<T::Partial>(this).map_err(|r| r.to_string())?;
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
}

// === Other Requirement ===
pub enum RuntimeResult<T> {
    Skip,
    RuntimeError(String),
    Ok(T),
}

impl<T> RuntimeResult<T> {
    pub fn map<F, O>(self, f: F) -> RuntimeResult<O>
    where
        F: FnOnce(T) -> O,
    {
        match self {
            RuntimeResult::Ok(output) => RuntimeResult::Ok(f(output)),
            RuntimeResult::Skip => RuntimeResult::Skip,
            RuntimeResult::RuntimeError(str) => RuntimeResult::RuntimeError(str),
        }
    }
}

pub trait DynamicLinkBT<S> {
    /// every impl DynamicLinkBT should provide as much information
    /// about it as possible so other implemntor know how to specify
    /// their behavior.
    ///
    /// all BuildtimeMeta will be accessible at `finish_building`
    /// for all implemntor of this trait
    type BuildtimeMeta: Any;
    fn buildtime_meta(&self) -> Self::BuildtimeMeta;

    type RuntimeSpec: DynamicLinkRT<S>;
    fn finish_building(
        self,
        all_buildtime_meta: &Vec<Box<dyn Any>>,
    ) -> Result<Self::RuntimeSpec, String>;

    /// this method is to make the trait more flexible
    /// but bad implementation of this method can lead to infinate loop
    fn push_more(&self) -> Option<Box<dyn DynamicLinkBTDyn<S>>> {
        None
    }
}

pub trait DynamicLinkRT<S>: 'static + Send + Sync {
    fn json_selector(&self) -> JsonSelector;
    fn on_select_one(
        &self,
        base_col: String,
        input: Value,
    ) -> Result<Box<dyn SelectOneJsonFragment<S>>, String>;
}

pub trait DynamicLinkBTDyn<S> {
    fn buildtime_meta(&self) -> Box<dyn Any>;
    fn finish_building(
        self: Box<Self>,
        buildtime_meta: &Vec<Box<dyn Any>>,
    ) -> Result<Arc<dyn DynamicLinkRT<S>>, String>;
    fn push_more(&self) -> Option<Box<dyn DynamicLinkBTDyn<S>>>;
}

impl<S, T: DynamicLinkBT<S>> DynamicLinkBTDyn<S> for T {
    #[inline]
    fn buildtime_meta(&self) -> Box<dyn Any> {
        Box::new(DynamicLinkBT::buildtime_meta(self))
    }

    #[inline]
    fn finish_building(
        self: Box<Self>,
        buildtime_meta: &Vec<Box<dyn Any>>,
    ) -> Result<Arc<dyn DynamicLinkRT<S>>, String> {
        Ok(Arc::new(DynamicLinkBT::finish_building(
            *self,
            buildtime_meta,
        )?))
    }

    #[inline]
    fn push_more(&self) -> Option<Box<dyn DynamicLinkBTDyn<S>>> {
        DynamicLinkBT::push_more(self)
    }
}

// === Usefuls ===
pub struct ReturnAsJsonMap<T>(pub Vec<(String, T)>);

// a common pattern is you have array of fragments and you
// want to build them as a map
impl<S: QueryBuilder, T> SelectOneJsonFragment<S> for ReturnAsJsonMap<T>
where
    T: SelectOneJsonFragment<S>,
{
    fn on_select(&mut self, st: &mut SelectSt<S>) {
        self.0.iter_mut().for_each(|e| e.1.on_select(st))
    }

    fn from_row(&mut self, row: &<S>::Row) {
        self.0.iter_mut().for_each(|e| e.1.from_row(row))
    }

    fn sub_op<'this>(
        &'this mut self,
        pool: Pool<S>,
    ) -> Pin<Box<dyn Future<Output = ()> + Send + 'this>> {
        Box::pin(async move {
            for item in self.0.iter_mut() {
                item.1.sub_op(pool.clone()).await
            }
        })
    }

    fn take(self: Box<Self>) -> serde_json::Value {
        let mut map = serde_json::Map::new();
        self.0.into_iter().for_each(|e| {
            map.insert(e.0, Box::new(e.1).take());
        });
        map.into()
    }
}

pub fn from_map(map: &mut Map<String, Value>, from: &Vec<&'static str>) -> Option<Value> {
    if from.len() == 1 {
        return Some(map.remove_entry(from[0])?.1);
    } else if from.len() == 2 {
        return Some(
            map.get_mut(from[0])?
                .as_object_mut()?
                .remove_entry(from[1])?
                .1,
        );
    } else if from.len() == 3 {
        return Some(
            map.get_mut(from[0])?
                .as_object_mut()?
                .get_mut(from[1])?
                .as_object_mut()?
                .remove_entry(from[1])?
                .1,
        );
    } else {
        panic!(
            "accessor of more that 3 \"{from:?}\" can be supported via recursive function but need unit testing to make sure it is valid"
        );
    }
}

pub fn map_is_empty(map: &mut Map<String, Value>) -> bool {
    if map.len() == 0 {
        true
    } else {
        map.values_mut().any(|e| {
            if let Some(e) = e.as_object_mut() {
                map_is_empty(e)
            } else {
                false
            }
        })
    }
}

#[cfg(test)]
mod tests {
    use crate::json_client::map_is_empty;
    use serde_json::json;
    use std::ops::Not;

    #[test]
    fn map_is_empty_1() {
        let mut input = json!({});
        let input = input.as_object_mut().unwrap();
        assert!(map_is_empty(input));

        let mut input = json!({"foo": true});
        let input = input.as_object_mut().unwrap();
        assert!(map_is_empty(input).not());

        let mut input = json!({"foo": {}});
        let input = input.as_object_mut().unwrap();
        assert!(map_is_empty(input));
    }
}
