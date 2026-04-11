#![allow(unused)]
#![allow(non_camel_case_types)]

use core::fmt;

pub trait Tuple<TupleSpec> {
    type Output;
    fn on_each(self, tuple_spec: TupleSpec) -> Self::Output;
}

impl<T0, T1, TS> Tuple<TS> for (T0, T1)
where
    TS: TupleSpec<Self, T0>,
    TS: TupleSpec<Self, T1>,
{
    type Output = (
        <TS as TupleSpec<Self, T0>>::Output,
        <TS as TupleSpec<Self, T1>>::Output,
    );
    fn on_each(self, tuple_spec: TS) -> Self::Output {
        (
            <TS as TupleSpec<Self, T0>>::on_each(tuple_spec, self.0),
            <TS as TupleSpec<Self, T1>>::on_each(tuple_spec, self.1),
        )
    }
}

pub trait TupleSpec<Tuple, Member>: Copy {
    type Output;
    fn on_each(self, member: Member) -> Self::Output;
}

mod foriegn_crate {
    use crate::TupleSpec;
    use core::fmt;

    #[derive(Clone, Copy)]
    struct my_ident;

    impl<T, M> TupleSpec<T, M> for my_ident
    where
        M: fmt::Debug,
    {
        type Output = String;
        fn on_each(self, member: M) -> Self::Output {
            format!("{member:?}")
        }
    }
}
