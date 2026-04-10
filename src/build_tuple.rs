pub trait BuildTuple {
    const LEN: usize;
    type Bigger<N>;
    fn into_bigger<N>(self, n: N) -> Self::Bigger<N>;
}

pub trait TupleLastItem {
    type Firsts;
    type Last;
    fn take(self) -> (Self::Firsts, Self::Last);
    fn get_ref(&self) -> &Self::Last;
    fn get_mut(&mut self) -> &mut Self::Last;
}

impl BuildTuple for () {
    const LEN: usize = 0;
    type Bigger<N> = (N,);
    fn into_bigger<N>(self, n: N) -> Self::Bigger<N> {
        (n,)
    }
}

impl<T0> BuildTuple for (T0,) {
    const LEN: usize = 0;
    type Bigger<N> = (T0, N);
    fn into_bigger<N>(self, n: N) -> Self::Bigger<N> {
        (self.0, n)
    }
}

impl<T0> TupleLastItem for (T0,) {
    type Firsts = ();

    type Last = T0;

    fn take(self) -> (Self::Firsts, Self::Last) {
        ((), self.0)
    }

    fn get_ref(&self) -> &Self::Last {
        &self.0
    }

    fn get_mut(&mut self) -> &mut Self::Last {
        &mut self.0
    }
}

macro_rules! it {
    ($const:literal; $([$ty:ident, $part:literal]),*; [$lty:ident, $lpart:literal]) => {
impl < $($ty,)* $lty>
    TupleLastItem
for ($($ty,)* $lty)
{
    type Firsts = ($($ty,)*);
    type Last = $lty;
    fn take(self) -> (Self::Firsts, Self::Last) {
        (
           ($(paste::paste!(self.$part),)*) ,
           paste::paste!(self.$lpart)
        )
    }
    fn get_ref(&self) -> &Self::Last {
       &paste::paste!(self.$lpart)
    }
    fn get_mut(&mut self) -> &mut Self::Last {
       &mut paste::paste!(self.$lpart)
    }
}
impl < $($ty,)* $lty>
    BuildTuple
for ($($ty,)* $lty)
{
    const LEN: usize = $const;
    type Bigger<N> = ($($ty,)* $lty, N,);
    fn into_bigger<N>(self, n: N) -> Self::Bigger<N> {
        ($(paste::paste!(self.$part),)* paste::paste!(self.$lpart), n,)
    }
}
    }; }

#[rustfmt::skip]
const _: () = {
it!(2;  [T0, 0]; [T1, 1]);
it!(3;  [T0, 0], [T1, 1]; [T2, 2]);
it!(4;  [T0, 0], [T1, 1], [T2, 2]; [T3, 3]);
it!(5;  [T0, 0], [T1, 1], [T2, 2], [T3, 3]; [T4, 4]);
it!(6;  [T0, 0], [T1, 1], [T2, 2], [T3, 3], [T4, 4]; [T5, 5]);
it!(7;  [T0, 0], [T1, 1], [T2, 2], [T3, 3], [T4, 4], [T5, 5]; [T6, 6]);
it!(8;  [T0, 0], [T1, 1], [T2, 2], [T3, 3], [T4, 4], [T5, 5], [T6, 6]; [T7, 7]);
it!(9;  [T0, 0], [T1, 1], [T2, 2], [T3, 3], [T4, 4], [T5, 5], [T6, 6], [T7, 7]; [T8, 8]);
it!(10;  [T0, 0], [T1, 1], [T2, 2], [T3, 3], [T4, 4], [T5, 5], [T6, 6], [T7, 7], [T8, 8]; [T9, 9]);
it!(11;  [T0, 0], [T1, 1], [T2, 2], [T3, 3], [T4, 4], [T5, 5], [T6, 6], [T7, 7], [T8, 8], [T9, 9]; [T10, 10]);
it!(12;  [T0, 0], [T1, 1], [T2, 2], [T3, 3], [T4, 4], [T5, 5], [T6, 6], [T7, 7], [T8, 8], [T9, 9], [T10, 10]; [T11, 11]);
it!(13;  [T0, 0], [T1, 1], [T2, 2], [T3, 3], [T4, 4], [T5, 5], [T6, 6], [T7, 7], [T8, 8], [T9, 9], [T10, 10], [T11, 11]; [T12, 12]);
it!(14;  [T0, 0], [T1, 1], [T2, 2], [T3, 3], [T4, 4], [T5, 5], [T6, 6], [T7, 7], [T8, 8], [T9, 9], [T10, 10], [T11, 11], [T12, 12]; [T13, 13]);
it!(15;  [T0, 0], [T1, 1], [T2, 2], [T3, 3], [T4, 4], [T5, 5], [T6, 6], [T7, 7], [T8, 8], [T9, 9], [T10, 10], [T11, 11], [T12, 12], [T13, 13]; [T14, 14]);
};
