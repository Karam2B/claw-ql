use std::{marker::PhantomData, path::Components};

use crate::build_tuple::BuildTuple;

pub trait BuildStep<'s, Ident, Step> {
    type NextSelf;
    fn build_step(self, step: &'s Step) -> Self::NextSelf;
}

pub trait BuildMutStep<Ident, Step> {
    fn build_step(&mut self, step: &Step);
}

pub struct Preparing<T>(T);
pub struct AsOwn<T>(T);
pub struct AsMut<T>(T);

pub struct BuilderPattern<Components> {
    pub(crate) components: Components,
}

impl Default for BuilderPattern<Preparing<()>> {
    fn default() -> Self {
        BuilderPattern {
            components: Preparing(()),
        }
    }
}

impl<B> BuilderPattern<Preparing<B>> {
    pub fn build_component<BuildComponent>(
        self,
        build_component: BuildComponent,
    ) -> BuilderPattern<Preparing<B::Bigger<BuildComponent>>>
    where
        B: BuildTuple,
    {
        let build_mode = self.components.0.into_bigger(build_component);
        BuilderPattern {
            components: Preparing(build_mode),
        }
    }
    pub fn start_mut(self) -> BuilderPattern<AsMut<B>> {
        BuilderPattern {
            components: AsMut(self.components.0),
        }
    }
    pub fn start_own(self) -> BuilderPattern<AsOwn<B>> {
        BuilderPattern {
            components: AsOwn(self.components.0),
        }
    }
}

#[allow(non_camel_case_types)]
pub struct collection;
#[allow(non_camel_case_types)]
pub struct link;

impl<Components> BuilderPattern<AsOwn<Components>> {
    #[track_caller]
    pub fn add_collection<'a, Next>(
        self,
        step: &'a Next,
    ) -> BuilderPattern<AsOwn<Components::NextSelf>>
    where
        Components: BuildStep<'a, collection, Next>,
    {
        BuilderPattern {
            components: AsOwn(self.components.0.build_step(&step)),
        }
    }
    #[track_caller]
    pub fn add_link<'a, Next>(self, step: &'a Next) -> BuilderPattern<AsOwn<Components::NextSelf>>
    where
        Components: BuildStep<'a, link, Next>,
    {
        BuilderPattern {
            components: AsOwn(self.components.0.build_step(&step)),
        }
    }
    pub fn finish(self) -> Components {
        self.components.0
    }
}

impl<Components> BuilderPattern<AsMut<Components>> {
    #[track_caller]
    pub fn add_collection<Next>(&mut self, step: &Next)
    where
        Components: BuildMutStep<collection, Next>,
    {
        self.components.0.build_step(step);
    }
    #[track_caller]
    pub fn add_link<Next>(&mut self, step: &Next)
    where
        Components: BuildMutStep<link, Next>,
    {
        self.components.0.build_step(step);
    }
    pub fn finish(self) -> Components {
        self.components.0
    }
}

macro_rules! it {
    ($([$ty:ident, $part:literal]),*) => {

impl<Ident, Next,$($ty),* > BuildMutStep<Ident, Next> for ($($ty,)*)
    where $($ty:  BuildMutStep<Ident, Next>,)*
{
    fn build_step(&mut self, step: &Next) {
        $(paste::paste!(self.$part.build_step(step));)*
    }
}
impl<'a, Ident, Next,$($ty),* > BuildStep<'a, Ident, Next> for ($($ty,)*)
    where $($ty:  BuildStep<'a, Ident, Next>,)*
{
    type NextSelf = (
        $($ty::NextSelf,)*
    );
    fn build_step(self, step: &'a Next) -> Self::NextSelf{
        ($(paste::paste!(self.$part.build_step(step)),)*)
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
