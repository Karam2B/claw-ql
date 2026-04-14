#![allow(unused)]
use crate::links::Link;
use sqlx::{ColumnIndex, Decode, Encode, prelude::Type};
use sqlx::{IntoArguments, Row};
use std::{future::Future, usize};

#[allow(non_camel_case_types)]
pub struct set_id<T> {
    pub to: T,
    pub id: i64,
}

pub struct SetIdSpec<OgSpec, Input> {
    pub og_spec: OgSpec,
    pub input: Input,
}

impl<To, From> Link<From> for set_id<To>
where
    From: Clone,
    To: Clone,
    To: Link<From>,
{
    type Spec = SetIdSpec<To::Spec, i64>;

    fn spec(self) -> Self::Spec
    where
        Self: Sized,
    {
        SetIdSpec {
            og_spec: self.to.spec(),
            input: self.id,
        }
    }
}
