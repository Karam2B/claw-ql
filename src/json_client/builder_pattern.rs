use super::DynamicLinkRT;
use super::{JsonClient, JsonCollection};
use crate::QueryBuilder;
use crate::builder_pattern::{AddCollection, AddLink, Finish, InitializeContext};
use crate::json_client::{DynamicLink, DynamicLinkBT};
use convert_case::{Case, Casing};
use sqlx::{Database, Pool};
use std::any::Any;
use std::collections::HashMap;
use std::io::Cursor;
use std::sync::Arc;

#[allow(non_camel_case_types)]
pub struct to_json_client<S: Database>(pub Pool<S>);

type ThisContext<S> = JsonClientBuilding<S>;

pub struct JsonClientBuilding<S: Database> {
    pub(super) collections: HashMap<String, Arc<dyn JsonCollection<S>>>,
    pub(super) links: HashMap<Vec<&'static str>, Box<dyn DynamicLinkBT<S>>>,
    pub(super) flex_ctx: Vec<Box<dyn Any>>,
    pub(super) db: Pool<S>,
}

impl<S> InitializeContext for to_json_client<S>
where
    S: Database,
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
    S: Database + QueryBuilder,
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
                "collections are globally unique, the identifier {} detected twice",
                next.table_name()
            )
        }
        ctx
    }
}

fn is_first_a_subset_of_second(entry: &Vec<&'static str>, key: &Vec<&'static str>) -> bool {
    let s = &key[..entry.len()];
    if entry == s { true } else { false }
}

impl<N, S> AddLink<N> for to_json_client<S>
where
    S: QueryBuilder,
    N: DynamicLink<S> + Clone,
{
    type This = to_json_client<S>;
    type Context = ThisContext<S>;
    type NextContext = ThisContext<S>;
    #[track_caller]
    fn build_component(next: &N, mut ctx: Self::Context) -> Self::NextContext {
        let next = next.clone();
        next.buildtime_extend(&mut ctx.flex_ctx);
        let entry = next.json_entry();

        for key in ctx.links.keys() {
            if is_first_a_subset_of_second(&entry, key) {
                panic!("{:?} is a subset of {:?}", entry, key);
            }
        }

        let exist = ctx.links.insert(entry, Box::new(next));
        if exist.is_some() {
            panic!(
                "internal bug in '{}:{}': should not exist if the subsetting is sound",
                file!(),
                line!(),
            )
        }
        ctx
    }
}

impl<S> Finish for to_json_client<S>
where
    S: QueryBuilder,
{
    type Context = ThisContext<S>;
    type Result = Result<JsonClient<S>, String>;
    #[track_caller]
    fn build_component(ctx: Self::Context) -> Self::Result {
        let mut vecc = HashMap::new();
        for (key, e) in ctx.links {
            let new = e.finish(&ctx.flex_ctx)?;
            vecc.insert(key, new);
        }
        Ok(JsonClient {
            collections: ctx.collections,
            links: vecc,
            db: ctx.db,
        })
    }
}
