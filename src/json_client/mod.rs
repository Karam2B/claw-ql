use crate::Schema;
use crate::collections::{CollectionHandler, HasHandler, LiqFilter, no_id};
use crate::json_client::add_collection::LiqType;
use crate::links::{LiqLink, LiqLinkExt};
use crate::migration::MigrationStep;
// use crate::json_client::builder_pattern::JsonClientBuilding;
use crate::prelude::stmt::InsertOneSt;
use crate::statements::update_st::UpdateSt;
use crate::{QueryBuilder, collections::Collection, prelude::stmt::SelectSt};
use convert_case::{Case, Casing};
use serde::{Serialize, de::DeserializeOwned};
use serde_json::{Map, Value, from_value};
use sqlx::{Database, Pool};
use std::any::Any;
use std::marker::PhantomData;
use std::sync::atomic::AtomicI64;
use std::{collections::HashMap, pin::Pin, sync::Arc};

pub mod add_collection;
pub mod axum_router_mod;
pub mod realtime;
pub mod select_one;
pub use select_one::SelectOneJsonFragment;
// pub mod update_one;
// pub use update_one::UpdateOneJsonFragment;
// pub mod delete_one;
// pub use delete_one::DeleteOneJsonFragment;
// pub mod insert_one;
// pub use insert_one::InsertOneJsonFragment;

pub struct ErrorId(i64);
pub trait ErrorReporter {
    fn report(&self, input: serde_json::Value) -> ErrorId;
}

pub struct JsonClient<S: Database> {
    pub collections: HashMap<String, Box<dyn JsonCollection<S>>>,
    pub links: HashMap<String, Box<dyn LiqLinkExt<S>>>,
    pub type_extentions: HashMap<String, Box<dyn LiqType<S>>>,
    pub filter_extentions: HashMap<String, Box<dyn LiqFilter<S>>>,
    pub errors_log: Vec<(ErrorId, serde_json::Value)>,
    pub error_count: AtomicI64,
    pub migration: Vec<MigrationStep>,
    pub db: Pool<S>,
}

impl<S: QueryBuilder> JsonClient<S> {
    pub fn from_schema<C: Collections<S>, L>(schema: Schema<C, L>, pool: Pool<S>) -> JsonClient<S> {
        JsonClient {
            collections: schema.collections.into_map(),
            links: Default::default(),
            type_extentions: Default::default(),
            filter_extentions: Default::default(),
            errors_log: Default::default(),
            error_count: 0.into(),
            migration: Vec::default(),
            db: pool,
        }
    }
}

pub enum JsonError {
    ParseError(String),
    String(String),
}

pub trait Collections<S: Database> {
    fn into_map(self) -> HashMap<String, Box<dyn JsonCollection<S>>>;
}

// mod impl_for_tuple {

//     use std::collections::HashMap;

//     use crate::json_client::{Collections, JsonCollection};
//     use convert_case::{Case, Casing};
//     use paste::paste;
//     use sqlx::Database;

//     macro_rules! implt {
//         ($([$ty:ident, $part:literal]),*) => {
//             #[allow(unused)]
//             impl<S: Database, $($ty,)*> Collections<S> for ($($ty,)*)
//             where
//                 $($ty: JsonCollection<S>,)*
//             {
//                 fn into_map(self) -> HashMap<String, Box<dyn JsonCollection<S>>> {
//                     let mut map: HashMap<String, Box<dyn JsonCollection<S>>> = HashMap::new();
//                     $(map.insert(
//                         paste!(self.$part).table_name().to_case(Case::Snake),
//                         Box::new(paste!(self.$part))
//                     );)*
//                     map
//                 }
//             }
//         }
//     }

//     implt!();
//     implt!([C0, 0]);
//     implt!([C0, 0], [C1, 1]);
//     implt!([C0, 0], [C1, 1], [C2, 2]);
// }

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

#[cfg(feature = "inventory")]
impl JsonClient<sqlx::Any> {
    pub fn new_from_inventory(
        db: Pool<sqlx::Any>,
    ) -> BuilderPattern<PhantomData<(to_json_client<sqlx::Any>,)>, (JsonClientBuilding<sqlx::Any>,)>
    {
        use crate::inventory::{Collection, Link};
        use convert_case::{Case, Casing};

        let mut b = JsonClientBuilding {
            collections: Default::default(),
            links: Default::default(),
            flex_ctx: Default::default(),
            db,
        };

        for coll in inventory::iter::<Collection> {
            let coll = (coll.obj)();
            let name = coll.table_name();
            let ret = b.collections.insert(name.to_case(Case::Snake), coll);

            if ret.is_some() {
                panic!(
                    "collections are globally unique, the identifier {} was used twice",
                    name
                )
            }
        }

        for link in inventory::iter::<Link> {
            let obj = (link.obj)();
            let meta = obj.buildtime_meta();

            b.flex_ctx.push(meta);

            let mut more = obj.push_more();
            while more.is_some() {
                let more_inner = more.unwrap();

                let buildtime_meta = more_inner.buildtime_meta();
                b.flex_ctx.push(Box::new(buildtime_meta));

                more = more_inner.push_more();

                b.links.push(more_inner);
            }

            b.links.push(obj);
        }

        BuilderPattern {
            __components: PhantomData,
            __context: (b,),
        }
    }
}

pub trait JsonCollection<S>: Send + Sync + 'static {
    // fn as_b(self: Box<Self>) -> Box<dyn JsonCollectionB>;
    fn clone_self(&self) -> Box<dyn JsonCollection<S>>;
    fn table_name_js(&self) -> &str;
    fn members_js(&self) -> Vec<String>;
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

impl<S: 'static> Clone for Box<dyn JsonCollection<S>> {
    fn clone(&self) -> Self {
        self.clone_self()
    }
}

impl<S: 'static> CollectionHandler for Box<dyn JsonCollection<S>> {
    fn table_name(&self) -> &str {
        self.table_name_js()
    }

    fn table_name_lower_case(&self) -> &str {
        todo!()
    }

    fn members(&self) -> Vec<String> {
        self.members_js()
    }

    type LinkedData = Value;
}

impl<S: 'static> no_id::Collection<S> for Box<dyn JsonCollection<S>> {
    type Partial = Value;

    type Data = Value;

    fn on_select(&self, stmt: &mut SelectSt<S>)
    where
        S: QueryBuilder,
    {
        JsonCollection::on_select(&**self, stmt);
    }

    fn on_insert(&self, this: Self::Data, stmt: &mut InsertOneSt<S>)
    where
        S: sqlx::Database,
    {
        todo!()
    }

    fn on_update(&self, this: Self::Partial, stmt: &mut UpdateSt<S>)
    where
        S: QueryBuilder,
    {
        todo!()
    }

    fn from_row_noscope(&self, row: &<S>::Row) -> Self::Data
    where
        S: Database,
    {
        todo!()
    }

    fn from_row_scoped(&self, row: &<S>::Row) -> Self::Data
    where
        S: Database,
    {
        JsonCollection::from_row_scoped(&**self, row)
    }
}

impl<S, T> JsonCollection<S> for T
where
    T: Clone,
    S: QueryBuilder,
    T: Collection<S> + 'static,
    T::Data: Serialize + DeserializeOwned,
    T::Partial: DeserializeOwned,
{
    fn clone_self(&self) -> Box<dyn JsonCollection<S>> {
        Box::new(self.clone())
    }
    // fn as_b(self: Box<Self>) -> &dyn JsonCollectionB {
    //     &*self
    // }
    #[inline]
    fn members_js(&self) -> Vec<String> {
        CollectionHandler::members(self)
    }

    #[inline]
    fn table_name_js(&self) -> &str {
        CollectionHandler::table_name(self)
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

pub trait DynamicLinkBT<S: Database> {
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

pub trait DynamicLinkRT<S: Database>: 'static + Send + Sync {
    fn json_selector(&self) -> JsonSelector;
    fn on_select_one<'a>(
        &self,
        base_col: String,
        input: Value,
        client: &JsonClient<S>,
    ) -> Result<Box<dyn SelectOneJsonFragment<S>>, String>;
}

pub trait DynamicLinkBTDyn<S> {
    fn buildtime_meta(&self) -> Box<dyn Any>;
    fn finish_building(
        self: Box<Self>,
        buildtime_meta: &Vec<Box<dyn Any>>,
    ) -> Result<Box<dyn DynamicLinkRT<S>>, String>;
    fn push_more(&self) -> Option<Box<dyn DynamicLinkBTDyn<S>>>;
}

impl<S, T> DynamicLinkBTDyn<S> for T
where
    S: Database,
    T: DynamicLinkBT<S>,
{
    #[inline]
    fn buildtime_meta(&self) -> Box<dyn Any> {
        Box::new(DynamicLinkBT::buildtime_meta(self))
    }

    #[inline]
    fn finish_building(
        self: Box<Self>,
        buildtime_meta: &Vec<Box<dyn Any>>,
    ) -> Result<Box<dyn DynamicLinkRT<S>>, String> {
        Ok(Box::new(DynamicLinkBT::finish_building(
            *self,
            buildtime_meta,
        )?))
    }

    #[inline]
    fn push_more(&self) -> Option<Box<dyn DynamicLinkBTDyn<S>>> {
        DynamicLinkBT::push_more(self)
    }
}

// // useful but obselete primitive
// pub struct ReturnAsJsonMap<T>(pub Vec<(String, T)>);
//
// // a common pattern is you have array of fragments and you
// // want to build them as a map
// impl<S: QueryBuilder, T> SelectOneJsonFragment<S> for ReturnAsJsonMap<T>
// where
//     T: SelectOneJsonFragment<S>,
// {
//     fn on_select(&mut self, st: &mut SelectSt<S>) {
//         self.0.iter_mut().for_each(|e| e.1.on_select(st))
//     }
//
//     fn from_row(&mut self, row: &<S>::Row) {
//         self.0.iter_mut().for_each(|e| e.1.from_row(row))
//     }
//
//     fn sub_op<'this>(
//         &'this mut self,
//         pool: Pool<S>,
//     ) -> Pin<Box<dyn Future<Output = ()> + Send + 'this>> {
//         Box::pin(async move {
//             for item in self.0.iter_mut() {
//                 item.1.sub_op(pool.clone()).await
//             }
//         })
//     }
//
//     fn take(self: Box<Self>) -> serde_json::Value {
//         let mut map = serde_json::Map::new();
//         self.0.into_iter().for_each(|e| {
//             map.insert(e.0, Box::new(e.1).take());
//         });
//         map.into()
//     }
// }

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

mod abstract_over_json_client {
    // this is an attempt to remove the extra four where clause that I included in
    // every json_client-related function, I just think the extra complication
    // and performance cost does not justify it, especially given the flexibilty of
    // the client
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
