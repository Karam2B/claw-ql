pub trait BuildTuple {
    const LEN: usize;
    type Bigger<N>;
    fn into_bigger<N>(self, n: N) -> Self::Bigger<N>;
}

pub trait TupleLastItem<const INDEX: i8> {
    type Last;
}

macro_rules! it {
    ($const:literal; $([$ty:ident, $part:literal]),*) => {

impl < $($ty,)*>
    BuildTuple
for ($($ty,)*)
{
    const LEN: usize = $const;
    type Bigger<N> = ($($ty,)* N,);
    fn into_bigger<N>(self, n: N) -> Self::Bigger<N> {
        ($(paste::paste!(self.$part),)* n,)
    }
}

    }; }

#[rustfmt::skip]
const _: () = {
    it!(0;);
    it!(1;  [T0, 0]);
    it!(2;  [T0, 0], [T1, 1]);
    it!(3;  [T0, 0], [T1, 1], [T2, 2]);
    it!(4;  [T0, 0], [T1, 1], [T2, 2], [T3, 3]);
    it!(5;  [T0, 0], [T1, 1], [T2, 2], [T3, 3], [T4, 4]);
    it!(6;  [T0, 0], [T1, 1], [T2, 2], [T3, 3], [T4, 4], [T5, 5]);
    it!(7;  [T0, 0], [T1, 1], [T2, 2], [T3, 3], [T4, 4], [T5, 5], [T6, 6]);
    it!(8;  [T0, 0], [T1, 1], [T2, 2], [T3, 3], [T4, 4], [T5, 5], [T6, 6], [T7, 7]);
    it!(9;  [T0, 0], [T1, 1], [T2, 2], [T3, 3], [T4, 4], [T5, 5], [T6, 6], [T7, 7], [T8, 8]);
    it!(10; [T0, 0], [T1, 1], [T2, 2], [T3, 3], [T4, 4], [T5, 5], [T6, 6], [T7, 7], [T8, 8], [T9, 9]);
    it!(11; [T0, 0], [T1, 1], [T2, 2], [T3, 3], [T4, 4], [T5, 5], [T6, 6], [T7, 7], [T8, 8], [T9, 9], [T10, 10]);
    it!(12; [T0, 0], [T1, 1], [T2, 2], [T3, 3], [T4, 4], [T5, 5], [T6, 6], [T7, 7], [T8, 8], [T9, 9], [T10, 10], [T11, 11]);
    it!(13; [T0, 0], [T1, 1], [T2, 2], [T3, 3], [T4, 4], [T5, 5], [T6, 6], [T7, 7], [T8, 8], [T9, 9], [T10, 10], [T11, 11], [T12, 12]);
};
