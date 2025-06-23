use super::builder_pattern::BuilderPattern;
use crate::collections::CollectionBasic;
use crate::json_client::builder_pattern::JsonClientBuilding;
use crate::prelude::stmt::InsertOneSt;
use crate::statements::update_st::UpdateSt;
use crate::{QueryBuilder, collections::Collection, prelude::stmt::SelectSt};
use builder_pattern::to_json_client;
use serde::{Serialize, de::DeserializeOwned};
use serde_json::{Map, Value, from_value};
use sqlx::{Database, Pool};
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

pub struct JsonClient<S: Database> {
    pub(crate) collections: HashMap<String, Arc<dyn JsonCollection<S>>>,
    pub(crate) links: HashMap<Vec<&'static str>, Arc<dyn DynamicLinkRT<S>>>,
    pub(crate) db: Pool<S>,
}

impl<S> JsonClient<S>
where
    S: Database,
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

pub trait DynamicLink<S>
where
    Self: Send + Sync + 'static,
    S: QueryBuilder,
{
    type RuntimeEntry: Send + Sync + 'static;
    fn json_entry(&self) -> Vec<&'static str>;
    fn buildtime(&self) -> Vec<Box<dyn Any>> {
        vec![]
    }
    fn finish_building(ctx: &Vec<Box<dyn Any>>) -> Result<Self::RuntimeEntry, String>;
    type SelectOneInput: DeserializeOwned;
    type SelectOne: SelectOneJsonFragment<S>;
    fn on_select_one(
        &self,
        base_col: String,
        input: Self::SelectOneInput,
        entry: &Self::RuntimeEntry,
    ) -> RuntimeResult<Self::SelectOne>;
    // type InsertOneInput: DeserializeOwned;
    // type InsertOne: InsertOneJsonFragment<S>;
    // fn on_insert_one(
    //     &self,
    //     base_col: String,
    //     input: Self::InsertOneInput,
    //     entry: &Self::RuntimeEntry,
    // ) -> Result<Option<Self::InsertOne>, String>;
    // type DeleteOneInput: DeserializeOwned;
    // type DeleteOne: DeleteOneJsonFragment<S>;
    // fn on_delete_one(
    //     &self,
    //     base_col: String,
    //     input: Self::DeleteOneInput,
    //     entry: &Self::RuntimeEntry,
    // ) -> Result<Option<Self::DeleteOne>, String>;
    // type UpdateOneInput: DeserializeOwned;
    // type UpdateOne: UpdateOneJsonFragment<S>;
    // fn on_update_one(
    //     &self,
    //     base_col: String,
    //     input: Self::UpdateOneInput,
    //     entry: &Self::RuntimeEntry,
    // ) -> Result<Option<Self::UpdateOne>, String>;
}

// a version of DynamicLink used during build_pattern
pub trait DynamicLinkBT<S> {
    fn buildtime_extend(&self, ctx: &mut Vec<Box<dyn Any>>);

    fn json_entry(&self) -> Vec<&'static str>;
    fn finish(
        self: Box<Self>,
        ctx: &Vec<Box<dyn Any>>,
    ) -> Result<Arc<dyn DynamicLinkRT<S>>, String>;
}

impl<S, T> DynamicLinkBT<S> for T
where
    S: QueryBuilder,
    T: DynamicLink<S>,
{
    fn json_entry(&self) -> Vec<&'static str> {
        self.json_entry()
    }
    fn buildtime_extend(&self, ctx: &mut Vec<Box<dyn Any>>) {
        ctx.extend(self.buildtime());
    }
    fn finish(
        self: Box<Self>,
        ctx: &Vec<Box<dyn Any>>,
    ) -> Result<Arc<dyn DynamicLinkRT<S>>, String> {
        let rt = T::finish_building(ctx)?;
        Ok(Arc::new((*self, rt)) as Arc<dyn DynamicLinkRT<S>>)
    }
}

// a version of DynamicLink that is trait-object compatible
pub trait DynamicLinkRT<S>: Send + Sync {
    fn json_entry(&self) -> Vec<&'static str>;
    fn on_select_one(
        &self,
        _base_col: String,
        input: Value,
    ) -> RuntimeResult<Box<dyn SelectOneJsonFragment<S>>>;
    // fn on_update_one(
    //     &self,
    //     base_col: String,
    //     input: Value,
    //     ctx: &dyn Any,
    // ) -> Result<Option<Box<dyn UpdateOneJsonFragment<S>>>, String>;
    // fn on_insert_one(
    //     &self,
    //     base_col: String,
    //     input: Value,
    //     ctx: &dyn Any,
    // ) -> Result<Option<Box<dyn InsertOneJsonFragment<S>>>, String>;
    // fn on_delete_one(
    //     &self,
    //     base_col: String,
    //     input: Value,
    //     ctx: &dyn Any,
    // ) -> Result<Option<Box<dyn DeleteOneJsonFragment<S>>>, String>;
}

impl<S, T> DynamicLinkRT<S> for (T, T::RuntimeEntry)
where
    S: QueryBuilder,
    T: DynamicLink<S>,
{
    fn json_entry(&self) -> Vec<&'static str> {
        T::json_entry(&self.0)
    }
    fn on_select_one(
        &self,
        base_col: String,
        input: Value,
    ) -> RuntimeResult<Box<dyn SelectOneJsonFragment<S>>> {
        let input = from_value::<T::SelectOneInput>(input);
        let input = match input {
            Ok(ok) => ok,
            Err(err) => return RuntimeResult::RuntimeError(err.to_string()),
        };

        DynamicLink::on_select_one(&self.0, base_col, input, &self.1)
            .map(|e| Box::new(e) as Box<dyn SelectOneJsonFragment<S>>)
    }
    //
    // fn on_update_one(
    //     &self,
    //     base_col: String,
    //     input: Value,
    //     ctx: &dyn Any,
    // ) -> Result<Option<Box<dyn UpdateOneJsonFragment<S>>>, String> {
    //     let input = from_value::<T::UpdateOneInput>(input);
    //     let input = match input {
    //         Ok(ok) => ok,
    //         Err(err) => return Err(err.to_string()),
    //     };
    //
    //     let output = DynamicLink::on_update_one(
    //         self,
    //         base_col,
    //         input,
    //         ctx.downcast_ref::<T::RuntimeEntry>().unwrap(),
    //     )?;
    //     let output = match output {
    //         Some(ok) => ok,
    //         None => return Ok(None),
    //     };
    //     let output: Box<dyn UpdateOneJsonFragment<S>> = Box::new(output);
    //
    //     Ok(Some(output))
    // }
    //
    // fn on_insert_one(
    //     &self,
    //     base_col: String,
    //     input: Value,
    //     entry: &dyn Any,
    // ) -> Result<Option<Box<dyn InsertOneJsonFragment<S>>>, String> {
    //     let input = from_value::<T::InsertOneInput>(input);
    //     let input = match input {
    //         Ok(ok) => ok,
    //         Err(err) => return Err(err.to_string()),
    //     };
    //
    //     let output = DynamicLink::on_insert_one(
    //         self,
    //         base_col,
    //         input,
    //         entry.downcast_ref::<T::RuntimeEntry>().unwrap(),
    //     )?;
    //     let output = match output {
    //         Some(ok) => ok,
    //         None => return Ok(None),
    //     };
    //     let output: Box<dyn InsertOneJsonFragment<S>> = Box::new(output);
    //
    //     Ok(Some(output))
    // }
    //
    // fn on_delete_one(
    //     &self,
    //     base_col: String,
    //     input: Value,
    //     entry: &dyn Any,
    // ) -> Result<Option<Box<dyn DeleteOneJsonFragment<S>>>, String> {
    //     let input = from_value::<T::DeleteOneInput>(input);
    //     let input = match input {
    //         Ok(ok) => ok,
    //         Err(err) => return Err(err.to_string()),
    //     };
    //
    //     let output = DynamicLink::on_delete_one(
    //         self,
    //         base_col,
    //         input,
    //         entry.downcast_ref::<T::RuntimeEntry>().unwrap(),
    //     )?;
    //
    //     let output = match output {
    //         Some(ok) => ok,
    //         None => return Ok(None),
    //     };
    //     let output: Box<dyn DeleteOneJsonFragment<S>> = Box::new(output);
    //
    //     Ok(Some(output))
    // }
}

// === Usefuls ===
pub struct ReturnAsJsonMap<T>(pub Vec<(String, T)>);

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
            "accessor more that 3 should be supported via recursive function but need unit testing to make sure it is valid"
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
    use serde_json::json;

    use crate::json_client::map_is_empty;

    #[test]
    fn map_is_empty_1() {
        let mut input = json!({});
        let input = input.as_object_mut().unwrap();
        assert!(map_is_empty(input));
    }
}
