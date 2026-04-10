pub trait IntoInferFromPhantom<I> {
    fn into_pd(self, _: PhantomData<I>) -> I;
}

impl<F, I> IntoInferFromPhantom<I> for F
where
    I: From<F>,
{
    #[inline]
    fn into_pd(self, _: PhantomData<I>) -> I {
        self.into()
    }
}
