#![allow(unused)]
#![warn(unused_must_use)]
#![allow(non_camel_case_types)]

use claw_ql::{
    ExpressionToFragment, QueryBuilder, SanitzingMechanisim, SelectListItem, WhereItem,
    build_tuple::BuildTuple, sanitize::SanitizeAndHardcode, statements::select_st::SelectSt,
};
use claw_ql_macros::dynamic_fn;

pub struct GenericSelect<From, SelectList, Where> {
    from: From,
    select_list: SelectList,
    where_: Where,
}

impl<F, S, W> GenericSelect<F, S, W> {
    pub fn mutate_dynamic_stmt<'q, Q>(self, stmt: &mut SelectSt<Q>)
    where
        Q: QueryBuilder + Default + SanitzingMechanisim,
        // for select
        S: SelectListItem,
        S: SanitizeAndHardcode<<Q as SanitzingMechanisim>::SanitzingMechanisim>,
        // for where
        W: WhereItem,
        Q: ExpressionToFragment<'q, W>,
    {
        stmt.select(self.select_list);
        stmt.where_(self.where_);
    }
    pub fn into_dynamic_stmt<'q, Q>(self, query_builder: Q) -> SelectSt<Q>
    where
        Q: QueryBuilder + Default + SanitzingMechanisim,
        // for init
        F: SelectListItem,
        F: SanitizeAndHardcode<<Q as SanitzingMechanisim>::SanitzingMechanisim>,
        // for select
        S: SelectListItem,
        S: SanitizeAndHardcode<<Q as SanitzingMechanisim>::SanitzingMechanisim>,
        // for where
        W: WhereItem,
        Q: ExpressionToFragment<'q, W>,
    {
        let mut st = SelectSt::init(self.from, query_builder);

        st.select(self.select_list);
        st.where_(self.where_);

        st
    }
}

impl<From> GenericSelect<From, (), ()> {
    fn new(from: From) -> Self {
        GenericSelect {
            from,
            select_list: (),
            where_: (),
        }
    }
}

impl<From, SelectList, Where> GenericSelect<From, SelectList, Where> {
    fn add_select<N>(
        self,
        next: N,
    ) -> GenericSelect<From, <SelectList as BuildTuple>::Bigger<N>, Where>
    where
        SelectList: BuildTuple,
    {
        GenericSelect {
            from: self.from,
            select_list: self.select_list.into_bigger(next),
            where_: self.where_,
        }
    }
    fn add_where<N>(
        self,
        next: N,
    ) -> GenericSelect<From, SelectList, <Where as BuildTuple>::Bigger<N>>
    where
        Where: BuildTuple,
    {
        GenericSelect {
            from: self.from,
            select_list: self.select_list,
            where_: self.where_.into_bigger(next),
        }
    }
}

pub struct todo;
pub struct todo_title;
pub struct todo_description;

#[test]
fn select_generic() {
    let st = GenericSelect::new(todo)
        .add_select(todo_description)
        .add_select(todo_title)
        .add_select(todo_description);

    // let st = st.add_select(todo_description);
    // let st = st.add_select(todo_title);
    // let st = st.add_select(todo_description);
    // let st = st.add_where(());
}

#[dynamic_fn]
fn dynamic_fn() {
    let st = GenericSelect::new(todo)
        .add_select(todo_description)
        .add_select(todo_title)
        .add_select(todo_description);

    let s = schema.select_one(todo).by_id(3);

    passing_to(s);

    return st;
}
