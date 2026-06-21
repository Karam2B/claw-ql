

The code base can be thought as two parts: 

# 1. Human specified code

These are parts of the code that I want to manually specify, these include: traits, structs, enums, internal macros and manual vtables.  if you think a modification is necessary, write a review of you suggesstions and I will update them manually.

# 2. AI generated code

This is the code that AI agents can generate based on the prompt provided, these are impls blocks, function bodies, macro invocations.

In function bodies, I may provide more instruction inside the `todo!($instruction)`s. that you can use

You can generate new functions if that helps writing better code, but these can never be `pub` or `pub(crate)`, i.e. they are used internally by the AI generated code.

You can generate new struct, enum, if you that is required to correctly implement a trait of a struct, enum I manually added.

# 3. need permission patterns

There are some patterns that you should never use, if you think it is needed for whatever reason, provide a summary for me and I will update that manually. these includes:
1. unsafe code

# When you have access to generic, never specify a specific type
when having access to generic type `T`, never write `if TypeId::of::<T>() == TypeId::of::<DesiredType>() {}` ever never, check what behaviour you needed from `DesiredType` and introduce its trait where you introduce `T`. example:

never do this:
```rust
fn example<T>() {
    if TypeId::of::<T>() == TypeId::of::<String>() {
        <DesiredType as Trait>::behavior();
    }
}
```

instead do this:

```rust
fn example<T: Trait>() {
    T::behavior();
}
```

a similar pattern I saw AI doing, you should never do is `struct Client<S: sqlx::Database> { pool: Pool<S>, _why: Weak<Client<Sqlite>> _db: PhantomData<S>}`

# generating tests
When doing asserts within a test, use `pretty_assertions::assert_eq!` instead of `assert_eq!` to get better error messages.
When doing asserts within a test, don't do `assert_eq!(whole.field1, { .. });assert_eq!(whole.field2, { .. })`, instead do `pretty_assertions::assert_eq!(whole, { .. });`, add `derive(Debug, PartialEq, Eq)` to the structs when necessary.

# types of error
Errors can be categorized into two type (relative to a level of abstraction): 1. bugs and devops errors that you should always panic over 2. errors that are propegated to a different level of abstraction.

Here are examples of that

## JsonClient:
JsonClient runs in the backend, and there is client on the frontend that may send invalid requests, from the perspective of the backend client "JsonClient", the next level of abstraction is usually the frontend client. A perfect frontend client should never compile on possible invalid requests, but it may have bugs. Only errors that the frontend client can cause are propegated via `Result` inside `JsonClient::$operation` calls.

# Writing where predicates

here are some tips on writing correct where predicates:

1. if you specify a trait as a bound, you don't have to specify the trait's super traits as bounds, for example:
```rust
pub trait Trait {}
pub trait Trait2: Trait {}
fn example<T: Trait + Trait2 >() {
    todo!()
}
```
is equivalent to:
```rust
fn example<T: Trait>() {
    todo!()
}
```

I perfer to keep as less predicates as possible, so I would only specify `Trait`

2. traits bounds on generic types

```rust
pub struct SomeType {
    pub vec: Vec<OtherType>,
}

impl<'de, S> Deserialize<'de> for SomeType
where 
    Vec<OtherType>: Deserialize<'de>,
{

}

impl<'de, S, T> Deserialize<'de> for Vec<T> 
where T: $bound 
{

}

```

is equivalent to 

```rust

impl<'de, S> Deserialize<'de> for SomeType
where 
    OtherType: $bound
{

}

```

I'm hisitant to know which one is better, I perfer if you made a decision to inform me after each implementation.

3. never subtype lifetime bounds

```rust
impl<'a, 'b> Trait<'a> for Type<'b> where 'a: 'b {
    todo!()
}
```

is equivalent to:

```rust
impl<'a> Trait<'a> for Type<'a> {
    todo!()
}
```

Subtyping lifetime is just a source of confusion in my own experience.

# useing '{', '}', '(', and ')'
if these are using inside a string, they will mess up with my LSP experience, please define them as const and import them

```rust
const OPEN_PARANTHESIS: &str = "(";
const CLOSE_PARANTHESIS: &str = ")";
const OPEN_CURLY: &str = "{";
const CLOSE_CURLY: &str = "}";

mod nesting::*::deep {
    use crate::OPEN_PARANTHESIS;

    fn example() {
        println!("{}", OPEN_PARANTHESIS);
    }
}
```
