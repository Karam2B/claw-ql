use crate::{
    QueryBuilder,
    any_set::AnySet,
    collections::{Collection, OnMigrate},
    dynamic_client::json_client::{
        JsonCollection, SelectOneJsonFragment,
    },
    operations::{
        insert_one::InsertOneFragment,
        select_one::SelectOneFragment,
    },
};
use serde::{Deserialize, Serialize};
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
    async fn custom_migration<'e>(
        &self,
        exec: impl for<'q> Executor<'q, Database = S> + Clone,
    ) where
        S: QueryBuilder,
    {
        let relation = self.clone();
        let spec = relation.spec(self.from.clone());
        spec.custom_migration(exec).await;
    }
}

#[derive(Debug)]
pub struct RelationEntry {
    pub from: String,
    pub to: String,
    /// like 'many_to_many<..>', 'one_to_many<..>', etc. there is no way to have this unique for now.
    pub ty: String,
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
                      + InsertOneFragment<S, Output: Serialize>
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
        let from = self.from.table_name().to_lowercase().to_string();
        let to = self.to.table_name().to_lowercase().to_string();
        let spec = self.clone().spec(self.from.clone());
        let ty = spec.global_ident().to_string();
        let entry = RelationEntry { from, to, ty };
        #[cfg(feature = "trace")]
        tracing::debug!("registering entry {:?}", entry);
        entries.push(entry)
    }

    // type Entry = <Self as LinkData<F>>::Spec::DynamicSpec
    type Entry = Vec<RelationEntry>;

    fn on_finish(
        &self,
        _build_ctx: &AnySet,
    ) -> Result<(), String> {
        Ok(())
    }

    fn json_entry() -> &'static str {
        "relation"
    }

    fn on_each_json_request(
        &self,
        base_col: &dyn JsonCollection<S>,
        input: Value,
        ctx: &Self::Entry,
    ) -> Option<
        Result<
            Box<(dyn SelectOneJsonFragment<S> + 'static)>,
            String,
        >,
    > {
        // make sure the two the two collections are related
        #[derive(Deserialize)]
        struct Input(HashMap<String, Value>);

        let base = base_col.table_name().to_lowercase();
        let rels = ctx.iter().filter(|e| e.from == base);
        let rels = rels.collect::<Vec<_>>();

        let input =
            serde_json::from_value::<Input>(input).ok()?.0;

        let mut not_related = Vec::default();

        let s = input
            .into_iter()
            .filter_map(|(to, input)| {
                // panic!("{rels:?}");
                if rels.iter().any(|rel| rel.to == to).not() {
                    not_related.push(format!("{base} is not related to {to}",))
                }

                let spec = self.clone().spec(self.from.clone());
                let specr: &dyn DynamicLinkForRelation<S> = &spec;
                let specr = specr.on_each_select_one_request(input);
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
            return Some(Err(format!("{not_related:?}")));
        }

        return Some(Ok(Box::new(s)));
    }
}
