use std::marker::PhantomData;

use crate::{AcceptNoneBind, IdentSafety, unstable::Unsateble};

pub struct Col<IS = ()> {
    pub(crate) table: Option<String>,
    pub(crate) alias: Option<String>,
    pub(crate) col: String,
    pub(crate) is: PhantomData<IS>,
}

impl<IS: IdentSafety> AcceptNoneBind for Col<IS> {
    type IdentSafety = IS;
    fn accept(self, _: &IS, _: Unsateble) -> String {

        format!(
            "{}{}{}",
            match self.table {
                Some(table) => format!("{table}."),
                None => "".to_string(),
            },
            self.col,
            match self.alias {
                Some(alias) => format!("AS {alias}"),
                None => "".to_string(),
            }
        )
    }
}

pub struct ColEq<T, IS = ()> {
    pub(crate) col: Col<IS>,
    pub(crate) item: T,
    pub(crate) is: PhantomData<IS>,
}

impl<IS: IdentSafety> Col<IS> {
    pub fn table(mut self, table: &str) -> Self {
        self.alias = Some(table.to_string());
        self
    }
    pub fn alias(mut self, alias: &str) -> Self {
        self.alias = Some(alias.to_string());
        self
    }
    pub fn eq<T1>(self, value: T1) -> ColEq<T1, IS> {
        ColEq {
            col: self,
            item: value,
            is: PhantomData,
        }
    }
}

pub mod exports {
    use super::*;
    use std::marker::PhantomData;

    #[track_caller]
    pub fn col<IS: IdentSafety>(str: &str) -> Col<IS> {
        IS::check(str);
        Col {
            table: None,
            col: str.to_string(),
            is: PhantomData::<IS>,
            alias: None,
        }
    }
}
