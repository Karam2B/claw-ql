use std::marker::PhantomData;

use crate::build_tuple::BuildTuple;

pub trait InitializeContext {
    type Context;
    fn initialize_context(self) -> Self::Context;
}

pub trait AddCollection<Collection> {
    type This;
    type Context;
    type NextContext;
    fn build_component(collection: &Collection, ctx: Self::Context) -> Self::NextContext;
}

pub trait AddLink<Link> {
    type This;
    type Context;
    type NextContext;
    fn build_component(link: &Link, ctx: Self::Context) -> Self::NextContext;
}

pub trait Finish {
    type Context;
    type Result;
    fn build_component(ctx: Self::Context) -> Self::Result;
}

pub struct BuilderPattern<Components, Context> {
    pub(crate) __components: Components,
    pub(crate) __context: Context,
}

impl Default for BuilderPattern<(), ()> {
    fn default() -> Self {
        BuilderPattern {
            __components: (),
            __context: (),
        }
    }
}

impl<B> BuilderPattern<B, ()>
where
    B: InitializeContext,
{
    pub fn build_component<BuildComponent>(
        self,
        build_component: BuildComponent,
    ) -> BuilderPattern<B::Bigger<BuildComponent>, ()>
    where
        B: BuildTuple,
        B::Context: BuildTuple,
        B::Bigger<BuildComponent>: InitializeContext,
    {
        let build_mode = self.__components.into_bigger(build_component);
        BuilderPattern {
            __context: (),
            __components: build_mode,
        }
    }
    pub fn start(self) -> BuilderPattern<PhantomData<B>, B::Context> {
        BuilderPattern {
            __components: PhantomData,
            __context: self.__components.initialize_context(),
        }
    }
}

impl<Components, Ctx> BuilderPattern<PhantomData<Components>, Ctx> {
    #[track_caller]
    pub fn add_collection<Cnext>(
        self,
        collection: Cnext,
    ) -> BuilderPattern<PhantomData<Components::This>, Components::NextContext>
    where
        Components: AddCollection<Cnext, Context = Ctx>,
    {
        let ctx = Components::build_component(&collection, self.__context);

        BuilderPattern {
            __components: PhantomData,
            __context: ctx,
        }
    }
    #[track_caller]
    pub fn add_link<Lnext>(
        self,
        link: Lnext,
    ) -> BuilderPattern<PhantomData<Components::This>, Components::NextContext>
    where
        Components: AddLink<Lnext, Context = Ctx>,
    {
        let ctx = Components::build_component(&link, self.__context);

        BuilderPattern {
            __components: PhantomData,
            __context: ctx,
        }
    }

    #[track_caller]
    pub fn finish(self) -> Components::Result
    where
        Components: Finish<Context = Ctx>,
    {
        Components::build_component(self.__context)
    }
}

macro_rules! it {
    ($([$ty:ident, $part:literal]),*) => {

impl<$($ty,)*> InitializeContext for ($($ty,)*)
    where $($ty: InitializeContext,)*
{
    type Context = ( $($ty::Context,)*);
    fn initialize_context(self) -> Self::Context {
        ($(paste::paste!(self.$part.initialize_context()),)*)
    }
}

impl<Next,$($ty),* > AddLink<Next> for ($($ty,)*)
    where $($ty:  AddLink<Next>,)*
{

    type This = (
        $($ty::This,)*
    );
    type Context = (
        $($ty::Context,)*
    );
    type NextContext = (
        $($ty::NextContext,)*
    );
    #[track_caller]
    fn build_component(next: &Next, ctx: Self::Context) -> Self::NextContext {
        ($($ty::build_component(next, paste::paste!(ctx.$part)),)*)
    }
}

impl<Next, $($ty),* > AddCollection<Next> for ($($ty,)*)
    where $($ty: AddCollection<Next>,)*
{
    type This = (
        $($ty::This,)*
    );
    type Context = (
        $($ty::Context,)*
    );
    type NextContext = (
        $($ty::NextContext,)*
    );
    #[track_caller]
    fn build_component(next: &Next, ctx: Self::Context) -> Self::NextContext {
        ($($ty::build_component(next, paste::paste!(ctx.$part)),)*)
    }
}

impl<$($ty,)*> Finish for ($($ty,)*)
    where $($ty: Finish,)*
{
    type Result = ($($ty::Result,)*);
    type Context = (
        $($ty::Context,)*
    );
    #[track_caller]
    fn build_component(ctx: Self::Context) -> Self::Result {
        ($(paste::paste!($ty::build_component(ctx.$part)),)*)
    }
}

    }}

#[rustfmt::skip]
#[allow(unused)]
const _: () = {
    it!{}
    it!{[R0, 0]}
    it!{[R0, 0], [R1, 1]}
    it!{[R0, 0], [R1, 1], [R2, 2]}
    it!{[R0, 0], [R1, 1], [R2, 2], [R3, 3]}
    it!{[R0, 0], [R1, 1], [R2, 2], [R3, 3], [R4, 4]}
    it!{[R0, 0], [R1, 1], [R2, 2], [R3, 3], [R4, 4], [R5, 5]}
    it!{[R0, 0], [R1, 1], [R2, 2], [R3, 3], [R4, 4], [R5, 5], [R6, 6]}
    it!{[R0, 0], [R1, 1], [R2, 2], [R3, 3], [R4, 4], [R5, 5], [R6, 6], [R7, 7]}
    it!{[R0, 0], [R1, 1], [R2, 2], [R3, 3], [R4, 4], [R5, 5], [R6, 6], [R7, 7], [R8, 8]}
    it!{[R0, 0], [R1, 1], [R2, 2], [R3, 3], [R4, 4], [R5, 5], [R6, 6], [R7, 7], [R8, 8], [R9, 9]}
    it!{[R0, 0], [R1, 1], [R2, 2], [R3, 3], [R4, 4], [R5, 5], [R6, 6], [R7, 7], [R8, 8], [R9, 9], [R10, 10]}
    it!{[R0, 0], [R1, 1], [R2, 2], [R3, 3], [R4, 4], [R5, 5], [R6, 6], [R7, 7], [R8, 8], [R9, 9], [R10, 10], [R11, 11]}
};
