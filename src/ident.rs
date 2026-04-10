pub mod ident {
    #[cfg(skip_without_comment)]
    pub mod i {}

    // use core::hash;
    // use std::{
    //     collections::{HashMap, HashSet},
    //     hash::Hash,
    // };

    // use crate::collections::CollectionBasic;

    // #[derive(Eq, PartialEq, Clone, Hash)]
    // pub enum TypeIdent {
    //     Leaf(&'static str),
    //     Branch(&'static str, Vec<TypeIdent>),
    // }

    // impl TypeIdent {
    //     pub fn leaf(name: &'static str) -> TypeIdent {
    //         TypeIdent::Leaf(name)
    //     }
    //     pub fn branch(name: &'static str, to: Vec<TypeIdent>) -> Result<TypeIdent, ()> {
    //         let mut ve = vec![];
    //         let mut set = HashSet::new();
    //         for each in to {
    //             let name = match each {
    //                 TypeIdent::Leaf(s) => s,
    //                 TypeIdent::Branch(s, _) => s,
    //             };
    //             let false_if_already_exist = set.insert(name);
    //             if false_if_already_exist == false {
    //                 return Err(());
    //             }
    //             ve.push(each.clone());
    //         }
    //         Ok(TypeIdent::Branch(name, ve))
    //     }
    // }

    // pub struct Singlton {
    //     _priv: (),
    // }

    // static SINGLTON: Singlton = Singlton { _priv: () };

    // pub trait AnyS<R> {
    //     fn type_id(&self, reg: &'static R) -> TypeIdent;
    // }

    // impl<F: CollectionBasic, T: CollectionBasic> AnyS<Singlton> for Relation<F, T> {
    //     fn type_id(&self, reg: &'static Singlton) -> TypeIdent {
    //         TypeIdent::branch(
    //             "relation",
    //             [
    //                 TypeIdent::leaf(self.from.table_name()),
    //                 TypeIdent::leaf(self.to.table_name()),
    //             ]
    //             .into_iter()
    //             .collect(),
    //         )
    //         .unwrap()
    //     }
    // }
}
