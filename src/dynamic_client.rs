use crate::{
    any_set::AnySet,
    links::{DynamicLink, LinkData, relation::Relation},
};
use build_mode::{maxmimal, minimal};
use collections::Collections;
use json_client::JsonCollection;
use sqlx::Executor;
use std::marker::PhantomData;

use crate::{
    QueryBuilder,
    build_tuple::BuildTuple,
    collections::{Collection, OnMigrate},
};

pub struct DynamicClient<BuildMode: BuildModeBehaviour, Collections, Links, Filters, DB> {
    pub collections: Collections,
    pub links: Links,
    pub filters: Filters,
    pub build_context: AnySet,
    pub build_mode: BuildMode,
    pub _pd: PhantomData<DB>,
}

impl Default for DynamicClient<minimal, (), (), (), ()> {
    fn default() -> Self {
        DynamicClient {
            _pd: PhantomData,
            links: (),
            collections: (),
            filters: (),
            build_mode: minimal,
            build_context: AnySet::default(),
        }
    }
}

impl<B: BuildModeBehaviour> DynamicClient<B, (), (), (), ()> {
    pub fn build_mode<BM: BuildModeBehaviour>(
        self,
        build_mode: BM,
    ) -> DynamicClient<BM, (), (), (), ()> {
        DynamicClient {
            collections: self.collections,
            links: self.links,
            _pd: PhantomData,
            build_context: self.build_context,
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
            build_context: self.build_context,
            build_mode: self.build_mode,
            filters: self.filters,
        }
    }
}

impl<B, C, L, F, D> DynamicClient<B, C, L, F, D>
where
    D: QueryBuilder,
    C: OnMigrate<D>,
    L: OnMigrate<D>,
    B: BuildModeBehaviour,
{
    pub async fn migrate(&self, exec: impl for<'e> Executor<'e, Database = D> + Clone) {
        self.collections.custom_migration(exec.clone()).await;
        self.links.custom_migration(exec).await;
    }
}

pub trait BuildModeBehaviour {
    // type Context;
}

#[allow(non_camel_case_types)]
pub mod build_mode {
    pub struct minimal;
    pub struct maxmimal;
    pub struct json_client_bm;
}

impl BuildModeBehaviour for build_mode::minimal {
    // type Context = ();
}

impl BuildModeBehaviour for build_mode::maxmimal {
    // type Context = ();
}

impl BuildModeBehaviour for build_mode::json_client_bm {
    // type Context = ();
}

#[cfg(feature = "beta")]
impl<B: BuildModeBehaviour> DynamicClient<B, (), (), (), ()> {
    pub fn catch_errors_early(self) -> DynamicClient<maxmimal, (), (), (), ()> {
        DynamicClient {
            collections: self.collections,
            links: self.links,
            _pd: PhantomData,
            build_context: self.build_context,
            build_mode: maxmimal,
            filters: self.filters,
        }
    }
}

pub mod links {
    use paste::paste;
    use std::{collections::HashMap, sync::Arc};

    use crate::{
        any_set::AnySet,
        links::{DynamicLink, DynamicLinkTraitObject},
    };

    pub trait Links<S> {
        fn on_finish(&self, build_ctx: &AnySet) -> Result<(), String>;
        fn into_dynamic_link_trait_objects(
            self,
        ) -> HashMap<&'static str, Arc<dyn DynamicLinkTraitObject<S>>>;
    }

    macro_rules! implt {
        ($([$ty:ident, $part:literal]),*) => {
    #[allow(unused)]
    impl <S, $($ty,)* >
    Links<S>
    for
        ($($ty,)*)
    where
        $($ty: DynamicLink<S> + Send + Sync + 'static,)*
    {

        fn on_finish(&self, build_ctx: &AnySet) -> Result<(), String> {
            $(
            paste!(self.$part.on_finish(build_ctx));
            )*

            Ok(())
        }
        fn into_dynamic_link_trait_objects(self) -> HashMap<&'static str,Arc<dyn DynamicLinkTraitObject<S>>> {
            let mut map: HashMap<_, Arc<dyn DynamicLinkTraitObject<S>>> = HashMap::new();
            $(
            map.insert(paste!(self.$part.json_entry()), paste!(Arc::new(self.$part)));
        )*
            map
        }
    }
        }
        } // end of macro_rules

    #[allow(unused)]
    impl<S, R0, R1> Links<S> for (R0, R1)
    where
        R0: DynamicLink<S> + Send + Sync + 'static,
        R1: DynamicLink<S> + Send + Sync + 'static,
    {
        fn on_finish(&self, build_ctx: &AnySet) -> Result<(), String> {
            self.0.on_finish(build_ctx);
            self.1.on_finish(build_ctx);

            Ok(())
        }
        fn into_dynamic_link_trait_objects(
            self,
        ) -> HashMap<&'static str, Arc<dyn DynamicLinkTraitObject<S>>> {
            let mut map: HashMap<_, Arc<dyn DynamicLinkTraitObject<S>>> = HashMap::new();
            map.insert(self.0.json_entry(), Arc::new(self.0));
            map.insert(self.1.json_entry(), Arc::new(self.1));
            map
        }
    }
    implt!();
    implt!([R0, 0]);
    implt!([R0, 0], [R1, 1], [R2, 2]);
}
pub mod collections {
    use super::json_client::{JC, JsonCollection};
    use crate::QueryBuilder;
    use paste::paste;
    use sqlx::Database;
    use std::sync::Arc;

    pub trait Collections<DB> {
        fn into_dynamic_collections(self) -> Vec<Arc<dyn JsonCollection<DB>>>;
    }

    impl<S> Collections<S> for JC<S> {
        fn into_dynamic_collections(self) -> Vec<Arc<dyn JsonCollection<S>>> {
            self.0.into_values().collect()
        }
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
        links::Links,
    };
    use crate::{
        QueryBuilder,
        any_set::AnySet,
        collections::{Collection, OnMigrate},
        execute::Execute,
        links::{DynamicLink, DynamicLinkTraitObject, LinkData, relation::Relation},
        operations::select_one::{SelectOneFragment, SelectOneOutput},
        prelude::{col, stmt::SelectSt},
    };
    use convert_case::{Case, Casing};
    use serde::{Deserialize, Serialize};
    use serde_json::{Map, Value};
    use sqlx::{ColumnIndex, Database, Decode, Encode, Executor, Pool, Row, prelude::Type};
    use std::{collections::HashMap, marker::PhantomData, ops::Not, pin::Pin, sync::Arc};

    pub struct JC<S>(pub(crate) HashMap<String, Arc<dyn JsonCollection<S>>>);
    pub struct JL<S>(pub(crate) HashMap<&'static str, Arc<dyn DynamicLinkTraitObject<S>>>);
    #[allow(unused)]
    pub struct JF<S>(pub(crate) HashMap<&'static str, PhantomData<S>>);

    pub struct JsonClient<S: Database> {
        pub(crate) collections: HashMap<String, Arc<dyn JsonCollection<S>>>,
        pub(crate) links: HashMap<&'static str, Arc<dyn DynamicLinkTraitObject<S>>>,
        pub(crate) link_context: AnySet,
        pub(crate) db: Pool<S>,
    }

    pub trait JsonCollection<S>: Send + Sync + 'static {
        fn on_select(&self, stmt: &mut SelectSt<S>)
        where
            S: QueryBuilder;
        fn from_row_scoped(&self, row: &S::Row) -> serde_json::Value
        where
            S: Database;
        fn table_name(&self) -> &'static str;
    }

    impl<S, T> JsonCollection<S> for T
    where
        S: QueryBuilder,
        T: Collection<S> + 'static,
        T::Yeild: Serialize,
    {
        #[inline]
        fn on_select(&self, stmt: &mut SelectSt<S>)
        where
            S: QueryBuilder,
        {
            self.on_select(stmt)
        }
        #[inline]
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

    pub trait SelectOneJsonFragment<S: QueryBuilder>: Send + Sync {
        fn on_select(&mut self, st: &mut SelectSt<S>);
        fn from_row(&mut self, row: &S::Row);
        fn sub_op<'this>(
            &'this mut self,
            pool: Pool<S>,
        ) -> Pin<Box<dyn Future<Output = ()> + Send + 'this>>;
        fn take(self: Box<Self>) -> serde_json::Value;
    }

    impl<S: QueryBuilder> SelectOneJsonFragment<S>
        for Vec<(String, Box<dyn SelectOneJsonFragment<S>>)>
    {
        fn on_select(&mut self, st: &mut SelectSt<S>) {
            self.iter_mut().for_each(|e| e.1.on_select(st))
        }

        fn from_row(&mut self, row: &<S>::Row) {
            self.iter_mut().for_each(|e| e.1.from_row(row))
        }

        fn sub_op<'this>(
            &'this mut self,
            pool: Pool<S>,
        ) -> Pin<Box<dyn Future<Output = ()> + Send + 'this>> {
            Box::pin(async move {
                for item in self.iter_mut() {
                    item.1.sub_op(pool.clone()).await
                }
            })
        }

        fn take(self: Box<Self>) -> serde_json::Value {
            let mut map = serde_json::Map::new();
            self.into_iter().for_each(|e| {
                map.insert(e.0, e.1.take());
            });
            map.into()
        }
    }

    impl<S: QueryBuilder, T> SelectOneJsonFragment<S> for (T, T::Inner)
    where
        T::Output: Serialize,
        T: SelectOneFragment<S>,
    {
        #[inline]
        fn on_select(&mut self, st: &mut SelectSt<S>) {
            self.0.on_select(&mut self.1, st)
        }

        #[inline]
        fn from_row(&mut self, row: &<S>::Row) {
            self.0.from_row(&mut self.1, row)
        }

        #[inline]
        fn sub_op<'this>(
            &'this mut self,
            pool: Pool<S>,
        ) -> Pin<Box<dyn Future<Output = ()> + Send + 'this>> {
            Box::pin(async { self.0.sub_op(&mut self.1, pool).await })
        }

        #[inline]
        fn take(self: Box<Self>) -> serde_json::Value {
            let taken = self.0.take(self.1);
            serde_json::to_value(taken).unwrap()
        }
    }

    impl<S> DynamicClient<minimal, (), (), (), S> {
        pub fn to_build_json_client(self) -> DynamicClient<json_client_bm, JC<S>, JL<S>, JF<S>, S> {
            DynamicClient {
                collections: JC(Default::default()),
                links: JL(Default::default()),
                _pd: PhantomData,
                build_context: self.build_context,
                build_mode: json_client_bm,
                filters: JF(Default::default()),
            }
        }
    }

    impl<S> DynamicClient<json_client_bm, JC<S>, JL<S>, JF<S>, S>
    where
        S: QueryBuilder,
    {
        pub fn add_relation<F, T>(mut self, link: Relation<F, T>) -> Self
        where
            F: 'static + Send + Sync,
            T: 'static + Send + Sync,
            Relation<F, T>: LinkData<F, Spec: OnMigrate<S>>,
            Relation<F, T>: DynamicLink<S>,
        {
            if self
                .build_context
                .get::<<Relation<F, T> as DynamicLink<S>>::Entry>()
                .is_none()
            {
                self.build_context
                    .set(<Relation<F, T> as DynamicLink<S>>::init_entry());
            }

            let entry = self
                .build_context
                .get_mut::<<Relation<F, T> as DynamicLink<S>>::Entry>()
                .unwrap();
            link.on_register(entry);
            let name = <Relation<F, T> as DynamicLink<S>>::json_entry();
            self.links.0.insert(name, Arc::new(link));
            self
        }

        #[track_caller]
        pub fn add_link<L>(mut self, link: L) -> Self
        where
            L: DynamicLink<S> + 'static + Send + Sync,
        {
            if self.build_context.get::<L::Entry>().is_none() {
                self.build_context.set(L::init_entry());
            }
            let entry = self.build_context.get_mut::<L::Entry>().unwrap();
            link.on_register(entry);
            Self { ..self }
        }
        pub fn add_collection<C>(mut self, collection: C) -> Self
        where
            C: JsonCollection<S>,
        {
            let table_name = collection.table_name();
            self.collections
                .0
                .insert(table_name.to_string(), Arc::new(collection));
            Self { ..self }
        }
        pub fn finish(self, pool: Pool<S>) -> Result<JsonClient<S>, String> {
            self.links
                .0
                .iter()
                .try_for_each(|e| e.1.on_finish(&self.build_context))?;

            Ok(JsonClient {
                collections: self.collections.0,
                db: pool,
                link_context: self.build_context,
                links: self.links.0,
            })
        }
    }

    impl<B, C, L, F, S> DynamicClient<B, C, L, F, S>
    where
        B: BuildModeBehaviour,
        C: Collections<S>,
        L: Links<S>,
        S: QueryBuilder,
    {
        pub fn create_json_client(self, pool: Pool<S>) -> Result<JsonClient<S>, String> {
            let collections = self
                .collections
                .into_dynamic_collections()
                .into_iter()
                .map(|e| (e.table_name().to_case(Case::Snake), e))
                .collect();

            self.links.on_finish(&self.build_context)?;

            let links = self.links.into_dynamic_link_trait_objects();

            Ok(JsonClient {
                collections,
                links,
                link_context: self.build_context,
                db: pool,
            })
        }
    }

    impl<S> JsonClient<S>
    where
        S: QueryBuilder<Output = <S as Database>::Arguments<'static>>,
        for<'c> &'c mut S::Connection: Executor<'c, Database = S>,
        for<'e> i64: Encode<'e, S> + Type<S> + Decode<'e, S>,
        for<'e> &'e str: ColumnIndex<S::Row>,
    {
        pub async fn select_one(&self, input: Value) -> Result<Value, String> {
            #[derive(Deserialize)]
            #[serde(deny_unknown_fields)]
            struct Input {
                pub collection: String,
                #[allow(unused)]
                #[serde(default)]
                pub filters: Map<String, Value>,
                #[serde(default)]
                pub links: Map<String, Value>,
            }

            let input: Input =
                serde_json::from_value(input).map_err(|e| format!("invalid input: {e:?}"))?;

            let c = self
                .collections
                .get(&input.collection)
                .ok_or("collection was not found")?;

            let mut st = SelectSt::init(c.table_name());

            let mut link_errors = Vec::default();

            let mut links = self
                .links
                .iter()
                .filter_map(|e| {
                    let name = e.1.json_entry();
                    let input = input.links.get(*e.0)?.clone();
                    let s =
                        e.1.on_each_json_request(c.as_ref(), input, &self.link_context);

                    match s {
                        Some(Ok(s)) => Some((name, s)),
                        None => None,
                        Some(Err(e)) => {
                            link_errors.push(e);
                            None
                        }
                    }
                })
                .collect::<Vec<_>>();

            if link_errors.is_empty().not() {
                return Err(format!("{link_errors:?}"));
            }

            #[rustfmt::skip]
            st.select(
                col("id").
                table(c.table_name()).
                alias("local_id")
            );

            c.on_select(&mut st);
            for link in links.iter_mut() {
                link.1.on_select(&mut st);
            }

            let mut res = st
                .fetch_one(&self.db, |r| {
                    let id: i64 = r.get("local_id");
                    let attr = c.from_row_scoped(&r);

                    for link in links.iter_mut() {
                        link.1.from_row(&r);
                    }

                    Ok(SelectOneOutput {
                        id,
                        attr,
                        links: HashMap::new(),
                    })
                })
                .await
                .unwrap();

            for link in links.iter_mut() {
                link.1.sub_op(self.db.clone()).await;
            }

            res.links = links.into_iter().map(|e| (e.0, e.1.take())).collect();

            Ok(serde_json::to_value(res).unwrap())
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
    pub fn add_link<Link>(mut self, link: Link) -> DynamicClient<maxmimal, C, L::Bigger<Link>, F, S>
    where
        Link: DynamicLink<S>,
    {
        if self.build_context.get::<Link::Entry>().is_none() {
            self.build_context.set(Link::init_entry());
        }
        let entry = self.build_context.get_mut::<Link::Entry>().unwrap();
        link.on_register(entry);
        DynamicClient {
            collections: self.collections,
            links: self.links.into_bigger(link),
            _pd: PhantomData,
            build_context: self.build_context,
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
            collections: self.collections,
            build_context: self.build_context,
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
            collections: self.collections.into_bigger(collection),
            build_context: self.build_context,
            build_mode: self.build_mode,
            filters: self.filters,
        }
    }
}

impl<C, L, F, S> DynamicClient<minimal, C, L, F, S>
where
    C: BuildTuple,
    L: BuildTuple,
    F: BuildTuple,
    S: QueryBuilder,
{
    pub fn add_collection<N>(self, collection: N) -> DynamicClient<minimal, C::Bigger<N>, L, F, S> {
        DynamicClient {
            _pd: PhantomData,
            links: self.links,
            collections: self.collections.into_bigger(collection),
            build_context: self.build_context,
            build_mode: self.build_mode,
            filters: self.filters,
        }
    }
    pub fn add_relation<From, To>(
        self,
        rel: Relation<From, To>,
    ) -> DynamicClient<minimal, C, L::Bigger<Relation<From, To>>, F, S> {
        // let false_if_existed = self.globals.insert(Link::json_entry());
        // if false_if_existed.not() {
        //     panic!("{} already exist as global!", Link::json_entry())
        // }
        // if self.build_context.get::<Link::Entry>().is_none() {
        //     self.build_context.set(Link::init_entry());
        // }
        // let entry = self.build_context.get_mut::<Link::Entry>().unwrap();
        // link.on_register(entry);
        DynamicClient {
            _pd: PhantomData,
            links: self.links.into_bigger(rel),
            collections: self.collections,
            build_context: self.build_context,
            build_mode: self.build_mode,
            filters: self.filters,
        }
    }
    pub fn add_link<Link>(mut self, link: Link) -> DynamicClient<minimal, C, L::Bigger<Link>, F, S>
    where
        Link: DynamicLink<S>,
    {
        if self.build_context.get::<Link::Entry>().is_none() {
            self.build_context.set(Link::init_entry());
        }
        let entry = self.build_context.get_mut::<Link::Entry>().unwrap();
        link.on_register(entry);
        DynamicClient {
            _pd: PhantomData,
            links: self.links.into_bigger(link),
            collections: self.collections,
            build_context: self.build_context,
            build_mode: self.build_mode,
            filters: self.filters,
        }
    }
}
