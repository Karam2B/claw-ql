use super::LinkData;
use crate::{
    QueryBuilder,
    collections::{CollectionBasic, OnMigrate},
    json_client::{
        DynamicLinkBT, DynamicLinkRT, FromParameter, JsonSelector, RuntimeResult,
        SelectOneJsonFragment,
    },
    links::set_id::SetId,
    operations::select_one_op::SelectOneFragment,
};
use core::fmt;
use serde::{Deserialize, Serialize, de::DeserializeOwned};
use sqlx::Executor;
use std::{any::Any, ops::Not};

#[derive(Serialize, Deserialize)]
#[allow(non_camel_case_types)]
pub struct empty_object {}

#[derive(Clone)]
pub struct Relation<From, To> {
    pub from: From,
    pub to: To,
}

impl<F: CollectionBasic, T: CollectionBasic> Relation<F, T> {
    #[inline]
    pub fn into_spec(&self) -> <Relation<F, T> as LinkData<F>>::Spec
    where
        Relation<F, T>: LinkData<F>,
    {
        self.clone().spec(self.from.clone())
    }
}

impl<F, T> Relation<F, T> {
    pub fn link(from: F, to: T) -> DynamicRelation<F, T> {
        DynamicRelation { from, to }
    }
}

impl<S, F, T> OnMigrate<S> for Relation<F, T>
where
    Relation<F, T>: LinkData<F, Spec: OnMigrate<S>>,
    F: CollectionBasic,
    T: CollectionBasic,
{
    async fn custom_migration<'e>(&self, exec: impl for<'q> Executor<'q, Database = S> + Clone)
    where
        S: QueryBuilder,
    {
        self.into_spec().custom_migration(exec).await
    }
}

impl<S, F, T> OnMigrate<S> for DynamicRelation<F, T>
where
    Relation<F, T>: LinkData<F, Spec: OnMigrate<S>>,
    F: CollectionBasic,
    T: CollectionBasic,
{
    async fn custom_migration<'e>(&self, exec: impl for<'q> Executor<'q, Database = S> + Clone)
    where
        S: QueryBuilder,
    {
        self.into_spec().custom_migration(exec).await
    }
}

pub trait DynamicLinkForRelation {
    fn metadata(&self) -> RelationEntry;
}

#[derive(Debug, Clone)]
pub struct RelationEntry {
    pub from: String,
    pub to: String,
    pub ty: &'static dyn RelationType,
}

// meant to identify each relation uniquely accross json_client
pub trait RelationType: 'static + Send + Sync {
    fn inspect(self: &'static Self) -> &'static str;
}

impl fmt::Debug for &'static dyn RelationType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.inspect())
    }
}

// similar to Relation, but includes opinionated behavior to how
// json_client should handles relations, this exists to avoid an
// opinionated implementations for Relation
#[derive(Clone)]
pub struct DynamicRelation<From, To> {
    pub from: From,
    pub to: To,
}

impl<F: CollectionBasic, T: CollectionBasic> DynamicRelation<F, T> {
    #[inline]
    pub fn into_spec(&self) -> <Relation<F, T> as LinkData<F>>::Spec
    where
        Relation<F, T>: LinkData<F>,
    {
        Relation {
            from: self.from.clone(),
            to: self.to.clone(),
        }
        .spec(self.from.clone())
    }
}

impl<F: CollectionBasic, T: CollectionBasic, S> DynamicLinkBT<S> for DynamicRelation<F, T>
where
    S: QueryBuilder,
    Relation<F, T>: LinkData<F, Spec: DynamicLinkForRelation>,
    // // inverses should always exist for Relations, but I wonder if I should specify another
    // type !!
    // DynamicRelationInverse<T, F>: DynamicLinkBT<S>,
    Self: DynamicLinkRT<S>,
{
    type BuildtimeMeta = RelationEntry;

    fn buildtime_meta(&self) -> Self::BuildtimeMeta {
        self.into_spec().metadata()
    }

    type RuntimeSpec = Self;

    fn finish_building(
        self,
        buildtime_meta: &Vec<Box<dyn Any>>,
    ) -> Result<DynamicRelation<F, T>, std::string::String> {
        Ok(self)
    }

    fn push_more(&self) -> Option<Box<dyn crate::json_client::DynamicLinkBTDyn<S>>> {
        None
        // Some(Box::new(DynamicRelation {
        //     from: self.to.clone(),
        //     to: self.from.clone(),
        // }))
    }
}

impl<F: CollectionBasic, T: CollectionBasic, S> DynamicLinkRT<S> for DynamicRelation<F, T>
where
    S: QueryBuilder,
    Relation<F, T>: LinkData<F, Spec: SelectOneFragment<S, Output: Serialize>>,
    // relation should have set_id relation, but this may not be true for inverse relation!!
    // SetId<T, i64>: LinkData<F, Spec: 'static>,
{
    #[inline]
    fn json_selector(&self) -> JsonSelector {
        JsonSelector {
            collection: FromParameter::Specific(self.from.table_name_lower_case().to_owned()),
            body: vec!["relation", self.to.table_name_lower_case()],
        }
    }

    #[inline]
    fn on_select_one(
        &self,
        base_col: String,
        input: serde_json::Value,
    ) -> RuntimeResult<Box<dyn SelectOneJsonFragment<S>>> {
        if let Err(err) = serde_json::from_value::<empty_object>(input) {
            return RuntimeResult::RuntimeError(err.to_string());
        }

        if base_col != self.from.table_name_lower_case() {
            return RuntimeResult::Skip;
        }

        RuntimeResult::Ok(Box::new((self.into_spec(), Default::default())))
    }
}
