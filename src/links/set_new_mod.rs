pub struct SetNew<Relation, Data> {
    pub relation: Relation,
    pub data: Data,
}

pub mod hardcode_api {
    use crate::{
        collections::HasHandler,
        links::{Link, LinkedViaId, set_new_mod::SetNew},
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
        type Spec = SetNew<<Entry::Handler as Link<From>>::Spec, Entry>;
        fn spec(self) -> Self::Spec {
            SetNew {
                relation: Entry::Handler::spec(Entry::Handler::default()),
                data: self.0,
            }
        }
    }
}
