use super::DynamicLinkRT;
use super::{JsonClient, JsonCollection};
use crate::QueryBuilder;
use crate::builder_pattern::{AddCollection, AddLink, Finish, InitializeContext};
use crate::json_client::DynamicLinkBT;
use convert_case::{Case, Casing};
use sqlx::{Database, Pool};
use std::any::Any;
use std::collections::HashMap;
use std::sync::Arc;

#[allow(non_camel_case_types)]
pub struct to_json_client<S: Database>(pub Pool<S>);

type ThisContext<S> = JsonClientBuilding<S>;

pub struct JsonClientBuilding<S: Database> {
    pub(super) collections: HashMap<String, Arc<dyn JsonCollection<S>>>,
    pub(super) links: HashMap<Vec<&'static str>, Arc<dyn DynamicLinkBT<S>>>,
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

impl<N, S> AddLink<N> for to_json_client<S>
where
    S: QueryBuilder,
{
    type This = to_json_client<S>;
    type Context = ThisContext<S>;
    type NextContext = ThisContext<S>;
    #[track_caller]
    fn build_component(link: &N, ctx: Self::Context) -> Self::NextContext {
        //         let build_entry = next.init();
        //         ctx.ctx.push(Box::new(build_entry));
        //         let next = next.clone();
        //         let name = next.json_entry();
        //         ctx.links.insert(name, ((), Arc::new(next)));
        todo!();
        ctx
    }
}

impl<S> Finish for to_json_client<S>
where
    S: QueryBuilder,
{
    type Context = ThisContext<S>;
    type Result = JsonClient<S>;
    #[track_caller]
    fn build_component(ctx: Self::Context) -> Self::Result {
        todo!()
        //         let mut vecc = HashMap::new();
        //         for (key, e) in ctx.links {
        //             let new = e.0.finish(&ctx.ctx)?;
        //             vecc.insert(key, (e.1, new));
        //         }
        //
        //         Ok(JsonClient {
        //             collections: ctx.collections,
        //             links: vecc,
        //             db: ctx.db,
        //         })
    }
}
