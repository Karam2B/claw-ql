use claw_ql::build_tuple::BuildTuple;

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

pub trait AddCollection<Next>: BuildContext {
    fn add_col(next: &Next, ctx: &mut Self::Context);
}

pub trait AddLink<Next>: BuildContext {
    fn add_link(next: &Next, ctx: &mut Self::Context);
}

pub trait Finish: BuildContext {
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
        B: AddCollection<Cn>,
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
        B: AddLink<Ln>,
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
        B: Finish,
    {
        self.__build_mode.finish(self.__build_ctx)
    }
}
