use proc_macro_error::abort;
use proc_macro2::TokenStream;
use quote::quote;
use syn::{
    Ident,
    parse::{Parse, ParseStream},
};

pub struct Input {
    ident: syn::Ident,
    rest: TokenStream,
}

impl Parse for Input {
    fn parse(input: ParseStream) -> syn::Result<Self> {
        let ident = input.parse::<syn::Ident>()?;
        let rest = input.parse::<TokenStream>()?;

        Ok(Self { ident, rest })
    }
}

pub struct TwoIdent {
    from: Ident,
    to: Ident,
}

impl Parse for TwoIdent {
    fn parse(input: ParseStream) -> syn::Result<Self> {
        Ok(Self {
            from: input.parse()?,
            to: input.parse()?,
        })
    }
}

pub fn many_to_many(rest: TwoIdent) -> TokenStream {
    let to = rest.to;
    let from = rest.from;
    quote! {
        // const _: () = {
        //     use ::cms_for_rust::macro_prelude::relation_macro::*;
        //     impl Linked<#to> for #from {
        //         type Spec = ManyToMany;
        //         fn spec() -> Self::Spec {
        //             ManyToMany {
        //                 conjuction_table: format!(
        //                     "{}{}",
        //                     <#from as Collection<Sqlite>>::table_name(),
        //                     <#to as Collection<Sqlite>>::table_name(),
        //                 ),
        //                 // this should be the inverse
        //                 base_id: format!(
        //                     "{}_id",
        //                     <#from as Collection<Sqlite>>::table_name().to_lowercase()
        //                 ),
        //                 destination_id: format!(
        //                     "{}_id",
        //                     <#to as Collection<Sqlite>>::table_name().to_lowercase()
        //                 ),
        //             }
        //         }
        //     }
        //     impl Linked<#from> for #to {
        //         type Spec = ManyToMany;
        //         fn spec() -> Self::Spec {
        //             ManyToMany {
        //                 conjuction_table: format!(
        //                     "{}{}",
        //                     <#from as Collection<Sqlite>>::table_name(),
        //                     <#to as Collection<Sqlite>>::table_name(),
        //                 ),
        //                 // this should be the inverse
        //                 base_id: format!(
        //                     "{}_id",
        //                     <#to as Collection<Sqlite>>::table_name().to_lowercase()
        //                 ),
        //                 destination_id: format!(
        //                     "{}_id",
        //                     <#from as Collection<Sqlite>>::table_name().to_lowercase()
        //                 ),
        //             }
        //         }
        //     }
        //     submit! {
        //         SubmitDynRelation {
        //             obj: || {
        //                 Arc::new(
        //                     ManyToManyDynamic::<#to, #from>::new()
        //                 )
        //             }
        //         }
        //     }
        //     submit! {
        //         SubmitDynRelation {
        //             obj: || {
        //                 Arc::new(
        //                     ManyToManyDynamic::<#from, #to>::new()
        //                 )
        //             }
        //         }
        //     }
        // };
    }
}

pub fn optional_to_many(rest: TwoIdent) -> TokenStream {
    let to = rest.to;
    let from = rest.from;
    let foriegn_key = format!("{}_id", to.to_string().to_lowercase());
    quote! {
        const _: () = {
            use ::claw_ql::prelude::macro_relation::*;
            impl LinkData<#to> for Relation<#to, #from> {
                type Spec = OptionalToManyInverse<#to, #from>;
                fn spec(self) -> Self::Spec {
                    OptionalToManyInverse {
                        foriegn_key: #foriegn_key.to_string(),
                        _pd: PhantomData
                    }
                }
            }
            impl LinkData<#from> for Relation<#from, #to> {
                type Spec = OptionalToMany<#from, #to>;
                fn spec(self) -> Self::Spec {
                    OptionalToMany {
                        foriegn_key: #foriegn_key.to_string(),
                        _pd: PhantomData
                    }
                }
            }
            // submit! {
            //     SubmitDynRelation {
            //         obj: || {
            //             Arc::new(
            //                 OptionalToManyDynamic::<#from, #to>::new()
            //             )
            //         }
            //     }
            // }
            // // todo!()
            // submit! {
            //     SubmitDynRelation {
            //         obj: || {
            //             Arc::new(
            //                 OptionalToManyInverseDynamic::<#from, #to>::new()
            //             )
            //         }
            //     }
            // }
        };
    }
}

pub fn main(input: Input) -> TokenStream {
    match input.ident.to_string().as_str() {
        "optional_to_many" => optional_to_many(match syn::parse2::<TwoIdent>(input.rest) {
            Ok(ok) => ok,
            Err(err) => {
                return err.to_compile_error();
            }
        }),
        "many_to_many" => many_to_many(match syn::parse2::<TwoIdent>(input.rest) {
            Ok(ok) => ok,
            Err(err) => {
                return err.to_compile_error();
            }
        }),
        _ => abort!(
            input.ident.span(),
            "unknown relation, only {} are supported, consider implementing Related manually",
            ["optional_to_many", "many_to_many"].join(", ")
        ),
    }
}
