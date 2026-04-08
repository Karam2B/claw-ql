#![allow(non_camel_case_types)]
#![allow(unused)]
#![warn(unused_must_use)]
use std::{fmt, sync::Arc};

struct debug_handler;

trait TraitHandler {
    type TraitObject: ?Sized;
}

impl TraitHandler for debug_handler {
    type TraitObject = dyn fmt::Debug;
}

pub struct owned_tuple<Tuple, H>(Tuple, H);

impl<T0, T1> owned_tuple<(T0, T1), debug_handler> {
    pub fn as_boxes(self) -> Vec<Box<<debug_handler as TraitHandler>::TraitObject>> {
        todo!()
    }

    pub fn as_arc(self) -> Vec<Arc<<debug_handler as TraitHandler>::TraitObject>> {
        todo!()
    }
}

fn to_vec<T, H>(t: T, trait_handler: H) -> Vec<Box<H::TraitObject>>
where
    H: TraitHandler,
{
    todo!()
}

fn tuples_again() {
    let t = ("hello world", 32);

    for each in to_vec(t, debug_handler) {}
}

trait OnMigrate {
    type Gene;
    fn to_gne() -> Self::Gene;
}

trait LiqOnMigrate {
    fn to_gene() -> Box<dyn fmt::Debug>;
}

impl<T> LiqOnMigrate for T
where
    T: OnMigrate,
    T::Gene: fmt::Debug + 'static,
{
    fn to_gene() -> Box<dyn fmt::Debug> {
        Box::new(<T as OnMigrate>::to_gne())
    }
}
