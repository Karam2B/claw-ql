/// to be removed in favor of macros
///
/// User-facing check, validation, and transormation.
/// Needed to execute Operation without bugs.
///
/// panics or compile error for `Operation` trait is considered a bug.
/// you might forgot to call `safety_check` before calling `exec_operation` or
/// there is a bug in this crate's code
///
/// most of the time these checks can be known inside const context,
/// since `const_trait_impl` is not stable yet, you have to run these at runtime.
/// for now you can use the macro `sql` which mimics this trait using const_blocks.
///
/// example of that is when you try to  check wither a 'where clause'
/// specifies any unique filters, if you have tuple of (T0,T1), there is no
/// way to check if EITHER T0 or T1 is a unique filter, that would be
/// equivalent of this hypothetical rust
///
/// code:
/// ```no_run
///     impl<T0,T1> SafeOperation
///     for SelectOneAndOnlyOne<Wheres = (T0,T1)>
///     where   
///         (T0: AssertUniqueFilter) or (T1: AssertUniqueFilter),
///     {}
/// ```
///
/// or just simply using `const_trait_impl`
///
/// ```no_run
///     impl<T0, T1> const SafeOperation for SelectOneAndOnlyOne<Wheres = (T0,T1)>
///     where
///         T0: [const] UniqueFilter + [const] Destruct,
///         T1: [const] UniqueFilter + [const] Destruct,
///     {
///         fn safety_check(self) -> Result<Self::Ok, Self::Error> {
///             if self.wheres.0.is_unique() || self.wheres.1.is_unique() {
///                 return Ok(self.0);
///             }
///             Err(Self::Error::NonUniqueOperation)
///         }
///     }
/// ```
///
/// if the the checks are "inevitably non-const", consider if the implementation of `Operation`
/// can have an output of `Option<T>` or `Result<T, _>`, if so, no need to
/// implement `SafeOperation`
///
/// in this crate `NeedCheck` is used to force you to use `SafeOperation`
/// before using `Operation` impls by making `Ok = NeedCheck<T>`, and
/// implementing Operation for `NeedCheck<T>`
pub trait SafeOperation {
    type Error;
    type Ok;
    fn safety_check(self) -> Result<Self::Ok, Self::Error>;
}
