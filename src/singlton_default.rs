//! example of how this is used in code
//!
//! `
//!     fn collection_basic<T: CollectionBasic>(t: T) {}
//!     // these are the same calls because todo: ZeroSizedDefault
//!     collection_basic(todo)
//!     collection_basic(PhantomData::<Todo>)
//!
//!     pub struct Todo {
//!         pub title: String,
//!     }
//!
//!     impl HasHandler for Todo {
//!         type Handler = todo;
//!     }
//!
//!     #[derive(Default)]
//!     #[allow(non_camel_case_types)]
//!     pub struct todo;
//!
//!     impl ZeroSizeDefault for todo {
//!         fn zero_size_default() -> &'static Self {
//!             &todo
//!         }
//!     }
//!
//!     impl CollectionBasic for todo {
//!         fn table_name(&self) -> &str {
//!             "Todo"
//!         }
//!
//!         fn table_name_lower_case(&self) -> &str {
//!             "todo"
//!         }
//!     }
//!
//!     impl<T> CollectionBasic for PhantomData<T>
//!     where
//!         T: HasHandler,
//!         T::Handler: ZeroSizeDefault,
//!     {
//!         fn table_name(&self) -> &str {
//!             T::Handler::table_name(ZeroSizeDefault::zero_size_default())
//!         }
//!         fn table_name_lower_case(&self) -> &str {
//!             T::Handler::table_name_lower_case(ZeroSizeDefault::zero_size_default())
//!         }
//!     }
//! `
//!

pub trait SingltonDefault: 'static {
    fn singlton_default() -> &'static Self;
}

pub mod impl_collectinos {
    use super::SingltonDefault;
    use crate::collections::{Collection, CollectionBasic, HasHandler};
    use std::marker::PhantomData;

    impl<T> CollectionBasic for PhantomData<T>
    where
        T: HasHandler,
        T::Handler: CollectionBasic + SingltonDefault,
    {
        fn table_name(&self) -> &str {
            T::Handler::table_name(SingltonDefault::singlton_default())
        }
        fn table_name_lower_case(&self) -> &str {
            T::Handler::table_name_lower_case(SingltonDefault::singlton_default())
        }
    }

    impl<T> Collection for PhantomData<T>
    where
        T: HasHandler,
        T::Handler: Collection + SingltonDefault,
    {
        type Partial = <T::Handler as Collection>::Partial;

        type Data = <T::Handler as Collection>::Data;

        type Id = <T::Handler as Collection>::Id;

        fn id(&self) -> &Self::Id {
            T::Handler::id(SingltonDefault::singlton_default())
        }
    }
}
