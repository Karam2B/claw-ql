use proc_macro::TokenStream;
use proc_macro_error::proc_macro_error;
use quote::ToTokens;

mod collection_derive;
mod dynamic_fn_mod;
mod flat_struct;
mod from_row_alias_derive;
mod on_migrate_derive;
mod pdev;
mod relation;
mod sql_mod;
// #[cfg(test)]
// mod tests;

#[cfg(test)]
#[track_caller]
fn expect_to_eq(expect: proc_macro2::TokenStream, to_be: proc_macro2::TokenStream) {
    pretty_assertions::assert_eq!(expect.to_string(), to_be.to_string())
}

#[proc_macro_derive(OnMigrate)]
#[proc_macro_error]
pub fn on_migrate(input: TokenStream) -> TokenStream {
    on_migrate_derive::main(input.into()).into()
}

#[proc_macro_derive(Collection)]
#[proc_macro_error]
pub fn collection(input: TokenStream) -> TokenStream {
    collection_derive::main(input.into()).into()
}

#[proc_macro_derive(FromRowAlias)]
#[proc_macro_error]
pub fn from_row_alias(input: TokenStream) -> TokenStream {
    from_row_alias_derive::main(input.into()).into()
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

#[proc_macro_attribute]
#[proc_macro_error]
pub fn simple_enum(_attr: TokenStream, input: TokenStream) -> TokenStream {
    simple_enum_mod::main(input.into()).into()
}

#[doc(hidden)]
#[allow(unreachable_code)]
#[proc_macro_attribute]
pub fn panic_for_fun(_attr: TokenStream, input: TokenStream) -> TokenStream {
    proc_macro_error::entry_point(
        || {
            let fn_item = syn::parse::<syn::ItemFn>(input).unwrap();
            proc_macro_error::abort!(fn_item.sig.ident.span(), "hi");
            quote::quote!().into()
        },
        false,
    )
    .into()
}

#[proc_macro]
#[proc_macro_error]
pub fn sql(input: TokenStream) -> TokenStream {
    match syn::parse::<sql_mod::statement>(input) {
        Ok(ok) => ok.to_token_stream().into(),
        Err(err) => err.to_compile_error().into(),
    }
}

mod simple_enum_mod {
    #![allow(unused)]
    #![warn(unused_must_use)]
    #![allow(non_camel_case_types)]
    use convert_case::Casing;
    use proc_macro::Ident;
    use proc_macro_error::{abort, proc_macro_error};
    use proc_macro2::TokenStream;
    use quote::{quote, quote_spanned};
    use syn::{
        Field, FieldsNamed, ItemEnum, TypePath, parse_quote, parse_quote_spanned,
        punctuated::Punctuated, spanned::Spanned, visit::Visit, visit_mut::VisitMut,
    };

    pub fn main(input: TokenStream) -> TokenStream {
        let mut input = match syn::parse2::<ItemEnum>(input) {
            Ok(data) => data,
            Err(err) => {
                return err.to_compile_error().into();
            }
        };

        #[derive(Default)]
        struct variants {
            units: Vec<syn::Ident>,
            zeros: Vec<syn::Ident>,
            ones: Vec<(syn::Ident, syn::Type)>,
        };
        impl VisitMut for variants {
            fn visit_variant_mut(&mut self, i: &mut syn::Variant) {
                let ident = i.ident.clone();
                match &i.fields {
                    syn::Fields::Unit => {
                        let f: Field = parse_quote!(#ident);
                        i.fields = syn::Fields::Unnamed(syn::FieldsUnnamed {
                            paren_token: Default::default(),
                            unnamed: Punctuated::from_iter([f]),
                        });
                        self.units.push(ident);
                    }
                    syn::Fields::Named(fields_named) => {
                        abort!(
                            fields_named.span(),
                            "nammed fields cannot be supported for simple enums!"
                        );
                    }
                    syn::Fields::Unnamed(fields_unnamed) if fields_unnamed.unnamed.len() == 0 => {
                        self.zeros.push(ident);
                    }
                    syn::Fields::Unnamed(fields_unnamed) if fields_unnamed.unnamed.len() == 1 => {
                        let ty = fields_unnamed.unnamed.first().unwrap().ty.clone();
                        self.ones.push((ident, ty));
                    }
                    syn::Fields::Unnamed(fields_unnamed) => {
                        abort!(
                            fields_unnamed.span(),
                            "unnamed fields has to be either one or zero"
                        );
                    }
                }

                // _ => ,
            }
        }

        let mut v = variants::default();
        v.visit_item_enum_mut(&mut input);
        let units = v.units;
        if v.zeros.is_empty().not() {
            todo!("contains zeros")
        }
        if v.ones.is_empty().not() {
            todo!("contains ones")
        }
        let this = &input.ident;

        let mut ret = quote! {
            #input
        };
        let mut macros = Vec::<(syn::Ident, syn::Ident)>::new();

        {
            let macro_name = syn::Ident::new(
                &format!(
                    "{}_auto_match",
                    this.to_string().to_case(convert_case::Case::Snake)
                ),
                this.span(),
            );

            ret.extend(quote! {
                #[macro_export]
                macro_rules! #macro_name {
                    ($this:expr, $method:ident) => {
                        match $this {
                            #(#this::#units(v) => v.$method()),*
                        }
                    };
                }
            });

            macros.push((macro_name, syn::Ident::new("auto_match", this.span())));
        }

        {
            let macro_name = syn::Ident::new(
                &format!(
                    "{}_is_subset_of",
                    this.to_string().to_case(convert_case::Case::Snake)
                ),
                this.span(),
            );

            ret.extend(quote! {
                #[macro_export]
                macro_rules! #macro_name {
                    ($of:ident) => {
                        impl From<#this> for $of {
                            fn from(value: #this) -> Self {
                                match value {
                                    #(#this::#units(v) => Self::#units(v)),*
                                }
                            }
                        }
                    };
                }
            });

            macros.push((macro_name, syn::Ident::new("is_subset_of", this.span())));
        }

        {
            let macro_name = syn::Ident::new(
                &format!(
                    "{}_closure",
                    this.to_string().to_case(convert_case::Case::Snake)
                ),
                this.span(),
            );

            ret.extend(quote! {
                #[macro_export]
                macro_rules! #macro_name {
                    ($this:expr, |$new_ident:ident| $method:tt) => {
                        match $this {
                            #(#this::#units(v) => (|$new_ident: #units| $method)(v),)*
                        }
                    };
                }
            });

            macros.push((macro_name, syn::Ident::new("closure", this.span())));
        }

        {
            let mod_name = syn::Ident::new(
                &format!(
                    "{}_macros",
                    this.to_string().to_case(convert_case::Case::Snake)
                ),
                this.span(),
            );
            let (macro_, alias) = macros
                .into_iter()
                .unzip::<_, _, Vec<syn::Ident>, Vec<syn::Ident>>();

            ret.extend(quote!(
                pub mod #mod_name {
                    #(pub use #macro_ as #alias ;)*
                }
            ));
        }

        for each in units {
            ret.extend(quote! {
                impl From<#each> for #this {
                    fn from(value: #each) -> Self {
                        Self::#each(value)
                    }
                }
            });
        }

        ret
    }

    use proc_macro_error::entry_point;
    use std::{ops::Not, panic::AssertUnwindSafe};

    #[test]
    pub fn failing() {
        let expect = quote!(
            #[derive(Debug)]
            enum Bar {
                Hi(String),
            }
        );

        // todo: create test_entry_point
        let expect = proc_macro_error::entry_point(|| main(expect).into(), true);

        pretty_assertions::assert_eq!(
            quote!(::core::compile_error! {"expected `enum`"}).to_string(),
            expect.to_string()
        );
    }

    #[test]
    fn simple_enum_test() {
        let expect = quote!(
            #[derive(Debug)]
            enum Bar {
                String,
                Custom,
            }
        );

        let expect = main(expect);
        let to_be = quote! {
            #[derive(Debug)]
            enum Bar {
                String(String),
                Custom(Custom),
            }

            #[macro_export]
            macro_rules! bar_auto_match {
                ($this:expr, $method:ident) => {
                    match $this {
                        Bar::String(v) => v.$method(),
                        Bar::Custom(v) => v.$method()
                    }
                };
            }



            #[macro_export]
            macro_rules! bar_is_subset_of {
                ($of:ident) => {
                    impl From<Bar> for $of {
                        fn from(value: Bar) -> Self {
                            match value {
                                Bar::String(v) => Self::String(v),
                                Bar::Custom(v) => Self::Custom(v),
                            }
                        }
                    }
                };
            }

            #[macro_export]
            macro_rules! bar_closure {
                ($this:expr, |$new_ident:ident| $method:tt) => {
                    match $this {
                        Bar::String(v) => (|$new_ident: String| $method)(v),
                        Bar::Custom(v) => (|$new_ident: Custom| $method)(v),
                    }
                };
            }

            pub mod bar_macros {
                pub use bar_auto_match as auto_match;
                pub use bar_is_subset_of as is_subset_of;
                pub use bar_closure as closure;
            }

            impl From<String> for Bar {
                fn from(value: String) -> Self {
                    Self::String(value)
                }
            }

            impl From<Custom> for Bar {
                fn from(value: Custom) -> Self {
                    Self::Custom(value)
                }
            }
        };

        pretty_assertions::assert_eq!(expect.to_string(), to_be.to_string());
    }
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
pub fn dynamic_fn(_attr: TokenStream, input: TokenStream) -> TokenStream {
    dynamic_fn_mod::main(input.into()).into()
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
