pub trait BuildTuple {
    type Bigger<N>;
    fn into_bigger<N>(self, n: N) -> Self::Bigger<N>;
}

pub trait Tuple<TupleSpec> {
    type Output;
    fn on_all_only_mut(self, tuple_spec: TupleSpec) -> Self::Output;
}

pub trait TupleSpec<Member> {
    type Output;
    fn on_each<const LAST_INDEX: usize, const INDEX: usize>(
        &mut self,
        member: Member,
    ) -> Self::Output;
}

pub trait TupleLast<TupleLastSpec> {
    type Output;
    fn on_all(self, tuple_spec: TupleLastSpec) -> Self::Output;
}

pub trait TupleLastSpec<Member, Last> {
    type Output;
    type LastOutput;
    fn on_each<const LAST_INDEX: usize, const INDEX: usize>(
        &mut self,
        member: Member,
    ) -> Self::Output;
    fn on_last<const INDEX: usize>(self, member: Last) -> Self::LastOutput;
}

pub trait TupleAsRef<'a> {
    type Output: 'a;
    fn tuple_as_ref(&'a self) -> Self::Output;
}

pub trait TupleAsMut<'a> {
    type Output: 'a;
    fn tuple_as_mut(&'a mut self) -> Self::Output;
}

pub struct Info<I, O> {
    pub input_info: I,
    pub output_info: O,
}

pub trait FnInfo<Input, Output> {
    type OutputInfo;
    type InputInfo;
    fn info(self) -> Info<Self::InputInfo, Self::OutputInfo>;
}

#[allow(non_camel_case_types)]
pub struct as_last_spec<S>(pub S);

impl<M, L, S> TupleLastSpec<M, L> for as_last_spec<S>
where
    S: TupleSpec<M>,
    S: TupleSpec<L>,
{
    type LastOutput = <S as TupleSpec<L>>::Output;
    type Output = <S as TupleSpec<M>>::Output;
    fn on_last<const INDEX: usize>(mut self, member: L) -> Self::LastOutput {
        TupleSpec::on_each::<INDEX, INDEX>(&mut self.0, member)
    }
    fn on_each<const LEN: usize, const INDEX: usize>(&mut self, member: M) -> Self::Output {
        TupleSpec::on_each::<LEN, INDEX>(&mut self.0, member)
    }
}

macro_rules! implt {
    ($($num:literal)* size $size:literal) => {
        paste::paste!(
            impl<$([<T $num>],)*> BuildTuple for ($([<T $num>],)*)
            {
                type Bigger<N> = ($([<T $num>],)* N,);
                fn into_bigger<N>(self, n: N) -> Self::Bigger<N> {
                    ($(paste::paste!(self.$num),)* n,)
                }
            }

            impl<'a, $([<T $num>]:'a,)*> TupleAsRef<'a> for ($([<T $num>],)*)
            {
                type Output = (
                    $(&'a [<T $num>],)*
                );
                fn tuple_as_ref(&'a self) -> Self::Output {
                    (
                        $(&self.$num,)*
                    )
                }
            }

            impl<'a, $([<T $num>]:'a,)*> TupleAsMut<'a> for ($([<T $num>],)*)
            {
                type Output = (
                    $(&'a mut [<T $num>],)*
                );
                fn tuple_as_mut(&'a mut self) -> Self::Output {
                    (
                        $(&mut self.$num,)*
                    )
                }
            }

            impl<$([<T $num>],)* TS> Tuple<TS> for ($([<T $num>],)*)
            where
                $(TS: TupleSpec<[<T $num>]>,)*
            {
                type Output = (
                    $(<TS as TupleSpec<[<T $num>]> >::Output,)*
                );
                fn on_all_only_mut(self, #[allow(unused)] mut tuple_spec: TS) -> Self::Output {
                    (
                        $(<TS as TupleSpec<[<T $num>]> >::on_each::<$size,$num>(&mut tuple_spec, self.$num),)*
                    )
                }
            }

            impl<F, O, $([<I $num>],)*> FnInfo<($([<I $num>],)*), O> for F
            where
                F: FnOnce($([<I $num>],)*) -> O,
            {
                type InputInfo = (
                    $(std::marker::PhantomData<[<I $num>]>,)*
                );
                type OutputInfo = std::marker::PhantomData<O>;
                fn info(self) -> Info<Self::InputInfo, Self::OutputInfo> {
                    Info {
                        input_info: (
                            $(std::marker::PhantomData::<[<I $num>]>,)*
                        ),
                        output_info: std::marker::PhantomData,
                    }
                }
            }

        );

    };
    ($($num:literal)* last $last:literal) => {
        implt!($($num)* $last size $last);
        paste::paste!(
            impl<$([<T $num>],)* [<T $last>], TS> TupleLast<TS> for ($([<T $num>],)* [<T $last>],)
            where
                $(TS: TupleLastSpec<[<T $num>], [<T $last>]>,)*
                TS: TupleLastSpec<[<T $last>], [<T $last>]>,
            {
                type Output = (
                    $(<TS as TupleLastSpec<[<T $num>], [<T $last>]>>::Output,)*
                    <TS as TupleLastSpec<[<T $last>], [<T $last>]>>::LastOutput,
                );
                fn on_all(self, #[allow(unused)] mut tuple_spec: TS) -> Self::Output {
                    (
                        $(<TS as TupleLastSpec<[<T $num>], [<T $last>]> >::on_each::<$last,$num>(&mut tuple_spec, self.$num),)*
                        <TS as TupleLastSpec<[<T $last>], [<T $last>]> >::on_last::<$last>(tuple_spec, self.$last),
                    )
                }
            }


        );
    };
}

impl<TS> TupleLast<TS> for () {
    type Output = ();
    fn on_all(self, _: TS) -> Self::Output {}
}

implt!(size 0);
implt!(last 0);
implt!(0 last 1);
implt!(0 1 last 2);
implt!(0 1 2 last 3);
implt!(0 1 2 3 last 4);
implt!(0 1 2 3 4 last 5);
implt!(0 1 2 3 4 5 last 6);
implt!(0 1 2 3 4 5 6 last 7);
implt!(0 1 2 3 4 5 6 7 last 8);
implt!(0 1 2 3 4 5 6 7 8 last 9);
implt!(0 1 2 3 4 5 6 7 8 9 last 10);
implt!(0 1 2 3 4 5 6 7 8 9 10 last 11);
implt!(0 1 2 3 4 5 6 7 8 9 10 11 last 12);
implt!(0 1 2 3 4 5 6 7 8 9 10 11 12 last 13);
implt!(0 1 2 3 4 5 6 7 8 9 10 11 12 13 last 14);
implt!(0 1 2 3 4 5 6 7 8 9 10 11 12 13 14 last 15);

#[cfg(test)]
mod basic_example {
    #![allow(unused)]
    #![deny(unused_must_use)]
    #![allow(non_camel_case_types)]

    use crate::tuple_trait::{Tuple, TupleAsRef, TupleSpec};
    use core::fmt;
    use std::marker::PhantomData;

    struct join<'q>(&'q mut String);

    impl<M> TupleSpec<&M> for join<'_>
    where
        M: fmt::Display,
    {
        type Output = ();
        fn on_each<const LEN: usize, const INDEX: usize>(&mut self, member: &M) -> Self::Output {
            self.0.push_str(format!("{}", member).as_str());
            if LEN != INDEX {
                self.0.push_str(", ");
            }
        }
    }

    #[test]
    fn use_spec() {
        let mut str = String::new();

        let tuple = (3, "hello world", true);

        TupleAsRef::tuple_as_ref(&tuple).on_all_only_mut(join(&mut str));

        let _ = tuple;

        assert_eq!(str.as_str(), "3, hello world, true")
    }

    struct phantomize;
    impl<M> TupleSpec<&M> for phantomize {
        type Output = PhantomData<M>;
        fn on_each<const LEN: usize, const INDEX: usize>(&mut self, member: &M) -> Self::Output {
            PhantomData
        }
    }

    #[test]
    fn phantom_exampe() {
        let tuple = (3, "hello world", true);

        let phantoms: (
            // phantoms types
            PhantomData<usize>,
            PhantomData<&str>,
            PhantomData<bool>,
        ) = tuple.tuple_as_ref().on_all_only_mut(phantomize);
    }
}
