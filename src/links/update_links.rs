pub struct SetNew<Relation, Data> {
    pub relation: Relation,
    pub data: Data,
}

pub mod set_new_hardcode_api {
    use crate::{
        collections::HasHandler,
        links::{Link, LinkedViaId, update_links::SetNew},
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

pub struct SetId<Relation, Id> {
    pub relation: Relation,
    pub id: Id,
}

pub mod set_id_hardcode_api {
    use crate::links::{Link, update_links::SetId};

    #[allow(non_camel_case_types)]
    pub struct set_id<T> {
        pub to: T,
        pub id: i64,
    }

    impl<To, From> Link<From> for set_id<To>
    where
        From: Clone,
        To: Clone,
        To: Link<From>,
    {
        type Spec = SetId<To::Spec, i64>;

        fn spec(self) -> Self::Spec
        where
            Self: Sized,
        {
            SetId {
                relation: self.to.spec(),
                id: self.id,
            }
        }
    }
}

pub struct Unset<Relation> {
    pub relation: Relation,
}
