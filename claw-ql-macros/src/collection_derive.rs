use proc_macro2::Ident;
use proc_macro_error::abort;
use proc_macro2::TokenStream;
use quote::quote;
use syn::{DeriveInput, spanned::Spanned, visit::Visit};

pub fn main(input: DeriveInput) -> TokenStream {
    let mut ts = quote!();

    struct Memeber<'ast> {
        ty: &'ast syn::Type,
        name: &'ast Ident,
        name_scoped: String,
    }

    struct MainDerive<'ast> {
        fields: Vec<Memeber<'ast>>,
        table_lower_case: String,
    }

    impl<'ast> Visit<'ast> for MainDerive<'ast> {
        fn visit_generics(&mut self, i: &'ast syn::Generics) {
            if i.lt_token.is_some() {
                abort!(i.span(), "geneerics are not supported");
            }
        }
        fn visit_field(&mut self, field: &'ast syn::Field) {
            match field.ident.as_ref() {
                Some(ident) => self.fields.push(Memeber {
                    ty: &field.ty,
                    name: ident,
                    name_scoped: format!("{}_{}", self.table_lower_case, ident),
                }),
                None => {
                    abort!(field.span(), "unamed fields are not supported");
                }
            }
        }
    }

    let d_ident = &input.ident;
    let partial_ident = Ident::new(&format!("{}Partial", d_ident), proc_macro2::Span::call_site());

    let mut main_derive = MainDerive { 
        fields: vec![],
        table_lower_case: d_ident.to_string().to_lowercase(),
    };
    main_derive.visit_derive_input(&input);

    let m_ty = main_derive.fields.iter().map(|m| m.ty.clone()).collect::<Vec<_>>();
    let m_name = main_derive.fields.iter().map(|m| m.name.clone()).collect::<Vec<_>>();
    let m_name_scoped =
        main_derive.fields.iter().map(|m| m.name_scoped.clone()).collect::<Vec<_>>();

    ts.extend(quote!(
        #[cfg_attr(feature = "serde", derive(::claw_ql::prelude::macro_derive_collection::Deserialize))]
        pub struct #partial_ident {
            #(pub #m_name: ::claw_ql::prelude::macro_derive_collection::update<#m_ty>,)*
        }
    ));

    ts.extend(quote!( const _: () = {
        use ::claw_ql::prelude::macro_derive_collection::*;
        use ::clau_ql::statements::create_table_st::CreateTableSt;

        impl<S> Collection<S> for #d_ident 
            where 
        S: QueryBuilder + DatabaseDefaultPrimaryKey,
        for<'s> &'s str: ColumnIndex<<S as Database>::Row>,
        #(
            #m_ty: Type<S> + for<'c> Decode<'c, S> + for<'e> Encode<'e, S>,
        )*
        {
            type PartailCollection = #partial_ident;

            fn on_migrate(stmt: &mut CreatTableSt<S>) {
                stmt.column("id", primary_key::<S>());
                #(
                stmt.column(
                    stringify!(#m_name),
                    col_type_check_if_null::<#m_ty>(),
                );
                )*
            }

            fn on_select(stmt: &mut SelectSt<S>)
            {
                #(
                   stmt.select(col(stringify!(#m_name_scoped)));
                )*
            }

            fn members() -> &'static [&'static str] {
                 &[
                     #(
                         stringify!(#m_name),
                     )*
                 ]
            }
        
            fn members_scoped() -> &'static [&'static str] {
                 &[
                     #(
                         #m_name_scoped,
                     )*
                 ]
            }
        
            fn table_name() -> &'static str {
                stringify!(#d_ident)
            }
        

        
            fn from_row_noscope(row: &<S as Database>::Row) -> Self
            {
                Self { #(
                    #m_name: row.get(stringify!(#m_name)),
                )*}
            }
        
            fn from_row_scoped(row: &<S as Database>::Row) -> Self
            {
                Self { #(
                        #m_name: row.get(#m_name_scoped),
                )*}
            }
        
        }
    };));

    return ts
}

#[test]
fn test_collection_derive() {
}
