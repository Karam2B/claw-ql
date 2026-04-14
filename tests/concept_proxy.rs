#![allow(unused)]
#![warn(unused_must_use)]

use std::{any::Any, borrow::Borrow};

use claw_ql::update_mod::Update;

mod base_crate {
    use std::borrow::Borrow;

    pub trait Proxy<PS> {
        type AsProxy;
    }
    pub trait AsProxy<PS>: Proxy<PS> {
        fn as_proxy(self, proxy_spec: PS) -> Self::AsProxy;
    }

    pub trait Original {
        type AsOriginal;
    }

    pub trait ProxySpec<From> {
        type Into;
    }

    pub trait AsProxySpec<From>: ProxySpec<From> {
        fn as_proxy(&self, value: From, name: &'static str) -> Self::Into;
    }

    // feature: flex_proxy
    pub trait FlexProxySpec<Into, From> {}

    // feature: visitor_spec
    pub trait Visit<PS, Visitor>: Original {
        fn visit(&self, visitor: Visitor);
    }
    pub trait VisitorSpec<PS, T>: ProxySpec<T>
    where
        PS: ProxySpec<T>,
    {
        fn visit(&self, value: &T, name: &'static str);
    }
}

mod extention_side {
    use super::base_crate::*;
    use claw_ql::update_mod::Update;
    use std::borrow::Borrow;

    pub struct PartialSpec;

    impl<T> ProxySpec<T> for PartialSpec {
        type Into = Update<T>;
    }

    impl<T> AsProxySpec<T> for PartialSpec {
        fn as_proxy(&self, value: T, _: &'static str) -> Self::Into {
            Update::Set(value)
        }
    }

    impl<'r, T, E> FlexProxySpec<Update<&'r E>, T> for PartialSpec
    where
        E: ?Sized,
        T: Borrow<E>,
    {
    }

    impl<'r, T, E> FlexProxySpec<&'r E, T> for PartialSpec
    where
        E: ?Sized,
        T: Borrow<E>,
    {
    }
    impl<T> FlexProxySpec<(), T> for PartialSpec {}
}

mod default_trait_side {
    use super::base_crate::*;

    pub trait ProxySpecDefault<T>: ProxySpec<T> {
        fn default() -> Self::Into;
    }

    mod extention_side {
        use claw_ql::update_mod::Update;

        use super::*;
        use crate::extention_side::*;

        impl<T> ProxySpecDefault<T> for PartialSpec {
            fn default() -> Self::Into {
                Update::Keep
            }
        }
    }

    mod example_side {
        use crate::example_side::TodoProxy;

        use super::*;

        impl<PS> Default for TodoProxy<PS>
        where
            PS: ProxySpecDefault<String>,
            PS: ProxySpecDefault<bool>,
            PS: ProxySpecDefault<Option<String>>,
        {
            fn default() -> Self {
                Self {
                    title: <PS as ProxySpecDefault<String>>::default(),
                    done: <PS as ProxySpecDefault<bool>>::default(),
                    description: <PS as ProxySpecDefault<Option<String>>>::default(),
                }
            }
        }
    }
}

mod extension_2_side {
    use super::base_crate::*;
    use std::any::Any;

    pub struct CastProxy;
    impl<T> ProxySpec<T> for CastProxy {
        type Into = Box<dyn Any>;
    }
    impl<T: 'static> AsProxySpec<T> for CastProxy {
        fn as_proxy(&self, value: T, _: &'static str) -> Self::Into {
            Box::new(value) as Box<dyn Any>
        }
    }
}

mod example_side {
    use super::base_crate::*;

    pub struct Todo {
        pub title: String,
        pub done: bool,
        pub description: Option<String>,
    }

    pub struct TodoProxy<PS>
    where
        PS: ProxySpec<String>,
        PS: ProxySpec<bool>,
        PS: ProxySpec<Option<String>>,
    {
        pub title: <PS as ProxySpec<String>>::Into,
        pub done: <PS as ProxySpec<bool>>::Into,
        pub description: <PS as ProxySpec<Option<String>>>::Into,
    }

    impl<PS> Proxy<PS> for Todo
    where
        PS: ProxySpec<String>,
        PS: ProxySpec<bool>,
        PS: ProxySpec<Option<String>>,
    {
        type AsProxy = TodoProxy<PS>;
        // fn as_proxy(self, proxy_spec: PS) -> Self::AsProxy {
        //     TodoProxy {
        //         title: proxy_spec.as_proxy(self.title, "title"),
        //         done: proxy_spec.as_proxy(self.done, "done"),
        //         description: proxy_spec.as_proxy(self.description, "description"),
        //     }
        // }
        // fn visit<Visitor>(this: &Self::AsProxy, visitor: Visitor) {
        //     // visitor.visit(&this.title, "title")
        //     // visitor.visit(&this.done, "done")
        //     // visitor.visit(&this.description, "description")
        // }
    }

    impl<PS> AsProxy<PS> for Todo
    where
        PS: AsProxySpec<String>,
        PS: AsProxySpec<bool>,
        PS: AsProxySpec<Option<String>>,
    {
        fn as_proxy(self, proxy_spec: PS) -> Self::AsProxy {
            TodoProxy {
                title: proxy_spec.as_proxy(self.title, "title"),
                done: proxy_spec.as_proxy(self.done, "done"),
                description: proxy_spec.as_proxy(self.description, "description"),
            }
        }
    }

    // impl<PS, Visitor> Visit<PS, Visitor> for TodoProxy<PS>
    // where
    //     Visitor: VisitorSpec<PS, String>,
    //     Visitor: VisitorSpec<PS, bool>,
    //     Visitor: VisitorSpec<PS, Option<String>>,
    //     PS: ProxySpec<String>,
    //     PS: ProxySpec<bool>,
    //     PS: ProxySpec<Option<String>>,
    // {
    //     fn visit(&self, visitor: Visitor) {
    //         <Visitor as VisitorSpec<PS, String>>::visit(&visitor, &self.title, "title");
    //         // visitor.visit(&self.title, "title");
    //         // visitor.visit(&self.done, "done");
    //         // visitor.visit(&self.description, "description");
    //     }
    // }

    impl<PS> Original for TodoProxy<PS>
    where
        PS: ProxySpec<String>,
        PS: ProxySpec<bool>,
        PS: ProxySpec<Option<String>>,
    {
        type AsOriginal = Todo;
    }

    pub struct TodoFlexProxy<PS, E0, E1, E2>
    where
        PS: FlexProxySpec<E0, String>,
        PS: FlexProxySpec<E1, bool>,
        PS: FlexProxySpec<E2, Option<String>>,
    {
        pub _ps: PS,
        pub title: E0,
        pub done: E1,
        pub description: E2,
    }

    impl<PS, E0, E1, E2> Original for TodoFlexProxy<PS, E0, E1, E2>
    where
        PS: FlexProxySpec<E0, String>,
        PS: FlexProxySpec<E1, bool>,
        PS: FlexProxySpec<E2, Option<String>>,
    {
        type AsOriginal = Todo;
    }
}

pub trait StructAsTuple {
    type Tuple;
    fn into_tuple(self) -> Self::Tuple;
    fn names(&self) -> &'static [&'static str];
    fn from_tuple(tuple: Self::Tuple) -> Self;
}

impl StructAsTuple for Todo {
    type Tuple = (String, bool, Option<String>);
    fn into_tuple(self) -> Self::Tuple {
        (self.title, self.done, self.description)
    }
    fn names(&self) -> &'static [&'static str] {
        &["title", "done", "description"]
    }
    fn from_tuple(tuple: Self::Tuple) -> Self {
        Self {
            title: tuple.0,
            done: tuple.1,
            description: tuple.2,
        }
    }
}

use base_crate::Proxy;
use base_crate::*;
use default_trait_side::*;
use example_side::{Todo, TodoProxy};
use extention_side::*;

use crate::example_side::TodoFlexProxy;
type TodoPartial = TodoProxy<PartialSpec>;

fn main() {
    let partial: TodoPartial = TodoPartial {
        title: Update::Keep,
        done: Update::Keep,
        description: Update::Keep,
    };

    let data: Todo = Todo {
        title: "title".to_string(),
        done: true,
        description: Some("description".to_string()),
    };

    let mut partial = data.as_proxy(PartialSpec);
    partial.title = Update::Keep;

    let mut keeps = <Todo as Proxy<PartialSpec>>::AsProxy::default();
    keeps.title = Update::Set("new_title".to_string());

    TodoFlexProxy {
        _ps: PartialSpec,
        title: "new_title",
        done: (),
        description: (),
    };
}
