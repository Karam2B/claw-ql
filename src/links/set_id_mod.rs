pub struct SetId<Relation, Id> {
    pub relation: Relation,
    pub id: Id,
}

pub mod hardcode_api {
    use crate::links::{Link, set_id_mod::SetId};

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
