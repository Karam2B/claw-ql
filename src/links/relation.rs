use crate::{
    QueryBuilder,
    any_set::AnySet,
    collections::{Collection, OnMigrate},
    json_client::{JsonCollection, SelectOneJsonFragment},
    operations::select_one_op::SelectOneFragment,
};
use convert_case::{Case, Casing};
use serde::Serialize;
use serde_json::Value;
use sqlx::Executor;
use std::{any::Any, collections::HashMap, ops::Not};

use super::{DynamicLink, LinkData};

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
#[derive(Debug)]
pub struct RelationEntry {
    pub from: String,
    pub to: String,
    /// like 'many_to_many<..>', 'one_to_many<..>', etc. there is no way to have this unique for now.
    pub ty: String,
}

pub struct RelationEntries {
    pub entries: Vec<RelationEntry>,
    #[allow(unused)]
    private_to_construct_hack: (),
}

pub trait DynamicLinkForRelation<S> {
    fn global_ident(&self) -> &'static str;
    fn on_each_select_one_request(
        &self,
        input: Value,
    ) -> Result<Box<dyn SelectOneJsonFragment<S>>, String>;
}

impl<S, F, T> DynamicLink<S> for Relation<F, T>
where
    S: QueryBuilder,
    Relation<F, T>: LinkData<
            F,
            Spec: Any
                      + SelectOneFragment<S, Output: Serialize>
                      // + InsertOneFragment<S, Output: Serialize>
                      + DynamicLinkForRelation<S>,
        >,
    Relation<F, T>: Clone,
    F: Clone,
    F: Collection<S>,
    T: Collection<S>,
{
    // behavior of this function should be tracked via semver
    // other links may depends on the behavior of this function
    fn on_register(&self, entries: &mut Self::Entry) {
        let from = self.from.table_name().to_case(Case::Snake).to_string();
        let to = self.to.table_name().to_case(Case::Snake).to_string();
        let spec = self.clone().spec(self.from.clone());
        let ty = spec.global_ident().to_string();
        let entry = RelationEntry { from, to, ty };
        #[cfg(feature = "trace")]
        tracing::debug!("registering entry {:?}", entry);
        entries.entries.push(entry)
    }

    // type Entry = <Self as LinkData<F>>::Spec::DynamicSpec
    type Entry = RelationEntries;
    fn init_entry() -> Self::Entry {
        RelationEntries {
            entries: Vec::new(),
            private_to_construct_hack: (),
        }
    }

    fn on_finish(&self, _build_ctx: &AnySet) -> Result<(), String> {
        Ok(())
    }

    fn json_entry() -> &'static str {
        "relation"
    }

    fn on_each_json_request(
        &self,
        base_col: &dyn JsonCollection<S>,
        input: Value,
        ctx: &AnySet,
    ) -> Option<Result<Box<(dyn SelectOneJsonFragment<S> + 'static)>, String>> {
        let input = serde_json::from_value::<HashMap<String, Value>>(input).ok()?;

        let base = base_col.table_name().to_case(Case::Snake);
        let spec = self.clone().spec(self.from.clone());

        // make sure the base collection have the said relation
        let rels = self
            .get_entry(&ctx)
            .entries
            .iter()
            .filter(|e| e.from == base);

        let mut not_related = Vec::default();

        let s = input
            .into_iter()
            .filter_map(|(to, input)| {
                // make sure base collection is related to `to`
                if rels.clone().any(|rel| rel.to == to).not() {
                    not_related.push(format!("{base} is not related to {to}",))
                }

                let specr = spec.on_each_select_one_request(input);

                // propegate all errors
                // todo: for now I'm displaying last error only! how to make this better?
                match specr {
                    Ok(s) => {
                        return Some((to, s));
                    }
                    Err(err) => {
                        not_related
                            .push(format!("invalid input for relation `{base}->{to}`: {err}",));
                        return None;
                    }
                };
            })
            .collect::<Vec<_>>();

        if not_related.is_empty().not() {
            return Some(Err(not_related.last().unwrap().clone()));
        }

        return Some(Ok(Box::new(s)));
    }
}
