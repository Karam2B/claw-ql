pub mod by_id_mod;

// // TODO: should we use this trait or FilterGeneric?
// pub trait Filter<Q, C: ?Sized>: Sync + Send {
//     fn on_delete(self, handler: &C, st: &mut DeleteSt<Q>)
//     where
//         Q: QueryBuilder;
//     fn on_update(self, handler: &C, st: &mut UpdateSt<Q>)
//     where
//         Q: QueryBuilder;
//     fn on_select(self, handler: &C, st: &mut SelectSt<Q>)
//     where
//         Q: QueryBuilder;
// }

// // #[rustfmt::skip]
// mod filters_tuple_impls {
//     use super::Filter;
//     use crate::{
//         QueryBuilder,
//         statements::{delete_st::DeleteSt, select_st::SelectSt, update_st::UpdateSt},
//     };
//     use paste::paste;

//     macro_rules! implt {
//         ($([$ty:ident, $part:literal],)*) => {
//     #[allow(unused)]
//     impl
//         <S,C, $($ty,)* >
//     Filter<S,C>
//     for
//         ($($ty,)*)
//     where

//         S: QueryBuilder,
//         $($ty:  Filter<S, C>,)*
//     {
//         fn on_delete(self, h: &C, st: &mut DeleteSt<S>) {
//             $(paste!(self.$part.on_delete(h, st));)*
//         }
//         fn on_update(self, h: &C, st: &mut UpdateSt<S>) {
//             $(paste!(self.$part.on_update(h, st));)*
//         }
//         fn on_select(self, h: &C, st: &mut SelectSt<S>) {
//             $(paste!(self.$part.on_select(h, st));)*
//         }
//     }
//         }}

//     implt!();
//     #[allow(unused)]
//     impl<S, C, R0> Filter<S, C> for (R0,)
//     where
//         S: QueryBuilder,
//         R0: Filter<S, C>,
//     {
//         fn on_delete(self, h: &C, st: &mut DeleteSt<S>) {
//             paste!(self.0.on_delete(h, st));
//         }
//         fn on_update(self, h: &C, st: &mut UpdateSt<S>) {
//             paste!(self.0.on_update(h, st));
//         }
//         fn on_select(self, h: &C, st: &mut SelectSt<S>) {
//             paste!(self.0.on_select(h, st));
//         }
//     }
//     implt!([R0, 0], [R1, 1],);
// }
