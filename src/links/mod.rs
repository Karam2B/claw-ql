#![allow(non_camel_case_types)]
use std::collections::HashMap;

use claw_ql_macros::simple_enum;
use hyper::StatusCode;
use serde::{Deserialize, Serialize, de::DeserializeOwned};
use serde_json::Value as JsonValue;
use serde_json::from_value;

use crate::QueryBuilder;
use crate::collections::Collection;
use crate::collections::CollectionBasic;
use crate::collections::MemberBasic;

// pub mod date_mod;
// pub mod group_by;
// pub mod relation_many_to_many;
pub mod relation_optional_to_many;
// pub mod set_id;
// pub mod set_new;

pub trait Link<Base> {
    type Spec;
    fn spec(self, base: &Base) -> Self::Spec;
}

pub trait LinkedToCollection {
    type To;
}

pub trait LinkedViaId {}
pub trait LinkedViaIds {}

mod deprecated {
    mod relation {
        // use crate::{
        //     QueryBuilder,
        //     collections::CollectionBasic,
        //     json_client::{
        //         DynamicLinkBT, DynamicLinkRT, FromParameter, JsonClient, JsonSelector, RuntimeResult,
        //         SelectOneJsonFragment,
        //     },
        //     links::set_id::SetId,
        //     migration::OnMigrate,
        //     operations::select_one_op::SelectOneFragment,
        // };
        // use convert_case::{Case, Casing};
        // use core::fmt;
        // use serde::{Deserialize, Serialize, de::DeserializeOwned};
        // use sqlx::Executor;
        // use std::{any::Any, ops::Not, sync::Arc};

        // #[derive(Serialize, Deserialize)]
        // #[allow(non_camel_case_types)]
        // pub struct empty_object {}

        // #[derive(Clone)]
        // pub struct Relation<From, To> {
        //     pub from: From,
        //     pub to: To,
        // }

        // // impl Link<$for> for Relation<$for, $to> { .. }

        // impl<F: CollectionBasic, T: CollectionBasic> Relation<F, T> {
        //     #[inline]
        //     pub fn into_spec(&self) -> <Relation<F, T> as LinkData<F>>::Spec
        //     where
        //         Relation<F, T>: LinkData<F>,
        //     {
        //         self.clone().spec(self.from.clone())
        //     }
        // }

        // impl<F, T> Relation<F, T> {
        //     pub fn link(from: F, to: T) -> DynamicRelation<F, T> {
        //         DynamicRelation { from, to }
        //     }
        // }

        // impl<S, F, T> OnMigrate<S> for Relation<F, T>
        // where
        //     Relation<F, T>: LinkData<F, Spec: OnMigrate<S>>,
        //     F: CollectionBasic,
        //     T: CollectionBasic,
        // {
        //     fn custom_migrate_statements(&self) -> Vec<String> {
        //         self.into_spec().custom_migrate_statements()
        //     }
        // }

        // impl<S, F, T> OnMigrate<S> for DynamicRelation<F, T>
        // where
        //     Relation<F, T>: LinkData<F, Spec: OnMigrate<S>>,
        //     F: CollectionBasic,
        //     T: CollectionBasic,
        // {
        //     fn custom_migrate_statements(&self) -> Vec<String> {
        //         self.into_spec().custom_migrate_statements()
        //     }
        // }

        // pub trait DynamicLinkForRelation {
        //     fn metadata(&self) -> RelationEntry;
        // }

        // #[derive(Debug, Clone)]
        // pub struct RelationEntry {
        //     pub from: String,
        //     pub to: String,
        //     pub ty: &'static dyn RelationType,
        // }

        // // meant to identify each relation uniquely accross json_client
        // pub trait RelationType: 'static + Send + Sync {
        //     fn inspect(self: &'static Self) -> &'static str;
        // }

        // impl fmt::Debug for &'static dyn RelationType {
        //     fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        //         f.write_str(self.inspect())
        //     }
        // }

        // // similar to Relation, but includes opinionated behavior to how
        // // json_client should handles relations, this exists to avoid an
        // // opinionated implementations for Relation
        // #[derive(Clone)]
        // pub struct DynamicRelation<From, To> {
        //     pub from: From,
        //     pub to: To,
        // }

        // impl<F: CollectionBasic, T: CollectionBasic> DynamicRelation<F, T> {
        //     #[inline]
        //     pub fn into_spec(&self) -> <Relation<F, T> as LinkData<F>>::Spec
        //     where
        //         Relation<F, T>: LinkData<F>,
        //     {
        //         Relation {
        //             from: self.from.clone(),
        //             to: self.to.clone(),
        //         }
        //         .spec(self.from.clone())
        //     }
        // }

        // impl<F: CollectionBasic, T: CollectionBasic, S> DynamicLinkBT<S> for DynamicRelation<F, T>
        // where
        //     S: QueryBuilder,
        //     Relation<F, T>: LinkData<F, Spec: DynamicLinkForRelation>,
        //     // // inverses should always exist for Relations, but I wonder if I should specify another
        //     // type !!
        //     // DynamicRelationInverse<T, F>: DynamicLinkBT<S>,
        //     Self: DynamicLinkRT<S>,
        // {
        //     type BuildtimeMeta = RelationEntry;

        //     fn buildtime_meta(&self) -> Self::BuildtimeMeta {
        //         self.into_spec().metadata()
        //     }

        //     type RuntimeSpec = Self;

        //     fn finish_building(
        //         self,
        //         buildtime_meta: &Vec<Box<dyn Any>>,
        //     ) -> Result<DynamicRelation<F, T>, std::string::String> {
        //         Ok(self)
        //     }

        //     fn push_more(&self) -> Option<Box<dyn crate::json_client::DynamicLinkBTDyn<S>>> {
        //         None
        //         // Some(Box::new(DynamicRelation {
        //         //     from: self.to.clone(),
        //         //     to: self.from.clone(),
        //         // }))
        //     }
        // }

        // impl<F, T, S> DynamicLinkRT<S> for DynamicRelation<F, T>
        // where
        //     S: QueryBuilder,
        //     Relation<F, T>: LinkData<F, Spec: SelectOneFragment<S, Output: Serialize>>,
        //     F: CollectionBasic,
        //     T: CollectionBasic,
        //     // relation should have set_id relation, but this may not be true for inverse relation!!
        //     // SetId<T, i64>: LinkData<F, Spec: 'static>,
        // {
        //     #[inline]
        //     fn json_selector(&self) -> JsonSelector {
        //         JsonSelector {
        //             collection: FromParameter::Specific(self.from.table_name_lower_case().to_owned()),
        //             body: vec!["relation", self.to.table_name_lower_case()],
        //         }
        //     }

        //     #[inline]
        //     fn on_select_one(
        //         &self,
        //         base_col: String,
        //         input: serde_json::Value,
        //         client: &JsonClient<S>,
        //     ) -> Result<Box<dyn SelectOneJsonFragment<S>>, String> {
        //         if let Err(err) = serde_json::from_value::<empty_object>(input) {
        //             return Err(err.to_string());
        //         }

        //         if base_col.to_case(Case::Snake) != self.from.table_name_lower_case() {
        //             return Err(format!(
        //                 "{} is not related to {}",
        //                 base_col,
        //                 self.from.table_name_lower_case()
        //             ));
        //         }

        //         Ok(Box::new((self.into_spec(), Default::default())))
        //     }
        // }
    }

    mod op_to_many {
        // #[cfg(feature = "serde")]
        // mod dynamic_client {
        //     use serde::Serialize;
        //     use sqlx::{ColumnIndex, Decode, prelude::Type};

        //     use crate::{
        //         QueryBuilder,
        //         collections::{Collection, CollectionBasic},
        //         json_client::RuntimeResult,
        //         // links::relation::{DynamicLinkForRelation, RelationEntry, RelationType},
        //         // prelude::macro_relation::{OptionalToManyInverse, optional_to_many},
        //     };

        //     impl<F, T> DynamicLinkForRelation for OptionalToManyInverse<F, T>
        //     where
        //         F: CollectionBasic,
        //         T: CollectionBasic,
        //     {
        //         fn metadata(&self) -> crate::links::relation::RelationEntry {
        //             struct OptionalToManyInverseIdent;
        //             static OPTIONAL_TO_MANY_INVERSE: OptionalToManyInverseIdent =
        //                 OptionalToManyInverseIdent;
        //             impl RelationType for OptionalToManyInverseIdent {
        //                 fn inspect(self: &'static Self) -> &'static str {
        //                     "optional_to_many_inverse"
        //                 }
        //             }

        //             RelationEntry {
        //                 from: self.from.table_name().to_owned(),
        //                 to: self.to.table_name().to_owned(),
        //                 ty: &OPTIONAL_TO_MANY_INVERSE,
        //             }
        //         }
        //     }
        //     impl<F, T> DynamicLinkForRelation for optional_to_many<F, T>
        //     where
        //         F: CollectionBasic,
        //         T: CollectionBasic,
        //     {
        //         fn metadata(&self) -> crate::links::relation::RelationEntry {
        //             struct OptionalToManyIdent;
        //             static OPTIONAL_TO_MANY: OptionalToManyIdent = OptionalToManyIdent;
        //             impl RelationType for OptionalToManyIdent {
        //                 fn inspect(self: &'static Self) -> &'static str {
        //                     "optional_to_many"
        //                 }
        //             }

        //             RelationEntry {
        //                 from: self.from.table_name().to_owned(),
        //                 to: self.to.table_name().to_owned(),
        //                 ty: &OPTIONAL_TO_MANY,
        //             }
        //         }
        //     }
        // }
    }

    mod set_new {

        // use super::relation_many_to_many::ManyToMany;
        // use super::{LinkData, relation::Relation, relation_optional_to_many::optional_to_many};
        // use crate::collections::{Collection, CollectionBasic};
        // use crate::execute::Execute;
        // use crate::operations::CollectionOutput;
        // use crate::{
        //     QueryBuilder, collections::HasHandler, operations::insert_one_op::InsertOneFragment,
        //     prelude::stmt::InsertOneSt,
        // };
        // use sqlx::Row;
        // use sqlx::{ColumnIndex, Decode, Encode, prelude::Type};

        // pub struct SetNew<Input> {
        //     pub input: Input,
        // }

        // pub trait SetNewTrait<T1, T2> {
        //     type Input;
        // }

        // pub struct SetNewSpec<Relation, Input> {
        //     relation: Relation,
        //     input: Option<Input>,
        // }

        // #[rustfmt::skip]
        // impl<C, To: HasHandler> LinkData<C> for SetNew<To>
        // where
        //     C: CollectionBasic,
        //     Relation<C, To::Handler>:
        //         LinkData<
        //             C,
        //             Spec: SetNewTrait<C, To::Handler, Input = To>
        //         >,
        // {
        //     type Spec = SetNewSpec<<Relation<C, To::Handler> as LinkData<C>>::Spec, To>;

        //     fn spec(self, from: C) -> Self::Spec
        //     where
        //         Self: Sized,
        //     {
        //         let relation = Relation {
        //             from: from.clone(),
        //             to: To::Handler::default()
        //         }
        //         .spec(from);
        //         SetNewSpec {
        //             relation,
        //             input: Some(self.input),
        //         }
        //     }
        // }

        // impl<C, To: HasHandler> LinkData<C> for SetNew<Vec<To>>
        // where
        //     C: CollectionBasic,
        //     Relation<C, To::Handler>: LinkData<C, Spec: SetNewTrait<C, To::Handler, Input = Vec<To>>>,
        // {
        //     type Spec = SetNewSpec<<Relation<C, To::Handler> as LinkData<C>>::Spec, Vec<To>>;

        //     fn spec(self, from: C) -> Self::Spec
        //     where
        //         Self: Sized,
        //     {
        //         let relation = Relation {
        //             from: from.clone(),
        //             to: To::Handler::default(),
        //         }
        //         .spec(from);
        //         SetNewSpec {
        //             relation,
        //             input: Some(self.input),
        //         }
        //     }
        // }

        // impl<T1, T2: CollectionBasic> SetNewTrait<T1, T2> for optional_to_many<T1, T2> {
        //     type Input = T2::LinkedData;
        // }

        // impl<S, C, To> InsertOneFragment<S> for SetNewSpec<optional_to_many<C, To>, To::LinkedData>
        // where
        //     C: Collection<S>,
        //     To: Collection<S>,
        //     To: CollectionBasic<LinkedData: Send + Sync>,
        //     S: QueryBuilder,
        //     To: Send + Sync,
        //     C: Send + Sync,
        //     i64: Type<S> + for<'e> Encode<'e, S> + for<'e> Decode<'e, S>,
        //     for<'s> &'s str: ColumnIndex<S::Row>,
        // {
        //     type Inner = (Option<To::LinkedData>, Option<i64>);

        //     type Output = CollectionOutput<To::LinkedData>;

        //     fn on_insert(&mut self, data: &mut Self::Inner, st: &mut InsertOneSt<S>) {
        //         st.col(self.relation.foriegn_key.clone(), data.1.unwrap())
        //     }

        //     fn returning(&mut self) -> Vec<String> {
        //         /* no op: I already have the foriegn_key value */
        //         vec![]
        //     }

        //     fn from_row(&mut self, _data: &mut Self::Inner, _row: &S::Row) {
        //         /* no op: I already have the foriegn_key value */
        //     }

        //     fn second_sub_op<'this, E: for<'q> sqlx::Executor<'q, Database = S> + Clone>(
        //         &'this mut self,
        //         _data: &'this mut Self::Inner,
        //         _exec: E,
        //     ) -> impl Future<Output = ()> + Send + use<'this, To, C, S, E> {
        //         async { /* no-op */ }
        //     }
        //     fn first_sub_op<'this, E: for<'q> sqlx::Executor<'q, Database = S> + Clone>(
        //         &'this mut self,
        //         data: &'this mut Self::Inner,
        //         exec: E,
        //     ) -> impl Future<Output = ()> + Send + use<'this, To, C, S, E> {
        //         async {
        //             let mut st = InsertOneSt::init(self.relation.to.table_name().to_string());
        //             self.relation.to.on_insert(
        //                 self.input
        //                     .take()
        //                     .expect("input should be initialized as Some(*) and taken only once"),
        //                 &mut st,
        //             );

        //             let mut members = self.relation.to.members();
        //             members.extend(["id".to_string()]);
        //             st.returning(members)
        //                 .fetch_one(exec, |r| {
        //                     *data = (
        //                         Some(self.relation.to.from_row_noscope(&r)),
        //                         Some(r.get("id")),
        //                     );
        //                     Ok(())
        //                 })
        //                 .await
        //                 .unwrap();
        //         }
        //     }

        //     fn take(self, data: Self::Inner) -> Self::Output {
        //         CollectionOutput {
        //             id: data.1.unwrap(),
        //             attr: data.0.unwrap(),
        //         }
        //     }
        // }

        // impl<T1, T2: CollectionBasic> SetNewTrait<T1, T2> for ManyToMany<T1, T2> {
        //     type Input = Vec<T2::LinkedData>;
        // }

        // // impl<S, C, To> InsertOneFragment<S> for SetNewSpec<ManyToMany<C, To>, Vec<To::LinkedData>>
        // // where
        // //     C: Collection<S>,
        // //     To: Collection<S>,
        // //     To::LinkedData: Clone,
        // //     To: CollectionBasic<LinkedData: Send + Sync>,
        // //     S: QueryBuilder,
        // //     To: Send + Sync,
        // //     C: Send + Sync,
        // //     i64: Type<S> + for<'e> Encode<'e, S> + for<'e> Decode<'e, S>,
        // //     for<'s> &'s str: ColumnIndex<S::Row>,
        // // {
        // // }
    }

    mod group_by {
        // use super::LinkData;
        // use super::relation::Relation;
        // use super::relation_many_to_many::ManyToMany;
        // use crate::QueryBuilder;
        // use crate::collections::Collection;
        // use crate::migration::OnMigrate;
        // use crate::prelude::col;
        // use crate::prelude::macro_relation::optional_to_many;
        // use crate::{
        //     collections::CollectionBasic,
        //     operations::select_one_op::SelectOneFragment,
        //     prelude::{join::join, stmt::SelectSt},
        // };
        // use convert_case::{Case, Casing};
        // use serde::Serialize;
        // use sqlx::{ColumnIndex, Executor};
        // use sqlx::{Sqlite, sqlite::SqliteRow};

        // #[allow(non_camel_case_types)]
        // pub struct count<T>(pub T);

        // pub struct CountSpec<From, To> {
        //     from: From,
        //     to: To,
        //     alias: String,
        //     junction: String,
        // }

        // trait CountingSupportedIn {}
        // impl<T0, T1> CountingSupportedIn for ManyToMany<T0, T1> {}
        // impl<From, To> CountingSupportedIn for optional_to_many<From, To> {}

        // impl<From, To> LinkData<From> for count<To>
        // where
        //     From: CollectionBasic,
        //     To: CollectionBasic,
        //     Relation<From, To>: LinkData<From, Spec: CountingSupportedIn>,
        // {
        //     type Spec = CountSpec<From, To>;
        //     fn spec(self, from: From) -> Self::Spec
        //     where
        //         Self: Sized,
        //     {
        //         let junction = format!("{}{}", self.0.table_name(), from.table_name());
        //         CountSpec {
        //             from,
        //             alias: format!("count_{}_s", self.0.table_name().to_case(Case::Snake)),
        //             to: self.0,
        //             junction,
        //         }
        //     }
        // }

        // #[derive(Clone, Debug, PartialEq, Eq, Serialize)]
        // pub struct CountResult(pub i64);

        // impl<From, To> SelectOneFragment<Sqlite> for CountSpec<From, To>
        // where
        //     From: Send + Sync + Collection<Sqlite>,
        //     To: Send + Sync + Collection<Sqlite>,
        //     // sqlx gasim
        //     for<'s> &'s str: ColumnIndex<SqliteRow>,
        // {
        //     type Inner = Option<i64>;

        //     type Output = CountResult;

        //     fn on_select(&mut self, _data: &mut Self::Inner, st: &mut SelectSt<Sqlite>) {
        //         let column_name_in_junction = format!("{}_id", self.from.table_name().to_case(Case::Snake));
        //         let junction = format!("{}{}", self.to.table_name(), self.from.table_name());
        //         st.select(format!(
        //             "COUNT({junction}.{column_name_in_junction}) AS {alias}",
        //             alias = self.alias
        //         ));
        //         st.join(join {
        //             foriegn_table: self.junction.clone(),
        //             foriegn_column: column_name_in_junction,
        //             local_column: "id".to_string(),
        //         });
        //         st.group_by(col("id").table(&self.from.table_name()));
        //     }

        //     fn from_row(&mut self, data: &mut Self::Inner, row: &SqliteRow) {
        //         use sqlx::Row;
        //         *data = Some(row.get(self.alias.as_str()));
        //     }

        //     fn sub_op<'this>(
        //         &'this mut self,
        //         _data: &'this mut Self::Inner,
        //         _pool: sqlx::Pool<Sqlite>,
        //     ) -> impl Future<Output = ()> + Send + use<'this, From, To> {
        //         async { /* no_op: count has no sub_op */ }
        //     }

        //     fn take(self, data: Self::Inner) -> Self::Output {
        //         CountResult(data.unwrap())
        //     }
        // }

        // // no op
        // impl<S> OnMigrate<S> for count<()> {
        //     fn custom_migrate_statements(&self) -> Vec<String> {
        //         vec![]
        //     }
        //     async fn custom_migration<'e>(&self, _: impl for<'q> Executor<'q, Database = S> + Clone)
        //     where
        //         S: QueryBuilder,
        //     {
        //         // no-op count is on-request only
        //     }
        // }

        // impl count<()> {
        //     pub fn dynamic_link() -> count<()> {
        //         count(())
        //     }
        // }
    }
}
