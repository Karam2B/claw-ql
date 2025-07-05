use super::DynamicLinkRT;
use super::{JsonClient, JsonCollection};
use crate::QueryBuilder;
use crate::builder_pattern::{BuildMutStep, collection, link};
use crate::json_client::{DynamicLinkBT, DynamicLinkBTDyn, JsonSelector};
use convert_case::{Case, Casing};
use sqlx::{Database, Pool};
use std::any::Any;
use std::collections::HashMap;
use std::io::Cursor;
use std::sync::Arc;

pub struct JsonClientBuilding<S: Database> {
    pub(super) collections: HashMap<String, Arc<dyn JsonCollection<S>>>,
    pub(super) links: Vec<Box<dyn DynamicLinkBTDyn<S>>>,
    pub(super) flex_ctx: Vec<Box<dyn Any>>,
    pub(super) db: Pool<S>,
}

impl<N, S> BuildMutStep<collection, N> for JsonClientBuilding<S>
where
    S: QueryBuilder,
    N: JsonCollection<S> + Clone,
{
    fn build_step(&mut self, step: &N) {
        let ret = self.collections.insert(
            step.table_name().to_case(Case::Snake),
            Arc::new(step.clone()),
        );
        if ret.is_some() {
            panic!(
                "collections are globally unique, the identifier {} was used twice",
                step.table_name()
            )
        }
    }
}

fn is_first_a_subset_of_second(entry: &JsonSelector, key: &JsonSelector) -> bool {
    if entry.collection != key.collection {
        return false;
    }
    let s = &key.body[..entry.body.len()];
    if entry.body == s { true } else { false }
}

impl<N, S> BuildMutStep<link, N> for JsonClientBuilding<S>
where
    S: QueryBuilder,
    N: DynamicLinkBT<S> + Clone + 'static,
{
    fn build_step(&mut self, step: &N) {
        let next = step.clone();

        let buildtime_meta = next.buildtime_meta();
        self.flex_ctx.push(Box::new(buildtime_meta));

        let mut more = next.push_more();
        while more.is_some() {
            let more_inner = more.unwrap();

            let buildtime_meta = more_inner.buildtime_meta();
            self.flex_ctx.push(Box::new(buildtime_meta));

            more = more_inner.push_more();

            self.links.push(more_inner);
        }

        self.links.push(Box::new(next));
    }
}

impl<S: Database> JsonClientBuilding<S> {
    #[track_caller]
    pub fn unwrap(self) -> JsonClient<S> {
        let mut links = HashMap::new();
        let mut all_json_selector = Vec::new();
        for dynlink_bt_dyn in self.links {
            let dynlink_rt = dynlink_bt_dyn.finish_building(&self.flex_ctx).unwrap();
            let json_selector = dynlink_rt.json_selector();
            for json_selector2 in all_json_selector.iter() {
                if is_first_a_subset_of_second(&json_selector, json_selector2) {
                    panic!("{:?} is a subset of {:?}", json_selector, json_selector2)
                }
            }

            all_json_selector.push(json_selector.clone());

            let out = links.insert(json_selector, dynlink_rt);

            if out.is_some() {
                panic!("internal bug: should not exist if the subsetting is sound",)
            }
        }

        JsonClient {
            collections: self.collections,
            db: self.db,
            links,
        }
    }
}
