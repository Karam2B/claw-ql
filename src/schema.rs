use std::marker::PhantomData;

use build_mode::unopinionated;

use crate::{
    QueryBuilder,
    build_tuple::{BuildTuple, TupleLastItem},
    collections::Collection,
    operations::{LinkData, Relation, select_one::GetOneWorker},
};

#[allow(non_camel_case_types)]
pub mod build_mode {
    pub struct unopinionated;
}

pub struct Schema<BuildMode, Collections, Links, Filters, DB> {
    pub collections: Collections,
    pub _pd: PhantomData<(BuildMode, Collections, Links, Filters, DB)>,
}

impl<C, L, F, B> Schema<B, C, L, F, ()> {
    pub fn build_mode<BM>(self) -> Schema<BM, C, L, F, ()> {
        Schema {
            collections: self.collections,
            _pd: PhantomData,
        }
    }
    pub fn infer_db<DB>(self) -> Schema<B, C, L, F, DB>
    where
        DB: QueryBuilder,
    {
        Schema {
            _pd: PhantomData,
            collections: self.collections,
        }
    }
}

impl<C, L, F, DB> Schema<unopinionated, C, L, F, DB>
where
    C: BuildTuple,
    L: BuildTuple,
    F: BuildTuple,
    DB: QueryBuilder,
{
    pub fn add_collection<N>(self, collection: N) -> Schema<unopinionated, C::Bigger<N>, L, F, DB>
    where
        N: Collection<DB>,
    {
        Schema {
            _pd: PhantomData,
            collections: self.collections.into_bigger(collection),
        }
    }
    pub fn add_relation<From, To>(
        self,
    ) -> Schema<
        unopinionated,
        C,
        <L::Bigger<(From, Relation<From, To>)> as BuildTuple>::Bigger<(To, Relation<To, From>)>,
        F,
        DB,
    >
    where
        Relation<From, To>: LinkData<From>,
        Relation<To, From>: LinkData<To>,
        L::Bigger<(From, Relation<From, To>)>: BuildTuple,
    {
        Schema {
            _pd: PhantomData,
            collections: self.collections,
        }
    }
    pub fn add_link<Base, Link>(self) -> Schema<unopinionated, C, L::Bigger<(Base, Link)>, F, DB>
    where
        Link: LinkData<Base>,
    {
        Schema {
            _pd: PhantomData,
            collections: self.collections,
        }
    }
    pub fn last_rel_is_crud<B1, L1, B2, L2>(self) -> Self
    where
        L: TupleLastItem<2, Last = ((B1, L1), (B2, L2))>,
        L1: LinkData<B1, Spec: GetOneWorker<DB>>,
    {
        self
    }
    pub fn last_link_is_crud<Base, Link>(self) -> Self
    where
        L: TupleLastItem<1, Last = (Base, Link)>,
        Link: LinkData<Base, Spec: GetOneWorker<DB>>,
    {
        self
    }
    pub fn add_filter<N>(self) -> Schema<unopinionated, C, L, F::Bigger<N>, DB> {
        Schema {
            _pd: PhantomData,
            collections: self.collections,
        }
    }
}

pub mod migrate {
    use sqlx::Executor;

    use crate::{QueryBuilder, collections::Collection};

    use super::{Schema, collections::Collections};

    impl<B, C, L, F, D> Schema<B, C, L, F, D>
    where
        C: Collections<D>,
        D: QueryBuilder,
    {
        pub async fn migrate(&self, exec: impl for<'e> Executor<'e, Database = D>) {
            self.collections.migrate(exec).await;
        }
    }
}

pub mod collections {
    use std::sync::Arc;

    use sqlx::Executor;

    use crate::{
        QueryBuilder, prelude::macro_relation::CreateTableSt, statements::create_table_st::header,
    };

    use super::json_client::DynamicCollection;

    pub trait Collections<DB> {
        fn into_dynamic_collections(self) -> Vec<Arc<dyn DynamicCollection<DB>>>;
        async fn migrate(&self, exec: impl for<'e> Executor<'e, Database = DB>);
    }

    impl<DB> Collections<DB> for () {
        fn into_dynamic_collections(self) -> Vec<Arc<dyn DynamicCollection<DB>>> {
            Vec::new()
        }
    }
    impl<DB, T0> Collections<DB> for (T0,)
    where
        T0: DynamicCollection<DB>,
        DB: QueryBuilder,
    {
        fn into_dynamic_collections(self) -> Vec<Arc<dyn DynamicCollection<DB>>> {
            vec![Arc::new(self.0)]
        }
        async fn migrate(&self) {
            let tb = self.0.table_name();
            let mut st = CreateTableSt::init(header::create, tb);
            self.0.on_migrate(&mut st)
        }
    }

    impl<DB, T0, T1> Collections<DB> for (T0, T1)
    where
        T0: DynamicCollection<DB>,
        T1: DynamicCollection<DB>,
    {
        fn into_dynamic_collections(self) -> Vec<Arc<dyn DynamicCollection<DB>>> {
            vec![Arc::new(self.0), Arc::new(self.1)]
        }
    }
}

pub mod json_client {
    use std::{collections::HashMap, sync::Arc};

    use sqlx::{Database, Pool};

    use crate::{QueryBuilder, collections::Collection, prelude::macro_relation::CreateTableSt};

    use super::{Schema, collections::Collections};

    pub struct JsonClient<DB: Database> {
        collections: HashMap<&'static str, Arc<dyn DynamicCollection<DB>>>,
        db: Pool<DB>,
    }

    pub trait DynamicCollection<DB>: 'static {
        fn table_name(&self) -> &'static str;
        fn on_migrate(&self, st: &mut CreateTableSt<DB>)
        where
            DB: QueryBuilder;
    }
    impl<DB, T> DynamicCollection<DB> for T
    where
        DB: QueryBuilder,
        T: Collection<DB> + 'static,
    {
        fn on_migrate(&self, st: &mut CreateTableSt<DB>) {
            self.on_migrate(st)
        }
        fn table_name(&self) -> &'static str {
            <Self as Collection<DB>>::table_name(self)
        }
    }

    impl<B, C, L, F, DB> Schema<B, C, L, F, DB>
    where
        C: Collections<DB>,
        DB: QueryBuilder,
    {
        pub fn create_json_client(self, pool: Pool<DB>) -> JsonClient<DB> {
            let collections = self
                .collections
                .into_dynamic_collections()
                .into_iter()
                .map(|e| (e.table_name(), e))
                .collect();

            JsonClient {
                collections,
                db: pool,
            }
        }
    }
}

impl Default for Schema<unopinionated, (), (), (), ()> {
    fn default() -> Self {
        Schema {
            _pd: PhantomData,
            collections: (),
        }
    }
}

#[cfg(feature = "serde")]
pub mod serde_client {}
