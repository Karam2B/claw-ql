use crate::{
    collections::HasHandler,
    links::{Link, LinkedViaId},
};

#[allow(non_camel_case_types)]
pub struct set_new<E>(pub E);

impl<From, Entry> Link<From> for set_new<Entry>
where
    // linkedviaids should have different set_new!
    Entry::Handler: Link<From, Spec: LinkedViaId>,
    Entry: HasHandler,
    From: Clone,
    Entry::Handler: Default,
{
    type Spec = SetNewSpec<<Entry::Handler as Link<From>>::Spec, Entry>;
    fn spec(self) -> Self::Spec {
        SetNewSpec {
            og_spec: Entry::Handler::spec(Entry::Handler::default()),
            entry: self.0,
        }
    }
}

pub struct SetNewSpec<OgSpec, Entry> {
    pub og_spec: OgSpec,
    pub entry: Entry,
}
