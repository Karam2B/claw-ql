use core::fmt;
use std::{any::Any, ops::Not};

use super::LinkData;
use crate::{
    QueryBuilder,
    collections::{CollectionBasic, OnMigrate},
    json_client::{DynamicLink, RuntimeResult, SelectOneJsonFragment},
};
use serde::{Deserialize, Serialize, de::DeserializeOwned};
use sqlx::Executor;

#[derive(Serialize, Deserialize)]
#[allow(non_camel_case_types)]
pub struct empty_object {}

#[derive(Clone)]
pub struct Relation<From, To> {
    pub from: From,
    pub to: To,
}

impl<S, F, T> OnMigrate<S> for Relation<F, T>
where
    Relation<F, T>: LinkData<F, Spec: OnMigrate<S>>,
    Relation<F, T>: Clone,
    F: Clone,
{
    async fn custom_migration<'e>(&self, exec: impl for<'q> Executor<'q, Database = S> + Clone)
    where
        S: QueryBuilder,
    {
        let relation = self.clone();
        let spec = relation.spec(self.from.clone());
        spec.custom_migration(exec).await;
    }
}

// todo: add meta information here!!
#[derive(Debug, Clone)]
pub struct RelationEntry {
    pub from: String,
    pub to: String,
    /// like 'many_to_many<..>', 'one_to_many<..>', etc. there is no way to have this unique for now.
    pub ty: &'static dyn RelationType,
}

pub trait RelationType: 'static + Send + Sync {
    fn inspect(self: &'static Self) -> &'static str;
}

impl fmt::Debug for &'static dyn RelationType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.inspect())
    }
}

#[allow(non_camel_case_types)]
struct many_to_many;
static MANY_TO_MANY: many_to_many = many_to_many;
impl RelationType for many_to_many {
    fn inspect(self: &'static Self) -> &'static str {
        "many_to_many"
    }
}

pub struct RelationEntries {
    pub entries: Vec<RelationEntry>,
    #[allow(unused)]
    private_to_construct_hack: (),
}

pub trait DynamicLinkForRelation<S: QueryBuilder> {
    fn make_entry(&self) -> RelationEntry;
    type SelectOneInput: DeserializeOwned;
    type SelectOne: SelectOneJsonFragment<S>;
    fn on_select_one_fr(
        &self,
        base_col: String,
        input: Self::SelectOneInput,
    ) -> RuntimeResult<Self::SelectOne>;
}

impl<F, T, S> DynamicLink<S> for Relation<F, T>
where
    Relation<F, T>: LinkData<F, Spec: DynamicLinkForRelation<S>>,
    F: Send + Sync + 'static + Clone,
    T: Send + Sync + 'static + Clone + CollectionBasic,
    S: QueryBuilder,
{
    type RuntimeEntry = RelationEntries;
    fn buildtime(&self) -> Vec<Box<dyn Any>> {
        let entry = self.clone().spec(self.from.clone()).make_entry();
        vec![Box::new(entry)]
    }

    fn json_entry(&self) -> Vec<&'static str> {
        vec!["relation", self.to.table_name_lower_case()]
    }

    fn finish_building(ctx: &Vec<Box<dyn std::any::Any>>) -> Result<Self::RuntimeEntry, String> {
        let entries: Vec<RelationEntry> = ctx
            .into_iter()
            .filter_map(|e| (**e).downcast_ref::<RelationEntry>())
            .map(|e| e.clone())
            .collect();

        let mut set = std::collections::HashSet::new();
        for entry in entries.iter() {
            let false_if_existed = set.insert((entry.ty.inspect(), entry.ty.type_id()));
            if false_if_existed.not() {
                panic!(
                    "type {} defines duplicate {}",
                    std::any::type_name_of_val(entry.ty),
                    entry.ty.inspect()
                )
            }
        }

        Ok(RelationEntries {
            private_to_construct_hack: (),
            entries,
        })
    }

    type SelectOneInput = <<Self as LinkData<F>>::Spec as DynamicLinkForRelation<S>>::SelectOneInput;

    type SelectOne = <<Self as LinkData<F>>::Spec as DynamicLinkForRelation<S>>::SelectOne;

    #[inline]
    fn on_select_one(
        &self,
        base_col: String,
        input: Self::SelectOneInput,
        entry: &Self::RuntimeEntry,
    ) -> crate::json_client::RuntimeResult<Self::SelectOne> {
        self.clone()
            .spec(self.from.clone())
            .on_select_one_fr(base_col, input)
    }
}
