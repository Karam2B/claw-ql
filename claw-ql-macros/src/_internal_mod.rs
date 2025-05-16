#![allow(unused)]
use helper::IfContainsIdent;
use proc_macro2::Ident;
use proc_macro2::TokenStream;
use quote::quote;
use syn::Token;
use syn::punctuated::Punctuated;
use syn::visit::Visit;
use syn::visit_mut::visit_field_mut;
use syn::visit_mut::visit_fields_mut;
use syn::visit_mut::visit_generics_mut;
use syn::visit_mut::visit_where_clause_mut;
use syn::{
    Item, Meta, parse_quote,
    visit_mut::{VisitMut, visit_item_mut},
};

pub fn main(mut ts: Item) -> TokenStream {
    #[derive(Default)]
    struct RemoteGenericForIdentSafety{

    }

    impl VisitMut for RemoteGenericForIdentSafety {
        fn visit_fields_mut(&mut self, i: &mut syn::Fields) {
            // remote all fields that contains I
            let ident = match self.ident_of_ident_safety {
                Some(ref i) => i.clone(),
                None => return visit_fields_mut(self, i),
            };

            let list = i
                .iter()
                .cloned()
                .filter(|e| {
                    let mut e = e.clone();
                    let mut check = IfContainsIdent::new(ident.clone());
                    check.visit_field(&e);
                    if check.contains_ident {
                        return false;
                    }

                    visit_field_mut(self, &mut e);
                    return true;
                })
                .collect::<Punctuated<_, Token![,]>>();

        

            match i {
                syn::Fields::Unnamed(_) => {
                    *i = syn::Fields::Unnamed(syn::FieldsUnnamed {
                        paren_token: Default::default(),
                        unnamed: list,
                    })
                }
                syn::Fields::Named(_) => {
                    *i = syn::Fields::Named(syn::FieldsNamed {
                        brace_token: Default::default(),
                        named: list,
                    })
                }
                _ => {}
            }
        }
        fn visit_generics_mut(&mut self, i: &mut syn::Generics) {
            i.params = i
                .params
                .clone()
                .into_iter()
                .filter(|i| {
                    let i = match i {
                        syn::GenericParam::Type(i) => i,
                        _ => return true,
                    };
                    let mut ident_safety = false;
                    let ident_safety_quote: Meta = parse_quote!(claw_ql_macros::ident_safety);

                    let ident_safe = i.attrs.iter().any(|e| {
                        if e.meta == ident_safety_quote {
                            ident_safety = true;
                            return false;
                        } else {
                            return true;
                        }
                    });

                    if ident_safety {
                        self.ident_of_ident_safety = Some(i.ident.clone());
                        return false;
                    }
                    return true;
                })
                .collect::<Punctuated<_, Token![,]>>();
            visit_generics_mut(self, i);
        }
        fn visit_type_param_mut(&mut self, i: &mut syn::TypeParam) {
            let mut ident_safety = false;
            let ident_safety_quote: Meta = parse_quote!(claw_ql_macros::ident_safety);
            i.attrs = i
                .attrs
                .iter()
                .filter(|e| {
                    if e.meta == ident_safety_quote {
                        ident_safety = true;
                        return false;
                    } else {
                        return true;
                    }
                })
                .cloned()
                .collect();

            if ident_safety {
                self.ident_of_ident_safety = Some(i.ident.clone());
                i.bounds = Default::default()
            }
        }

        fn visit_where_clause_mut(&mut self, i: &mut syn::WhereClause) {
            if let Some(ident) = &self.ident_of_ident_safety {
                i.predicates = i
                    .predicates
                    .clone()
                    .into_iter()
                    .filter(|e| {
                        let mut check = IfContainsIdent::new(ident.clone());
                        check.visit_where_predicate(e);
                        if check.contains_ident {
                            return false;
                        }
                        return true;
                    })
                    .collect();
            }

            visit_where_clause_mut(self, i)
        }
    }
    visit_item_mut(&mut RemoteGenericForIdentSafety::default(), &mut ts);
    quote!(#ts)
}

pub mod helper {
    use proc_macro2::Ident;
    use syn::visit::Visit;

    pub struct IfContainsIdent {
        ident: Ident,
        pub contains_ident: bool,
    }
    impl IfContainsIdent {
        pub fn new(ident: Ident) -> Self {
            IfContainsIdent {
                ident,
                contains_ident: false,
            }
        }
    }
    impl<'ast> Visit<'ast> for IfContainsIdent {
        fn visit_ident(&mut self, i: &'ast proc_macro2::Ident) {
            if self.ident == *i {
                self.contains_ident = true
            }
        }
    }
}
