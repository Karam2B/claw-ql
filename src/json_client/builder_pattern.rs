use super::{DynamicLink, DynamicLinkTraitObject};
use super::{JsonClient, JsonCollection};
use crate::QueryBuilder;
use crate::builder_pattern::{AddCollection, AddLink, BuildContext, Finish};
use convert_case::{Case, Casing};
use sqlx::{Database, Pool};
use std::ops::Not;
use std::sync::Arc;

#[allow(non_camel_case_types)]
pub struct to_json_client<S: Database>(pub Pool<S>);

impl<S> BuildContext for to_json_client<S>
where
    S: Database,
{
    type Context = JsonClient<S>;
    fn init_context(&self) -> Self::Context {
        JsonClient {
            collections: Default::default(),
            links: Default::default(),
            any_set: Default::default(),
            dynamic_errors: Default::default(),
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

        if ret.is_some() {
            ctx.dynamic_errors.push(format!(
                "collections are globally unique, the identifier {} detected twice",
                next.table_name()
            ))
        }
    }
}

impl<T, N, S> AddLink<T, N> for to_json_client<S>
where
    S: Database + QueryBuilder,
    N: DynamicLink<S> + 'static + Send + Sync + Clone,
{
    fn add_link(next: &N, ctx: &mut Self::Context) {
        let amut = Arc::get_mut(&mut ctx.any_set).expect("one at a time");
        if amut.get::<N::Entry>().is_none() {
            amut.set(N::init_entry());
        }
        let entry = amut.get_mut::<N::Entry>().unwrap();
        let next = next.clone();
        next.on_register(entry);
        let name = next.json_entry();
        ctx.links.insert(name, Arc::new(next));
    }
}

impl<C, S> Finish<C> for to_json_client<S>
where
    S: Database,
{
    type Result = Result<JsonClient<S>, String>;
    fn finish(self, ctx: Self::Context) -> Self::Result {
        ctx.links
            .values()
            .try_for_each(|e| e.on_finish(&ctx.any_set))?;

        if ctx.dynamic_errors.is_empty().not() {
            let err = ctx.dynamic_errors.join(", ");
            return Err(err);
        }

        Ok(ctx)
    }
}
