use proc_macro::TokenStream;
use proc_macro_error::proc_macro_error;

mod collection_derive;
mod flat_struct;
mod pdev;
mod relation;
#[cfg(test)]
mod tests;

#[proc_macro_derive(Collection)]
#[proc_macro_error]
pub fn collection(input: TokenStream) -> TokenStream {
    let derive = match syn::parse::<syn::DeriveInput>(input) {
        Ok(data) => data,
        Err(err) => {
            return err.to_compile_error().into();
        }
    };

    collection_derive::main(derive).into()
}

#[proc_macro]
#[proc_macro_error]
pub fn relation(input: TokenStream) -> TokenStream {
    let input = match syn::parse::<relation::Input>(input) {
        Ok(data) => data,
        Err(err) => {
            return err.to_compile_error().into();
        }
    };

    relation::main(input).into()
}

///  #[macro_export]
///  macro_rules! id_trait {
///      (feature_name) => {
///          "unstable_id_trait"
///      };
///      (stability) => {
///          unstable(reason = "expiremental")
///      };
///      // feature_name is always string literal
///      // stability is either `stable` or `unstable` note: this will never be quoted by `"`
///      // since is always string literal representing `since` (if stable) or `reason` (if unstable)
///      //
///      // you can cut this short by having this match arm as `transformer ($_ty:ty) $rest:ty`
///      (transformer ($feature_name:expr, $stability:expr, $since:expr) $ty:ty) => {
///          $ty
///      };
///  }
///
///
/// mod hi_2 {
///     #![allow(non_camel_case_types)]
///     // should that be optional since if not specified we will look for catch all unstable?
///     struct FeatureName(String);
///     struct TrackingIssue(Option<String>);
///     enum Stability {
///         Unstable { reason: Option<String> },
///         Stable { since: String },
///     }
///     pub trait macro_ {}
///     struct Parser(Option<Box<dyn macro_>>);
///     impl macro_ for () {
///         // 1. change vis from pub to pub(crate) unless opt-in [what?]
///         // 2. append "Availability" section to the item's doc
///     }
/// }
///  pub(crate) use id_trait;
#[proc_macro_attribute]
#[proc_macro_error]
pub fn pdev(attr: TokenStream, input: TokenStream) -> TokenStream {
    let mod_name = match syn::parse::<syn::Ident>(attr) {
        Ok(data) => data,
        Err(err) => {
            return err.to_compile_error().into();
        }
    };

    pdev::main(input, mod_name).into()
}

#[proc_macro_attribute]
#[proc_macro_error]
pub fn flat_struct(_attr: TokenStream, input: TokenStream) -> TokenStream {
    let result = match syn::parse::<flat_struct::ItemNestedStruct>(input) {
        Ok(data) => data,
        Err(err) => {
            return err.to_compile_error().into();
        }
    };

    result.into()
}

// // TODO
// builder_trait! {
//     components = ["link", "collection"];
//     level = mut;
//     rename mod_name = handler;
//     type Setting = ();
//     type ImplDefaultSetting = ();
//
// }
//
// struct Builder;
//
// builder_impl! {
//     impl handler for Builder {
//         type Context = String;
//         fn add_collection(&mut next, &mut context)
//            where Next: Clone
//         {
//             let _ = next.clone();
//         }
//     }
// }
