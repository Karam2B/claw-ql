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

    let table_name_camel_case = &input.ident;
    let partial_ident = Ident::new(&format!("{}Partial", table_name_camel_case), proc_macro2::Span::call_site());

    let mut main_derive = MainDerive { 
        fields: vec![],
        table_lower_case: table_name_camel_case.to_string().to_lowercase(),
    };
    main_derive.visit_derive_input(&input);

    let member_ty = main_derive.fields.iter().map(|m| m.ty.clone()).collect::<Vec<_>>();
    let member_name = main_derive.fields.iter().map(|m| m.name.clone()).collect::<Vec<_>>();
    let member_name_scoped =
        main_derive.fields.iter().map(|m| m.name_scoped.clone()).collect::<Vec<_>>();

    ts.extend(quote!(
        #[cfg_attr(feature = "serde", derive(::claw_ql::prelude::macro_derive_collection::Deserialize))]
        #[derive(Default, Debug)]
        pub struct #partial_ident {
            #(pub #member_name: ::claw_ql::prelude::macro_derive_collection::update<#member_ty>,)*
        }
    ));

    ts.extend(quote!( const _: () = {
        use ::claw_ql::prelude::macro_derive_collection::*;

        impl<S> Collection<S> for PhantomData<#table_name_camel_case> 
            where 
        S: QueryBuilder + DatabaseDefaultPrimaryKey,
        for<'s> &'s str: ColumnIndex<<S as Database>::Row>,
        <S as DatabaseDefaultPrimaryKey>::KeyType: Type<S> + for<'c> Decode<'c, S> + for<'e> Encode<'e, S>,
        #(
            #member_ty: Type<S> + for<'c> Decode<'c, S> + for<'e> Encode<'e, S>,
        )*
        {
            type PartailCollection = #partial_ident;
            type Yeild = #table_name_camel_case;

            fn on_migrate(&self, stmt: &mut CreateTableSt<S>) {
                stmt.column("id", primary_key::<S>());
                #(
                stmt.column(
                    stringify!(#member_name),
                    col_type_check_if_null::<#member_ty>(),
                );
                )*
            }

            fn on_select(&self, stmt: &mut SelectSt<S>)
            {
                #(
                   stmt.select(col(stringify!(#member_name)).
                    table(stringify!(#table_name_camel_case)).
                    alias(#member_name_scoped)
                   );
                )*
            }

            fn members(&self) -> &'static [&'static str] {
                 &[
                     #(
                         stringify!(#member_name),
                     )*
                 ]
            }
        
            fn members_scoped(&self) -> &'static [&'static str] {
                 &[
                     #(
                         #member_name_scoped,
                     )*
                 ]
            }
        
            fn table_name(&self) -> &'static str {
                stringify!(#table_name_camel_case)
            }
        

        
            fn from_row_noscope(&self, row: &<S as Database>::Row) -> Self::Yeild
            {
                Self { #(
                    #member_name: row.get(stringify!(#member_name)),
                )*}
            }
        
            fn from_row_scoped(&self, row: &<S as Database>::Row) -> Self::Yeild
            {
                Self { #(
                        #member_name: row.get(#member_name_scoped),
                )*}
            }
        
        }
    };));

    return ts
}

#[test]
fn test_collection_derive() {
}
