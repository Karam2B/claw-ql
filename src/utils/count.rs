pub trait Count {
    fn count(&self) -> usize;
}

#[rustfmt::skip]
mod impls {
    use super::Count;

    impl<T> Count for Vec<T> { fn count(&self) -> usize { self.len() } }
    impl<T> Count for Option<T> { fn count(&self) -> usize { if self.is_some() { 1 } else { 0 } } }
    impl Count for () { fn count(&self) -> usize { 0 } }
    impl<T> Count for (T,) { fn count(&self) -> usize { 1 } }
    impl<T0, T1> Count for (T0, T1) { fn count(&self) -> usize { 2 } }
    impl<T0, T1, T2> Count for (T0, T1, T2) { fn count(&self) -> usize { 3 } }

}
