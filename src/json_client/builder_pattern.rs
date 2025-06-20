use super::DynamicLinkTraitObject;
use super::{JsonClient, JsonCollection};
use crate::QueryBuilder;
use crate::builder_pattern::{AddCollection, AddLink, BuildContext, Finish};
use crate::json_client::{JsonClientBuilder, JsonClientBuilderDyn};
use convert_case::{Case, Casing};
use sqlx::{Database, Pool};
use std::any::Any;
use std::collections::HashMap;
use std::sync::Arc;

#[allow(non_camel_case_types)]
pub struct to_json_client<S: Database>(pub Pool<S>);

pub struct JsonClientBuilding<S: Database> {
    pub(crate) collections: HashMap<String, Arc<dyn JsonCollection<S>>>,
    pub(crate) links: HashMap<
        Vec<&'static str>,
        (
            Box<dyn JsonClientBuilderDyn>,
            Arc<dyn DynamicLinkTraitObject<S>>,
        ),
    >,
    pub(crate) ctx: Vec<Box<dyn Any>>,
    pub(crate) finishes: Vec<Box<dyn JsonClientBuilderDyn>>,
    pub(crate) db: Pool<S>,
}

impl<S> BuildContext for to_json_client<S>
where
    S: Database,
{
    type Context = JsonClientBuilding<S>;
    fn init_context(&self) -> Self::Context {
        JsonClientBuilding {
            collections: Default::default(),
            links: Default::default(),
            ctx: Default::default(),
            finishes: Default::default(),
            db: self.0.clone(),
        }
    }
}

impl<T, N, S> AddCollection<T, N> for to_json_client<S>
where
    S: Database + QueryBuilder,
    N: JsonCollection<S> + Clone,
{
    fn add_col(next: &N, ctx: &mut Self::Context) {
        let ret = ctx.collections.insert(
            next.table_name().to_case(Case::Snake),
            Arc::new(next.clone()),
        );

        // if ret.is_some() {
        //     ctx.dynamic_errors.push(format!(
        //         "collections are globally unique, the identifier {} detected twice",
        //         next.table_name()
        //     ))
        // }
    }
}

impl<T, N, S> AddLink<T, N> for to_json_client<S>
where
    S: Database + QueryBuilder,
    N: DynamicLinkTraitObject<S> + 'static + Send + Sync + Clone + JsonClientBuilder,
{
    fn add_link(next: &N, ctx: &mut Self::Context) {
        let build_entry = next.init();
        ctx.ctx.push(Box::new(build_entry));
        let next = next.clone();
        let name = next.json_entry();
        ctx.links.insert(name, ((), Arc::new(next)));
    }
}

impl<C, S> Finish<C> for to_json_client<S>
where
    S: Database,
{
    type Result = Result<JsonClient<S>, String>;
    fn finish(self, ctx: Self::Context) -> Self::Result {
        // ctx.ctx
        let mut vecc = HashMap::new();
        for (key, e) in ctx.links {
            let new = e.0.finish(&ctx.ctx)?;
            vecc.insert(key, (e.1, new));
        }

        Ok(JsonClient {
            collections: ctx.collections,
            links: vecc,
            db: ctx.db
        })
    }
}
