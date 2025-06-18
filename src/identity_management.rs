#![allow(dead_code)]
use std::{marker::PhantomData, ops::Not};

// identity management is aimed to:
// 1. prevent sql injection by rejecting unrecognized input
// 2. reduce errors (like spelling) either at a runtime or buildtime
// 3. help in migration from one version of the schema to another
//
// it seems like the top level of this crate makes this concept
// obselete!!
pub trait AcceptColumn<I: IdentManager> {
    fn accept(self, im: &I) -> String;
}

pub trait IdentManager: Default {}

impl IdentManager for () {}
impl<T> IdentManager for PhantomData<T> {}

mod err {
    #[allow(non_camel_case_types)]
    pub struct col<C>(pub C);
}

#[derive(Default)]
struct Registery {
    pub table: Vec<String>,
    pub columns: Vec<(String, String)>,
}

impl IdentManager for Registery {}

impl AcceptColumn<Registery> for String {
    fn accept(self, im: &Registery) -> String {
        if im.columns.iter().any(|e| e.1 == self).not() {
            panic!("column {self} is not safe")
        }
        self
    }
}
impl AcceptColumn<Registery> for &str {
    fn accept(self, im: &Registery) -> String {
        let str = self.to_owned();
        if im.columns.iter().any(|e| e.1 == self).not() {
            panic!("column {self} is not safe")
        }
        str
    }
}
impl AcceptColumn<()> for String {
    fn accept(self, _im: &()) -> String {
        self
    }
}
impl AcceptColumn<()> for &str {
    fn accept(self, _im: &()) -> String {
        self.to_owned()
    }
}

#[allow(non_camel_case_types)]
mod schema {
    use std::marker::PhantomData;

    use crate::identity_management::AcceptColumn;

    struct todo;
    struct todo_title;
    struct todo_done;

    struct cat;
    struct cat_title;

    impl AcceptColumn<PhantomData<todo>> for todo_title {
        fn accept(self, _im: &PhantomData<todo>) -> String {
            String::from("todo_title")
        }
    }
    impl AcceptColumn<PhantomData<todo>> for todo_done {
        fn accept(self, _im: &PhantomData<todo>) -> String {
            String::from("todo_done")
        }
    }

    impl AcceptColumn<PhantomData<cat>> for cat_title {
        fn accept(self, _im: &PhantomData<cat>) -> String {
            String::from("cat_title")
        }
    }

    fn example() {
        let mut st = super::SelectSt {
            select_list: Vec::default(),
            im: PhantomData::<todo>,
        };

        st.select(todo_title);
        st.select(todo_done);
        // st.select(cat_title); // compile time error
    }
}

struct SelectSt<I: IdentManager = ()> {
    select_list: Vec<String>,
    im: I,
}

impl<I: IdentManager> SelectSt<I> {
    fn select<T>(&mut self, item: T)
    where
        T: AcceptColumn<I>,
    {
        self.select_list.push(item.accept(&self.im))
    }
}
