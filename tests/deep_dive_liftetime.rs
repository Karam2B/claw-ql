#![allow(unused)]
use std::marker::PhantomData;

struct CreateLifetime<'a>(PhantomData<(&'a (), &'a mut ())>);

impl<'a> CreateLifetime<'a> {
    fn add_let_statment_as_shared<T>(self, _: &'a T) -> CreateLifetime<'a> {
        CreateLifetime(PhantomData)
    }
    fn add_let_statment_as_mutable<T>(self, _: &'a mut T) -> CreateLifetime<'a> {
        CreateLifetime(PhantomData)
    }
}

#[rustfmt::skip]
fn lifetime_as_set_of_let_statments() {
    let mut shared_string = String::new();
    let mut mutable_string = String::new();

    let lifetime = CreateLifetime(PhantomData)
        .add_let_statment_as_shared(&shared_string)
        .add_let_statment_as_mutable(&mut mutable_string);

    // things you can't do:

    // &mut mutable_string;       // 1. no another mutable ref for mutable_string
    // &mutable_string;           // 2. no shared ref for mutable_string
    // &mut shared_string;        // 3. no mutable ref for shared_string

    // 4. and ofcourse no move of borrowed strings
    // drop(shared_string);      
    // drop(mutable_string);


    drop(lifetime);

    // let's consider all use cases where one lifetime is restricted by the other:

    // 1. `'static: 'doing_absolutely_nothing`
    // this is always the case because `'static` is the strictest/longest lifetime (the "top set"
    // from set theory perspective, all lifetimes are subsets of 'static)

    // 'dummpy_lifetime: 'static,
    // this require 'dummy_lifetime to be as strict/long as 'static,
    // meaning that these two examples are equivalent:
    //
    // `impl<'a> Type<'a> 'a: 'static {}`
    //
    // `impl Type<'static> {}`
    //
    // which make 'dummy_lifetime interchangable with 'static
    

    // 3. what about if you have two different lifetimes, one is restricted by the other? like `'a: 'b`?
    // 
    // this is similar to the second case, as was shown above, these two examples are equivalent:
    //
    // `impl<'a, 'b> Trait<'a> for Type<'b> where 'a: 'b {}`
    //
    // `impl<'a> Trait<'a> for Type<'a> {}`

    // meaning that you should never restrict a lifetime by another lifetime,
    // it is just another piece of code that confuses the programmer with no reasons,
    // in fact if you think of lifetimes as "wiring" (connect `let` statements to a region of
    // code, where access of let statement is restricted), what purpose does `'a: 'b` serve?

    // lifetimes are wiring:

    // there 2 things going on in this example 1. there is a region of code start at 5 and ends at 11, 2. there are two let statments that
    // are restricted inside that region of code

    // a nice way to defind lifetimes is as follows: lifetime represets a collection
    // of `let` statements/bindings and a region of code -- such as access to these binding is restricted inside 
    // that region of code -- that is from the line where lifetime was created to the 
    // line where it was dropped.
    // 
    // I just find it ingenius that in rust you are not allowed to use the keyword `let` outside of a function,
    // instead you have to use the keyword `static` to initiate values, because `'static` lifetime is the lifetime that contains no temporary (`let`) binding, or its set of `let` bindings is always empty

    // think about this snippet of code `impl<'a> Trait<'a> for String {}` is lifetime `'a` from a shared or
    // mutable reference? the syntax of Rust does't even let you know, because from the perspective of implementors, 
    // this doesn't matter, as implementor you have only two sinarios: 
    // 1. `impl<'a> Trait<'a> for Type<'a> {}`, 2.`impl<'a, 'b> Trait<'a> for Type<'b> {}`,
    // the first is to join two set of `let` statments together, and the second is to keep them seperate.
    // the first restrict whatever type holding 'a to be used before the earliest drop of any its bindings, the second makes no such restriction
    // (as explained `impl<'a, 'b> Trait<'a> for Type<'b> where 'a: 'b {}` equavalnt to the first)
    //

    // # Don't invoke set theory language

    // a common way to think about `'a: 'b` is to say that `'b` is subset of `'a`, but I find the perspective of set theory confusing.
    // The perspective of set theory is usefull in variance when asking "is `Type<'a>` assignable to `Type<'b>`?", 
    // meaning is the first a subset of the second, but I find it confusing when thinking about "is `'a` assignable to `'b`?"
    // because to think of lifetime as sets you might model it as follow 
    // 1. lifetime is a set of `let` statments, makes 'static an empty set (this is wrong way of thinking about it, because two sets can be dijoint where subsetting is irrelavent)
    // 2. lifetime is region of code, makes 'static the top set, represent all lines of code (also wrong because two region of code can be disjoint where subseting is irrelavant) 
    // 3. `'a: 'b` means region of "a" ends after region of "b" (this is the correct way to think about lifetimes, but as you can see, it has nothing to do about set thoery or about subsetting), in fact of read the colon as "subtype", and by subtype you mean "live longer", you have to xxx
    //
    // ``
    // because your job is to "wire"
    // this explains that from some perspective, you don't care about 

}
