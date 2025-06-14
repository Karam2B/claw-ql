use crate::build_tuple::BuildTuple;

pub struct BuilderPattern<BuildMode: BuildContext, Collections, Links, Filters> {
    __build_mode: BuildMode,
    __build_ctx: BuildMode::Context,
    __collections: Collections,
    __links: Links,
    __filters: Filters,
}

pub trait BuildContext {
    type Context;
    fn init_context(&self) -> Self::Context;
}

pub trait AddCollection<Tuple, Next>: BuildContext {
    fn add_col(next: &Next, ctx: &mut Self::Context);
}

pub trait AddLink<Tuple, Next>: BuildContext {
    fn add_link(next: &Next, ctx: &mut Self::Context);
}

pub trait Finish<C>: BuildContext {
    type Result;
    fn finish(self, ctx: Self::Context) -> Self::Result;
}

impl Default for BuilderPattern<(), (), (), ()> {
    fn default() -> Self {
        BuilderPattern {
            __build_mode: (),
            __build_ctx: (),
            __collections: (),
            __links: (),
            __filters: (),
        }
    }
}

impl<B> BuilderPattern<B, (), (), ()>
where
    B: BuildTuple + BuildContext,
{
    pub fn build_mode<BM>(self, build_mode: BM) -> BuilderPattern<B::Bigger<BM>, (), (), ()>
    where
        BM: BuildContext,
        B::Context: BuildTuple,
        B::Bigger<BM>: BuildContext,
    {
        let build_mode = self.__build_mode.into_bigger(build_mode);
        let build_ctx = build_mode.init_context();
        BuilderPattern {
            __build_ctx: build_ctx,
            __build_mode: build_mode,
            __collections: (),
            __links: (),
            __filters: (),
        }
    }
}

impl<B, C, L, F> BuilderPattern<B, C, L, F>
where
    B: BuildTuple + BuildContext,
    C: BuildTuple,
    L: BuildTuple,
    F: BuildTuple,
{
    pub fn add_collection<Cn>(mut self, collection: Cn) -> BuilderPattern<B, C::Bigger<Cn>, L, F>
    where
        B: AddCollection<C, Cn>,
    {
        B::add_col(&collection, &mut self.__build_ctx);
        BuilderPattern {
            __build_mode: self.__build_mode,
            __build_ctx: self.__build_ctx,
            __collections: self.__collections.into_bigger(collection),
            __links: self.__links,
            __filters: self.__filters,
        }
    }
    pub fn add_link<Ln>(mut self, link: Ln) -> BuilderPattern<B, C, L::Bigger<Ln>, F>
    where
        B: AddLink<L, Ln>,
    {
        B::add_link(&link, &mut self.__build_ctx);
        BuilderPattern {
            __build_mode: self.__build_mode,
            __build_ctx: self.__build_ctx,
            __collections: self.__collections,
            __links: self.__links.into_bigger(link),
            __filters: self.__filters,
        }
    }
    pub fn finish(self) -> B::Result
    where
        B: Finish<C>,
    {
        self.__build_mode.finish(self.__build_ctx)
    }
}

macro_rules! it {
    ($([$ty:ident, $part:literal]),*) => {

impl<$($ty,)*> BuildContext for ($($ty,)*)
    where $($ty: BuildContext,)*
{
    type Context = ( $($ty::Context,)*);
    fn init_context(&self) -> Self::Context {
        ($(paste::paste!(self.$part.init_context()),)*)
    }
}

impl<Next,Tuple, $($ty),* > AddLink<Tuple, Next> for ($($ty,)*)
    where $($ty: BuildContext + AddLink<Tuple, Next>,)*
{
    fn add_link(next: &Next, ctx: &mut Self::Context) {
        $($ty::add_link(next, &mut paste::paste!(ctx.$part));)*
    }
}

impl<Next,Tuple, $($ty),* > AddCollection<Tuple, Next> for ($($ty,)*)
    where $($ty: BuildContext + AddCollection<Tuple, Next>,)*
{
    fn add_col(next: &Next, ctx: &mut Self::Context) {
        $($ty::add_col(next, &mut paste::paste!(ctx.$part));)*
    }
}

impl<C, $($ty,)*> Finish<C> for ($($ty,)*)
    where $($ty: BuildContext+Finish<C>,)*
{
    type Result = ($($ty::Result,)*);
    fn finish(self, ctx: Self::Context) -> Self::Result {
        ($(paste::paste!(self.$part.finish(ctx.$part)),)*)
    }
}

    }}

#[rustfmt::skip]
#[allow(unused)]
const _: () = {
    it!();
    it!([R0, 0]);
    it!([R0, 0], [R1, 1]);
    it!([R0, 0], [R1, 1], [R2, 2]);
    it!([R0, 0], [R1, 1], [R2, 2], [R3, 3]);
    it!([R0, 0], [R1, 1], [R2, 2], [R3, 3], [R4, 4]);
    it!([R0, 0], [R1, 1], [R2, 2], [R3, 3], [R4, 4], [R5, 5]);
    it!([R0, 0], [R1, 1], [R2, 2], [R3, 3], [R4, 4], [R5, 5], [R6, 6]);
    it!([R0, 0], [R1, 1], [R2, 2], [R3, 3], [R4, 4], [R5, 5], [R6, 6], [R7, 7]);
    it!([R0, 0], [R1, 1], [R2, 2], [R3, 3], [R4, 4], [R5, 5], [R6, 6], [R7, 7], [R8, 8]);
    it!([R0, 0], [R1, 1], [R2, 2], [R3, 3], [R4, 4], [R5, 5], [R6, 6], [R7, 7], [R8, 8], [R9, 9]);
    it!([R0, 0], [R1, 1], [R2, 2], [R3, 3], [R4, 4], [R5, 5], [R6, 6], [R7, 7], [R8, 8], [R9, 9], [R10, 10]);
    it!([R0, 0], [R1, 1], [R2, 2], [R3, 3], [R4, 4], [R5, 5], [R6, 6], [R7, 7], [R8, 8], [R9, 9], [R10, 10], [R11, 11]);
};
