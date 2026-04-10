#![allow(unused)]
#![allow(non_camel_case_types)]
#![warn(unused_must_use)]
use std::{collections::HashMap, marker::PhantomData};

use claw_ql::{
    ConnectInMemory, QueryBuilder,
    json_client::add_collection::SerializableAny,
    links::set_id::SetIdSpec,
    operations::select_one_op::{SelectOneFragment, select_one},
    prelude::macro_relation::optional_to_many,
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

impl Link<todo> for may_have_a_category {
    type Spec = optional_to_many<todo, category>;
    fn spec(self, c: &todo) -> Self::Spec {
        optional_to_many {
            foriegn_key: "".to_string(),
            from: todo,
            to: category,
        }
    }
}

impl LinkedToCollection for may_have_a_category {
    type To = category;
}

impl Link<todo> for category {
    type Spec = optional_to_many<todo, category>;
    fn spec(self, c: &todo) -> Self::Spec {
        optional_to_many {
            foriegn_key: "".to_string(),
            from: todo,
            to: category,
        }
    }
}

impl LinkedToCollection for category {
    type To = todo;
}

impl DatedTrait for todo {}

pub mod link_method {
    use std::marker::PhantomData;

    use claw_ql::{
        QueryBuilder,
        build_tuple::BuildTuple,
        operations::select_one_op::{SelectOne, SelectOneFragment},
    };

    use crate::Link;

    pub fn link<S, Base, L, F, D>(
        s: SelectOne<S, Base, L, F>,
        ty: D,
    ) -> SelectOne<S, Base, L::Bigger<D::Spec>, F>
    // D: LinkIdent<Base>,
    // D::SpecInfo: LinkSpec<Spec: SelectOneFragment<S> + Send>,
    where
        Base: Clone,
        L: BuildTuple,
        S: QueryBuilder,
        D: Link<Base, Spec: SelectOneFragment<S>>,
    {
        SelectOne {
            links: s.links.into_bigger(ty.spec(&s.collection)),
            filters: s.filters,
            collection: s.collection,
            _pd: PhantomData,
        }
    }
}

pub trait Link<Base> {
    type Spec;
    fn spec(self, base: &Base) -> Self::Spec;
}
pub trait LinkedToCollection {
    type To;
}
pub trait LiqLink {}

pub struct date;
pub struct date_spec<F>(F);
pub trait DatedTrait: Clone {}
impl<C: DatedTrait> Link<C> for date {
    type Spec = date_spec<C>;
    fn spec(self, b: &C) -> Self::Spec {
        date_spec(b.clone())
    }
}

pub struct set_id<T> {
    to: T,
    id: i32,
}

pub struct set_id_fragment<B, T> {
    from: B,
    to: T,
    id: i32,
}

impl<B, T> Link<B> for set_id<T>
where
    T: LinkedToCollection,
    T: Link<B, Spec = optional_to_many<B, T::To>>,
{
    type Spec = set_id_fragment<B, T::To>;
    fn spec(self, b: &B) -> Self::Spec {
        let old_spec = self.to.spec(b);
        set_id_fragment {
            from: old_spec.from,
            to: old_spec.to,
            id: self.id,
        }
    }
}

#[tokio::test]
async fn main_test() {
    let pool = Sqlite::connect_in_memory().await;
    // let link_extentions: HashMap<String, Box<dyn LiqLinkData<Sqlite>>> = Default::default();
    // HashMap<JsonSelector, Box<dyn DynamicLinkRT<S>>>

    let res = select_one(todo);
    // let res = to_be_dep::link(res, may_have_a_category.set_id(3));
    // let res = link_method::link(res, may_have_a_category);
    let res = link_method::link(res, date);
    let res = link_method::link(
        res,
        set_id {
            to: may_have_a_category,
            id: 3,
        },
    );

    // let res = to_be_dep::link(res, date);

    let res = res.exec_op(pool.clone()).await;

    let link_extentions: HashMap<String, Box<dyn LiqLinkData<Sqlite>>> = HashMap::from([
        (
            "claw_ql::date".to_string(),
            Box::new(date::liq_link()) as Box<dyn LiqLinkData<Sqlite>>,
        ),
        (
            "claw_ql::optional_to_many".to_string(),
            Box::new(optional_to_many_liq_link::default()) as Box<dyn LiqLinkData<Sqlite>>,
        ),
    ]);
}
