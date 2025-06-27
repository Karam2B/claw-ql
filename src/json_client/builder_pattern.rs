use super::DynamicLinkRT;
use super::{JsonClient, JsonCollection};
use crate::QueryBuilder;
use crate::builder_pattern::{AddCollection, AddLink, Finish, InitializeContext};
use crate::json_client::{DynamicLinkBT, DynamicLinkBTDyn, JsonSelector};
use convert_case::{Case, Casing};
use sqlx::{Database, Pool};
use std::any::Any;
use std::collections::HashMap;
use std::io::Cursor;
use std::sync::Arc;

#[allow(non_camel_case_types)]
pub struct to_json_client<S: QueryBuilder>(pub Pool<S>);

type ThisContext<S> = JsonClientBuilding<S>;

pub struct JsonClientBuilding<S: Database> {
    pub(super) collections: HashMap<String, Arc<dyn JsonCollection<S>>>,
    pub(super) links: Vec<Box<dyn DynamicLinkBTDyn<S>>>,
    pub(super) flex_ctx: Vec<Box<dyn Any>>,
    pub(super) db: Pool<S>,
}

impl<S> InitializeContext for to_json_client<S>
where
    S: QueryBuilder,
{
    type Context = JsonClientBuilding<S>;
    fn initialize_context(self) -> Self::Context {
        JsonClientBuilding {
            collections: Default::default(),
            links: Default::default(),
            flex_ctx: Default::default(),
            db: self.0.clone(),
        }
    }
}

impl<N, S> AddCollection<N> for to_json_client<S>
where
    S: QueryBuilder,
    N: JsonCollection<S> + Clone,
{
    type This = to_json_client<S>;
    type Context = ThisContext<S>;
    type NextContext = ThisContext<S>;
    #[track_caller]
    fn build_component(next: &N, mut ctx: Self::Context) -> Self::NextContext {
        let ret = ctx.collections.insert(
            next.table_name().to_case(Case::Snake),
            Arc::new(next.clone()),
        );
        if ret.is_some() {
            panic!(
                "collections are globally unique, the identifier {} was used twice",
                next.table_name()
            )
        }
        ctx
    }
}

fn is_first_a_subset_of_second(entry: &JsonSelector, key: &JsonSelector) -> bool {
    if entry.collection != key.collection {
        return false;
    }
    let s = &key.body[..entry.body.len()];
    if entry.body == s { true } else { false }
}

impl<N, S> AddLink<N> for to_json_client<S>
where
    S: QueryBuilder,
    N: DynamicLinkBT<S> + Clone + 'static,
{
    type This = to_json_client<S>;
    type Context = ThisContext<S>;
    type NextContext = ThisContext<S>;
    #[track_caller]
    fn build_component(next: &N, mut ctx: Self::Context) -> Self::NextContext {
        let next = next.clone();

        let buildtime_meta = next.buildtime_meta();
        ctx.flex_ctx.push(Box::new(buildtime_meta));

        let mut more = next.push_more();
        while more.is_some() {
            let more_inner = more.unwrap();

            let buildtime_meta = more_inner.buildtime_meta();
            ctx.flex_ctx.push(Box::new(buildtime_meta));

            more = more_inner.push_more();

            ctx.links.push(more_inner);
        }

        ctx.links.push(Box::new(next));

        ctx
    }
}

impl<S> Finish for to_json_client<S>
where
    S: QueryBuilder,
{
    type Context = ThisContext<S>;
    type Result = Result<JsonClient<S>, String>;
    fn build_component(ctx: Self::Context) -> Self::Result {
        let mut links = HashMap::new();
        let mut all_json_selector = Vec::new();
        for dynlink_bt_dyn in ctx.links {
            let dynlink_rt = dynlink_bt_dyn.finish_building(&ctx.flex_ctx)?;
            let json_selector = dynlink_rt.json_selector();
            for json_selector2 in all_json_selector.iter() {
                if is_first_a_subset_of_second(&json_selector, json_selector2) {
                    return Err(format!(
                        "{:?} is a subset of {:?}",
                        json_selector, json_selector2
                    ));
                }
            }

            all_json_selector.push(json_selector.clone());

            let out = links.insert(json_selector, dynlink_rt);

            if out.is_some() {
                panic!("internal bug: should not exist if the subsetting is sound",)
            }
        }

        Ok(JsonClient {
            collections: ctx.collections,
            db: ctx.db,
            links,
        })
    }
}
