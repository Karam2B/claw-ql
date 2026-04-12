#![warn(unused_must_use)]

// pub mod date_mod;
// pub mod group_by;
// pub mod relation_many_to_many;
pub mod relation_optional_to_many;
pub mod set_id_mod;
pub mod set_new_mod;
// pub mod relation_optional_to_many_inverse;
// pub mod set_id;
// pub mod set_new;

pub trait Link<Base> {
    type Spec;
    fn spec(self, base: &Base) -> Self::Spec;
}

/// link back Link::Spec to Base
pub trait LinkedToBase {
    type Base;
}

pub trait LinkedViaId {}
pub trait LinkedViaIds {}

pub trait CollectionsStore {
    type Store;
}

pub trait DynamicLink<DynamicBase: CollectionsStore, S> {
    type OnRequest;
    type OnRequestInput;
    type OnRequestError;

    fn on_request(
        &self,
        base: DynamicBase,
        input: Self::OnRequestInput,
    ) -> Result<Self::OnRequest, Self::OnRequestError>;

    type CreateLinkOk;
    type CreateLinkInput;
    type CreateLinkError;

    fn create_link(
        &self,
        store: &DynamicBase::Store,
        input: Self::CreateLinkInput,
    ) -> Result<Self::CreateLinkOk, Self::CreateLinkError>;

    type ModifyLinkOk;
    type ModifyLinkInput;
    type ModifyLinkError;

    fn modify_link(
        &self,
        store: &DynamicBase::Store,
        input: Self::ModifyLinkInput,
    ) -> Result<Self::ModifyLinkOk, Self::ModifyLinkError>;
}

mod functional_impls {
    use crate::links::Link;
    use paste::paste;

    macro_rules! implt {
    ( $([$t:ident, $part:literal])*) => {
        #[allow(unused)]
        impl<$($t,)* Base> Link<Base> for ($($t,)*)
        where
            $($t: Link<Base>,)*
        {
            type Spec = ( $($t::Spec,)* );
            fn spec(self, base: &Base) -> Self::Spec {
                ( $(paste!(self.$part).spec(base),)* )
            }
        }
    };
}

    implt!();
    implt!([T0, 0]);
    implt!([T0, 0] [T1, 1]);
    implt!([T0, 0] [T1, 1] [T2, 2]);
    implt!([T0, 0] [T1, 1] [T2, 2] [T3, 3]);

    impl<Base, T> Link<Base> for Vec<T>
    where
        T: Link<Base>,
    {
        type Spec = Vec<T::Spec>;

        fn spec(self, base: &Base) -> Self::Spec {
            self.into_iter().map(|e| e.spec(base)).collect()
        }
    }
}
