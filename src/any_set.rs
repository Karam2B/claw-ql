use std::{
    any::{Any, TypeId},
    collections::HashMap,
};

#[derive(Default)]
pub struct AnySet {
    inner: HashMap<TypeId, Box<dyn Any>>,
}

pub enum InsertOption<T> {
    Replaces(T),
    WasNew,
}

impl AnySet {
    pub fn set<T: Any>(&mut self, item: T) -> InsertOption<T> {
        let type_id = item.type_id();
        match self.inner.insert(type_id, Box::new(item)) {
            Some(replace) => InsertOption::Replaces(*replace.downcast::<T>().unwrap()),
            None => InsertOption::WasNew,
        }
    }
    pub fn get_mut<T: Any>(&mut self) -> Option<&mut T> {
        let type_id = TypeId::of::<T>();
        let get = self.inner.get_mut(&type_id);

        get.map(|e| e.as_mut().downcast_mut::<T>().unwrap())
    }
    pub fn get<T: Any>(&self) -> Option<&T> {
        let type_id = TypeId::of::<T>();
        let get = self.inner.get(&type_id);

        get.map(|e| e.as_ref().downcast_ref::<T>().unwrap())
    }
}
