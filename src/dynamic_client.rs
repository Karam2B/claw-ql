use crate::any_set::AnySet;
use build_mode::{maxmimal, minimal};
use collections::Collections;
use json_client::JsonCollection;
use std::marker::PhantomData;

use crate::{
    QueryBuilder,
    build_tuple::{BuildTuple, TupleLastItem},
    collections::{Collection, OnMigrate},
    operations::{LinkData, Relation, select_one::SelectOneFragment},
};

pub struct DynamicClient<BuildMode: BuildModeBehaviour, Collections, Links, Filters, DB> {
    pub collections: Collections,
    pub links: Links,
    pub filters: Filters,
    pub link_context: AnySet,
    pub context: BuildMode::Context,
    pub build_mode: BuildMode,
    pub _pd: PhantomData<DB>,
}

impl Default for DynamicClient<minimal, (), (), (), ()> {
    fn default() -> Self {
        DynamicClient {
            _pd: PhantomData,
            context: Default::default(),
            links: (),
            collections: (),
            filters: (),
            build_mode: minimal,
            link_context: AnySet::default(),
        }
    }
}

impl<B: BuildModeBehaviour> DynamicClient<B, (), (), (), ()> {
    pub fn build_mode<BM: BuildModeBehaviour>(
        self,
        build_mode: BM,
    ) -> DynamicClient<BM, (), (), (), ()>
    where
        BM::Context: Default,
    {
        let context = BM::Context::default();
        DynamicClient {
            collections: self.collections,
            links: self.links,
            context,
            _pd: PhantomData,
            link_context: self.link_context,
            build_mode,
            filters: self.filters,
        }
    }
    pub fn infer_db<S>(self) -> DynamicClient<B, (), (), (), S>
    where
        S: QueryBuilder,
    {
        DynamicClient {
            _pd: PhantomData,
            links: self.links,
            collections: self.collections,
            context: self.context,
            link_context: self.link_context,
            build_mode: self.build_mode,
            filters: self.filters,
        }
    }
}

pub trait BuildModeBehaviour {
    type Context;
}

#[allow(non_camel_case_types)]
pub mod build_mode {
    pub struct minimal;
    pub struct maxmimal;
    pub struct json_client_bm;
}

impl BuildModeBehaviour for build_mode::minimal {
    type Context = ();
}

impl BuildModeBehaviour for build_mode::maxmimal {
    type Context = ();
}

impl BuildModeBehaviour for build_mode::json_client_bm {
    type Context = ();
}

#[cfg(feature = "beta")]
impl<B: BuildModeBehaviour> DynamicClient<B, (), (), (), ()> {
    pub fn catch_errors_early(self) -> DynamicClient<maxmimal, (), (), (), ()> {
        DynamicClient {
            collections: self.collections,
            links: self.links,
            context: (),
            _pd: PhantomData,
            link_context: self.link_context,
            build_mode: maxmimal,
            filters: self.filters,
        }
    }
}



pub mod migrate {
    use sqlx::Executor;

    use crate::{QueryBuilder, collections::OnMigrate};

    use super::{BuildModeBehaviour, DynamicClient, collections::Collections};

    impl<B, C, L, F, D> DynamicClient<B, C, L, F, D>
    where
        C: Collections<D>,
        D: QueryBuilder,
        L: OnMigrate<D>,
        B: BuildModeBehaviour,
    {
        pub async fn migrate(&self, exec: impl for<'e> Executor<'e, Database = D> + Clone) {
            self.collections.migrate(exec.clone()).await;
            self.links.custom_migration(exec).await;
        }
    }
}

pub mod collections {
    use super::json_client::JsonCollection;
    use crate::execute::Execute;
    use crate::{
        QueryBuilder, prelude::macro_relation::CreateTableSt, statements::create_table_st::header,
    };
    use paste::paste;
    use sqlx::Database;
    use sqlx::Executor;
    use std::sync::Arc;

    pub trait Collections<DB> {
        fn into_dynamic_collections(self) -> Vec<Arc<dyn JsonCollection<DB>>>;
        fn migrate(
            &self,
            exec: impl for<'e> Executor<'e, Database = DB> + Clone + Send,
        ) -> impl Future<Output = ()> + Send;
    }

    macro_rules! implt {
        ($([$ty:ident, $part:literal]),*) => {
    #[allow(unused)]
    impl
        <S, $($ty,)* >
    Collections<S>
    for
        ($($ty,)*)
    where
        S: QueryBuilder<Output = <S as Database>::Arguments<'static>>,
        S::Context1: Send + Sync,
        S::Fragment: Send + Sync,
        $($ty: JsonCollection<S> + Send + Sync,)*
    {
        fn into_dynamic_collections(self) -> Vec<Arc<dyn JsonCollection<S>>> {
            vec![$(
             Arc::new(paste!(self.$part)),
             )*]
        }
        async fn migrate(&self, exec: impl for<'e> Executor<'e, Database = S> + Clone + Send) {
            $(
            let tb = paste!(self.$part).table_name();
            let mut st = CreateTableSt::init(header::create, tb);
            paste!(self.$part).on_create_table(&mut st);
            st.execute(exec.clone()).await;
            )*
        }
    }
        }
        } // end of macro_rules

    implt!();
    implt!([R0, 0]);
    implt!([R0, 0], [R1, 1]);
    implt!([R0, 0], [R1, 1], [R2, 2]);
}

pub mod json_client {
    use super::{
        BuildModeBehaviour, DynamicClient,
        build_mode::{json_client_bm, minimal},
        collections::Collections,
    };
    use crate::{
        QueryBuilder,
        any_set::AnySet,
        collections::Collection,
        links::group_by::{DynamicLink, DynamicLinkAssociatedEntry},
        prelude::macro_relation::CreateTableSt,
    };
    use serde::Serialize;
    use sqlx::{Database, Pool};
    use std::{collections::HashMap, marker::PhantomData, sync::Arc};

    pub struct JC<S>(HashMap<&'static str, Arc<dyn JsonCollection<S>>>);
    pub struct JL<S>(Vec<Arc<dyn JsonLink<S>>>);
    pub struct JF<S>(HashMap<&'static str, PhantomData<S>>);

    pub struct JsonClient<S: Database> {
        pub collections: HashMap<&'static str, Arc<dyn JsonCollection<S>>>,
        pub links: Vec<Arc<dyn JsonLink<S>>>,
        pub db: Pool<S>,
    }

    pub trait JsonCollection<S>: 'static {
        fn from_row_scoped(&self, row: &S::Row) -> serde_json::Value
        where
            S: Database;
        fn table_name(&self) -> &'static str;
        fn on_create_table(&self, st: &mut CreateTableSt<S>)
        where
            S: QueryBuilder;
    }

    impl<S, T> JsonCollection<S> for T
    where
        S: QueryBuilder,
        T: Collection<S> + 'static,
        T::Yeild: Serialize,
    {
        fn on_create_table(&self, st: &mut CreateTableSt<S>) {
            self.on_migrate(st)
        }
        fn table_name(&self) -> &'static str {
            self.table_name()
        }
        #[inline]
        fn from_row_scoped(&self, row: &S::Row) -> serde_json::Value
        where
            S: Database,
        {
            let row = <Self as Collection<S>>::from_row_scoped(self, row);
            serde_json::to_value(row).unwrap()
        }
    }

    pub trait JsonLink<S>: DynamicLink<S> {
        fn on_each_json_request(&self, link_ctx: AnySet, base_col: &dyn JsonCollection<S>);
    }

    impl<S> DynamicClient<minimal, (), (), (), S> {
        pub fn to_build_json_client(self) -> DynamicClient<json_client_bm, JC<S>, JL<S>, JF<S>, S> {
            DynamicClient {
                collections: JC(Default::default()),
                links: JL(Default::default()),
                context: self.context,
                _pd: PhantomData,
                link_context: self.link_context,
                build_mode: json_client_bm,
                filters: JF(Default::default()),
            }
        }
    }

    impl<S> DynamicClient<json_client_bm, JC<S>, JL<S>, JF<S>, S>
    where
        S: QueryBuilder,
    {
        pub fn add_link<L>(mut self, link: L) -> Self
        where
            L: JsonLink<S> + DynamicLinkAssociatedEntry + 'static,
        {
            if let Some(entry) = link.register_entry_in_context() {
                self.link_context.set(entry);
            }
            self.links.0.push(Arc::new(link));
            Self { ..self }
        }
        pub fn add_collection<C>(mut self, collection: C) -> Self
        where
            C: JsonCollection<S>,
        {
            let table_name = collection.table_name();
            self.collections.0.insert(table_name, Arc::new(collection));
            Self { ..self }
        }
        pub fn finish(self, pool: Pool<S>) -> Result<JsonClient<S>, String> {
            self.links
                .0
                .iter()
                .try_for_each(|e| e.on_finish(&self.link_context))?;

            Ok(JsonClient {
                collections: self.collections.0,
                db: pool,
                links: self.links.0
            })
        }
    }

    impl<B, C, L, F, S> DynamicClient<B, C, L, F, S>
    where
        B: BuildModeBehaviour,
        C: Collections<S>,
        S: QueryBuilder,
    {
        pub fn create_json_client(self, pool: Pool<S>) -> JsonClient<S> {
            let collections = self
                .collections
                .into_dynamic_collections()
                .into_iter()
                .map(|e| (e.table_name(), e))
                .collect();

            let links = Default::default();

            JsonClient {
                collections,
                links,
                db: pool,
            }
        }
    }
}



#[cfg(feature = "beta")]
impl<C, L, F, S> DynamicClient<maxmimal, C, L, F, S>
where
    C: BuildTuple,
    L: BuildTuple,
    F: BuildTuple,
    S: QueryBuilder,
{
    pub fn add_link<Link>(self, link: Link) -> DynamicClient<maxmimal, C, L::Bigger<Link>, F, S> {
        DynamicClient {
            collections: self.collections,
            links: self.links.into_bigger(link),
            context: self.context,
            _pd: PhantomData,
            link_context: self.link_context,
            build_mode: self.build_mode,
            filters: self.filters,
        }
    }
    pub fn add_relation<From, To>(
        self,
        relation: Relation<From, To>,
    ) -> DynamicClient<maxmimal, C, L::Bigger<<Relation<From, To> as LinkData<From>>::Spec>, F, S>
    where
        From: Clone,
        Relation<From, To>: LinkData<From, Spec: OnMigrate<S>>,
    {
        let imove = relation.from.clone();
        DynamicClient {
            _pd: PhantomData,
            links: self.links.into_bigger(relation.spec(imove)),
            context: self.context,
            collections: self.collections,
            link_context: self.link_context,
            build_mode: self.build_mode,
            filters: self.filters,
        }
    }
    pub fn add_collection<N>(self, collection: N) -> DynamicClient<maxmimal, C::Bigger<N>, L, F, S>
    where
        C::Bigger<N>: Collections<S>,
        N: Collection<S> + JsonCollection<S> + Send + Sync + 'static,
    {
        DynamicClient {
            _pd: PhantomData,
            links: self.links,
            context: self.context,
            collections: self.collections.into_bigger(collection),
            link_context: self.link_context,
            build_mode: self.build_mode,
            filters: self.filters,
        }
    }
}

#[cfg(feature = "beta")]
impl<C, L, F, DB> DynamicClient<minimal, C, L, F, DB>
where
    C: BuildTuple,
    L: BuildTuple,
    F: BuildTuple,
    DB: QueryBuilder,
{
    pub fn add_collection<N>(self, collection: N) -> DynamicClient<minimal, C::Bigger<N>, L, F, DB>
    where
        N: Collection<DB>,
    {
        DynamicClient {
            _pd: PhantomData,
            links: self.links,
            context: self.context,
            collections: self.collections.into_bigger(collection),
            link_context: self.link_context,
            build_mode: self.build_mode,
            filters: self.filters,
        }
    }
    // pub fn add_relation<From, To>(
    //     self,
    //     rel: Relation<From, To>
    // ) -> Schema<
    //     minimal,
    //     C,
    //     <L::Bigger<(From, Relation<From, To>)> as BuildTuple>::Bigger<(To, Relation<To, From>)>,
    //     F,
    //     DB,
    // >
    // where
    //     Relation<From, To>: LinkData<From>,
    //     Relation<To, From>: LinkData<To>,
    //     L::Bigger<(From, Relation<From, To>)>: BuildTuple,
    // {
    //     Schema {
    //         _pd: PhantomData,
    //         links: self.links.into_bigger(,
    //         collections: self.collections,
    //     }
    // }
    // pub fn add_link<Base, Link>(self, link: (Base, Link)) -> Schema<minimal, C, L::Bigger<(Base, Link)>, F, DB>
    // where
    //     Link: LinkData<Base>,
    // {
    //     Schema {
    //         _pd: PhantomData,
    //         links: self.links.into_bigger(link),
    //         collections: self.collections,
    //     }
    // }
    pub fn last_rel_is_crud<B1, L1, B2, L2>(self) -> Self
    where
        L: TupleLastItem<2, Last = ((B1, L1), (B2, L2))>,
        L1: LinkData<B1, Spec: SelectOneFragment<DB>>,
    {
        self
    }
    pub fn last_link_is_crud<Base, Link>(self) -> Self
    where
        L: TupleLastItem<1, Last = (Base, Link)>,
        Link: LinkData<Base, Spec: SelectOneFragment<DB>>,
    {
        self
    }
    pub fn add_filter<N>(self, filter: N) -> DynamicClient<minimal, C, L, F::Bigger<N>, DB> {
        DynamicClient {
            _pd: PhantomData,
            links: self.links,
            context: self.context,
            collections: self.collections,
            link_context: self.link_context,
            filters: self.filters.into_bigger(filter),
            build_mode: self.build_mode,
        }
    }
}
