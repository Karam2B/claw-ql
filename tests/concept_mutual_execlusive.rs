#![allow(unused)]

use std::collections::HashSet;
pub trait MutualExclusiveState {
    type State: From<Self::MutuallyExclusiveState>;
    type MutuallyExclusiveState: TryFrom<Self::State>;
}
pub trait MutualExclusive<WithRespectTo: MutualExclusiveState> {
    fn state(&self) -> WithRespectTo::State;
}

pub enum RollingDice {
    One,
    Two,
    Three,
    Four,
    Five,
    Six,
}

pub struct CollectionOfRollingEvents {
    events: HashSet<RollingDice>,
}

// by implementing this, I guarantee that ther is no
// duplicate in my HashSet.
impl MutualExclusiveState for CollectionOfRollingEvents {
    type State = RollingDice;
    type MutuallyExclusiveState = RollingDice;
}

pub struct AlwaysTrueBool;
pub struct AlwaysFalseBool;

impl From<AlwaysTrueBool> for bool {
    fn from(value: AlwaysTrueBool) -> Self {
        true
    }
}

impl TryFrom<bool> for AlwaysTrueBool {
    type Error = ();
    fn try_from(value: bool) -> Result<Self, Self::Error> {
        if value { Ok(AlwaysTrueBool) } else { Err(()) }
    }
}

pub struct OnlyOneIs<T, U> {
    t: Vec<T>,
    u: U,
}

fn example() {
    struct A {
        turned_on: bool,
        color: String,
    }

    struct Collection {
        items: Vec<A>,
    }

    // by implementing this, I guarantee that only one
    // item can be turned on at a time.
    impl MutualExclusiveState for Collection {
        type State = bool;
        type MutuallyExclusiveState = AlwaysTrueBool;
    }

    impl Collection {
        fn new(items: Vec<String>) -> Self {
            Self {
                items: items
                    .into_iter()
                    .map(|item| A {
                        turned_on: false,
                        color: item,
                    })
                    .collect(),
            }
        }
        fn turn_index_on(&mut self, color: &str) {
            for item in self.items.iter_mut() {
                if item.color == color {
                    item.turned_on = true;
                } else {
                    item.turned_on = false;
                }
            }
        }
    }

    impl MutualExclusive<Collection> for A {
        fn state(&self) -> bool {
            self.turned_on
        }
    }

    let mut collection = Collection::new(vec!["red".to_string(), "blue".to_string()]);
    collection.turn_index_on("red");
}

mod v2 {
    use crate::{AlwaysFalseBool, AlwaysTrueBool};

    pub struct OnlyOneIsOn<T, U> {
        t: Vec<T>,
        u: Option<U>,
    }

    pub enum InitState {
        AllFalse,
        OneTrue,
    }

    pub trait Subset<P>: Sized {
        fn try_from_and_clone(p: &P) -> Option<Self>;
        fn into(self) -> P;
    }

    impl Subset<bool> for AlwaysTrueBool {
        fn try_from_and_clone(p: &bool) -> Option<Self> {
            if *p { Some(AlwaysTrueBool) } else { None }
        }
        fn into(self) -> bool {
            true
        }
    }

    impl Subset<bool> for AlwaysFalseBool {
        fn try_from_and_clone(p: &bool) -> Option<Self> {
            if !p { Some(AlwaysFalseBool) } else { None }
        }
        fn into(self) -> bool {
            false
        }
    }

    impl<P, ME> OnlyOneIsOn<P, ME>
    where
        ME: Subset<P>,
    {
        pub fn new(mut t: Vec<P>) -> Result<(Self, InitState), ()> {
            let mut iterate = t.iter_mut();
            let mut found = None;
            while let Some(item) = iterate.next() {
                if let Some(me) = ME::try_from_and_clone(item) {
                    found = Some(me);
                }
            }
            if found.is_none() {
                return Ok((OnlyOneIsOn { t, u: None }, InitState::AllFalse));
            }
            for rest in iterate {
                if let Some(me) = ME::try_from_and_clone(rest) {
                    // there is more than one true
                    return Err(());
                }
            }

            Ok((OnlyOneIsOn { t, u: found }, InitState::OneTrue))
        }
    }

    pub struct StartsWith(String);

    // impl StartsWith {
    //     pub fn new(s: String, starts) -> Self {
    //         Self(s)
    //     }
    // }

    impl Subset<String> for StartsWith {
        fn try_from_and_clone(p: &String) -> Option<Self> {
            if p.starts_with(&Self(p.clone()).0) {
                Some(Self(p.clone()))
            } else {
                None
            }
        }
        fn into(self) -> String {
            self.0
        }
    }
}
