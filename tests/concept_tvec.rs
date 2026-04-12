#![cfg(feature = "skip_without_comment")]
pub struct IVec<T: ?Sized> {
    pub inner: Vec<Box<T>>,
}

impl<T: ?Sized> Default for IVec<T> {
    fn default() -> Self {
        IVec {
            inner: Default::default(),
        }
    }
}

pub trait ToHeap<S> {
    fn to_heap(s: S) -> Box<Self>;
}

pub trait MyTrait {
    fn call(&self) {}
}

impl<T: MyTrait + 'static> ToHeap<T> for dyn MyTrait {
    fn to_heap(s: T) -> Box<Self> {
        Box::new(s)
    }
}

impl<T: ?Sized> IVec<T> {
    pub fn push<S>(&mut self, _a: S)
    where
        T: ToHeap<S>,
    {
    }
}

pub struct TVec<T: ?Sized> {
    pub inner: Vec<Box<T>>,
}

pub struct TVecIter<T> {
    pub index: usize,
    pub tvec: TVec<T>,
}

impl<T> Iterator for TVecIter<T> {
    type Item = Box<T>;
    fn next(&mut self) -> Option<Self::Item> {
        todo!()
    }
}

impl<T: ?Sized> IVec<T> {
    pub fn finish(self) -> TVec<T> {
        todo!()
    }
}

pub struct IncorrectType;

impl<T: ?Sized> TVec<T> {
    pub fn get_typed<S>(_index: usize) -> Result<S, IncorrectType> {
        todo!()
    }
}

impl MyTrait for () {}
impl MyTrait for &str {}

fn map<S: ?Sized>(_: TVec<S>, _: impl FnOnce(Box<S>)) {}

fn main() {
    let mut ivec = IVec::<dyn MyTrait>::default();

    ivec.push(());
    ivec.push("bar");
    // bugs `if runtimeval { ivec.push(()) }`
    // sulotion
    // const if true { ivec.push(a) }
    // but how to mark push method to be callable in const-if's

    let tvec = ivec.finish();

    // I cannot push on tvec

    map(tvec, |e| e.call());
}

fn _optimized() {
    let ivec = ((), "bar");

    // let tvec = ivec.finish();

    // I cannot push on tvec

    ivec.0.call();
    ivec.1.call();
}
