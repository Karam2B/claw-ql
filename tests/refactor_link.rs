#![allow(unused)]
#![allow(non_camel_case_types)]
#![warn(unused_must_use)]
use std::{collections::HashMap, marker::PhantomData};

use claw_ql::{
    ConnectInMemory,
    QueryBuilder,
    json_client::add_collection::SerializableAny,
    links::set_id::SetIdSpec,
    operations::select_one_op::{SelectOneFragment, select_one},
    // prelude::macro_relation::OptionalToMany,
};
use claw_ql_macros::{Collection, relation};
use serde::{Deserialize, Serialize};
use sqlx::{Sqlite, sqlite::types};

#[derive(Collection, Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct Todo {
    pub title: String,
    pub done: bool,
    pub description: Option<String>,
}

#[derive(Collection, Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct Category {
    pub title: String,
}

#[derive(Collection, Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct Tag {
    pub title: String,
}

pub struct may_have_a_category;

const _: () = {
    impl From<may_have_a_category> for OptionalToMany<todo, category> {
        fn from(value: may_have_a_category) -> Self {
            OptionalToMany {
                foriegn_key: "category_id".to_string(),
                from: todo,
                to: category,
            }
        }
    }

    impl LinkSpec for may_have_a_category {
        type Base = todo;
        type Spec = OptionalToMany<todo, category>;
    }

    impl LinkIdent<todo> for may_have_a_category {
        type SpecInfo = may_have_a_category;
        fn to_spec(self, _: todo) -> Self::SpecInfo {
            self
        }
    }

    impl LinkIdent<todo> for date {
        type SpecInfo = dated<todo>;
        fn to_spec(self, base: todo) -> Self::SpecInfo {
            dated(base)
        }
    }

    trait Link<Spec> {}

    impl Link<date> for todo {}
};

pub struct dated<T>(T);
impl<S, T> SelectOneFragment<S> for dated<T>
where
    S: QueryBuilder,
    T: Send + Sync,
{
    type Inner = ();

    type Output = ();

    fn on_select(&mut self, data: &mut Self::Inner, st: &mut claw_ql::prelude::stmt::SelectSt<S>) {
        todo!()
    }

    fn from_row(&mut self, data: &mut Self::Inner, row: &<S>::Row) {
        todo!()
    }

    fn sub_op<'this>(
        &'this mut self,
        data: &'this mut Self::Inner,
        pool: sqlx::Pool<S>,
    ) -> impl Future<Output = ()> + Send + use<'this, T, S> {
        async { todo!() }
    }

    fn take(self, data: Self::Inner) -> Self::Output {
        todo!()
    }
}
pub struct date;

pub struct date_as_liq_link {
    // tables
    dated_collections: Vec<String>,
}

impl SerializableAny for date_as_liq_link {
    fn typeid(&self) -> String {
        todo!()
    }
}

// impl<T> LinkIdent<T> for date {
//     type SpecInfo = dated<T>;
//     fn to_spec(self, base: T) -> Self::SpecInfo {
//         dated(base)
//     }
// }

pub trait LinkSpec: Sized {
    type Spec: From<Self>;
    type Base;
}

pub trait LinkIdent<Base> {
    type SpecInfo: LinkSpec<Base = Base>;
    fn to_spec(self, base: Base) -> Self::SpecInfo;
    fn to_spec_2(self, base: Base) -> <Self::SpecInfo as LinkSpec>::Spec
    where
        Self: Sized,
    {
        self.to_spec(base).into()
    }
}

pub trait LiqLinkData<Q> {}
impl<Q, T> LiqLinkData<Q> for T {}

pub mod to_be_dep {
    use std::marker::PhantomData;

    use claw_ql::{
        QueryBuilder,
        build_tuple::BuildTuple,
        operations::select_one_op::{SelectOne, SelectOneFragment},
    };

    use crate::{LinkIdent, LinkSpec};

    pub fn link<S, Base, L, F, D>(
        s: SelectOne<S, Base, L, F>,
        ty: D,
    ) -> SelectOne<S, Base, L::Bigger<<D::SpecInfo as super::LinkSpec>::Spec>, F>
    where
        Base: Clone,
        L: BuildTuple,
        S: QueryBuilder,
        D: LinkIdent<Base>,
        D::SpecInfo: LinkSpec<Spec: SelectOneFragment<S> + Send>,
    {
        SelectOne {
            links: s.links.into_bigger(ty.to_spec(s.collection.clone()).into()),
            filters: s.filters,
            collection: s.collection,
            _pd: PhantomData,
        }
    }
}

pub struct set_id_spec<From, To, Input> {
    input: Input,
    to: To,
    from: From,
}
pub struct set_id<To, Input> {
    input: Input,
    to: To,
}

impl<F, T> LinkIdent<F> for set_id<T, i32> {
    type SpecInfo = set_id_spec<F, T, i32>;
    fn to_spec(self, base: F) -> Self::SpecInfo {
        set_id_spec {
            from: base,
            input: self.input,
            to: self.to,
        }
    }
}

impl<F, T> LinkSpec for set_id_spec<F, T, i32> {
    type Base = F;
    type Spec = Self;
}

impl<F: Send + Sync, T: Send + Sync, S: QueryBuilder> SelectOneFragment<S>
    for set_id_spec<F, T, i32>
{
    type Inner = ();

    type Output = ();

    fn on_select(&mut self, data: &mut Self::Inner, st: &mut claw_ql::prelude::stmt::SelectSt<S>) {
        todo!()
    }

    fn from_row(&mut self, data: &mut Self::Inner, row: &<S>::Row) {
        todo!()
    }

    fn sub_op<'this>(
        &'this mut self,
        data: &'this mut Self::Inner,
        pool: sqlx::Pool<S>,
    ) -> impl Future<Output = ()> + Send + use<'this, T, F, S> {
        async { todo!() }
    }

    fn take(self, data: Self::Inner) -> Self::Output {
        todo!()
    }
}

trait LinkedViaI32<To>: Sized {
    fn set_id(self, ii: i32) -> set_id<To, i32> {
        todo!()
    }
}

// impl<F, T> LinkedViaI32<T> for OptionalToMany<F, T> {}
// impl<T, To> SetId<To, i32> for T
// // where
// //     T: LinkData<Spec: LinkedViaI32<To>>,
// {
//     fn set_id(self, n: To) -> set_id<To, i32> {
//         todo!()
//     }
// }

#[tokio::test]
async fn main_test() {
    let pool = Sqlite::connect_in_memory().await;
    // let link_extentions: HashMap<String, Box<dyn LiqLinkData<Sqlite>>> = Default::default();
    // HashMap<JsonSelector, Box<dyn DynamicLinkRT<S>>>

    let res = select_one(todo);
    // let res = to_be_dep::link(res, may_have_a_category.set_id(3));
    let res = to_be_dep::link(
        res,
        set_id {
            to: category,
            input: 3,
        },
    );

    let res = to_be_dep::link(res, date);

    let res = res.exec_op(pool.clone()).await;
}

pub struct optional_to_many {
    // (table, fk)
    pub existing_links: Vec<(String, String)>,
}

#[tokio::test]
async fn jc_test() {
    let pool = Sqlite::connect_in_memory().await;
    let link_extentions: HashMap<String, Box<dyn LiqLinkData<Sqlite>>> = HashMap::from([
        (
            "claw_ql::date".to_string(),
            Box::new(
            // date_as_liq_link {
            //     c: Default::default(),
            // }
        ) as Box<dyn LiqLinkData<Sqlite>>,
        ),
        (
            "claw_ql::optional_to_many".to_string(),
            Box::new(optional_to_many {
                existing_links: Default::default(),
            }) as Box<dyn LiqLinkData<Sqlite>>,
        ),
    ]);
    // HashMap<JsonSelector, Box<dyn DynamicLinkRT<S>>>

    let res = select_one(todo);
    // let res = to_be_dep::link(res, may_have_a_category.set_id(3));
    let res = to_be_dep::link(
        res,
        set_id {
            to: category,
            input: 3,
        },
    );

    let res = res.exec_op(pool.clone()).await;
}
