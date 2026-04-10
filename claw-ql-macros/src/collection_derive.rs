use convert_case::{Case, Casing};
use proc_macro_error::abort;
use proc_macro2::Ident;
use proc_macro2::TokenStream;
use quote::quote;
use syn::Visibility;
use syn::{DeriveInput, spanned::Spanned, visit::Visit};

pub fn main(input: DeriveInput) -> TokenStream {
    let mut ts = quote!();
    if let Visibility::Public(_) = input.vis.clone() {
    } else {
        abort!(input.vis.span(), "collections has to be public");
    }

    #[derive(Default)]
    struct MainDerive {
        mem_ty: Vec<syn::Type>,
        mem_name: Vec<Ident>,
    }

    impl Visit<'_> for MainDerive {
        fn visit_generics(&mut self, i: &syn::Generics) {
            if i.lt_token.is_some() {
                abort!(i.span(), "geneerics are not supported");
            }
        }

        fn visit_field(&mut self, field: &syn::Field) {
            match field.ident.as_ref() {
                Some(ident) => {
                    ();
                    self.mem_ty.push(field.ty.clone());
                    self.mem_name.push(ident.clone());
                }
                None => {
                    abort!(field.span(), "unamed fields are not supported");
                }
            }
        }
    }

    let this = &input.ident;
    let this_lowercase = Ident::new(
        input.ident.to_string().to_case(Case::Snake).as_str(),
        input.ident.span(),
    );
    let partial = Ident::new(&format!("{}Partial", this), proc_macro2::Span::call_site());

    let mut main_derive = MainDerive::default();
    main_derive.visit_derive_input(&input);
    let MainDerive { mem_ty, mem_name } = main_derive;

    // let mem_scope = mem_name
    //     .iter()
    //     .map(|m| format!("{}_{}", this_lowercase, m.to_string()))
    //     .collect::<Vec<_>>();

    let serde_derive = if cfg!(feature = "serde") {
        Some(quote!(,::claw_ql::prelude::macro_derive_collection::Deserialize))
    } else {
        None
    };

    ts.extend(quote!(
        #[derive(Default, Debug #serde_derive)]
        pub struct #partial {
            #(pub #mem_name: ::claw_ql::prelude::macro_derive_collection::update<#mem_ty>,)*
        }

        #[derive(Clone, Default)]
        #[allow(non_camel_case_types)]
        pub struct #this_lowercase;
    ));

    // type LinkedData = #table_name_camel_case;
    // fn members(&self) -> Vec<String>
    // {
    //     vec![
    //         #(String::from(stringify!(#member_name)),)*
    //     ]
    // }

    // impl<S> OnMigrate<S> for #table_name_lower_case_ident
    // where
    //     S: QueryBuilder<Output = <S as Database>::Arguments<'static>>,
    //     for<'q> S::Arguments<'q>: IntoArguments<'q, S>,
    //     S: QueryBuilder + DatabaseDefaultPrimaryKey,
    //     <S as DatabaseDefaultPrimaryKey>::KeyType: Type<S> + for<'c> Decode<'c, S> + for<'e> Encode<'e, S>,
    //     #(
    //         #member_ty: Type<S> + for<'c> Decode<'c, S> + for<'e> Encode<'e, S>,
    //     )*
    // {
    //     fn custom_migrate_statements(&self) -> Vec<String> {

    //         let mut stmt = CreateTableSt::init(header::create, self.table_name());
    //         stmt.column_def("id", primary_key::<S>());
    //         #(
    //         stmt.column_def(
    //             stringify!(#member_name),
    //             col_type_check_if_null::<#member_ty>(),
    //         );
    //         )*
    //         vec![Buildable::build(stmt).0]
    //     }
    //     fn custom_migration<'e>(
    //         &self,
    //         exec: impl for<'q> Executor<'q, Database = S> + Clone,
    //     ) -> impl Future<Output = ()>
    //     where
    //     {
    //         async move {
    //             let mut stmt = CreateTableSt::init(header::create, self.table_name());
    //             stmt.column_def("id", primary_key::<S>());
    //             #(
    //             stmt.column_def(
    //                 stringify!(#member_name),
    //                 col_type_check_if_null::<#member_ty>(),
    //             );
    //             )*
    //             stmt.execute(exec).await.unwrap();
    //         }
    //     }
    // }

    // impl<S> Queries<S> for #table_name_lower_case_ident
    //     where
    // S: QueryBuilder + DatabaseDefaultPrimaryKey,
    // for<'s> &'s str: sqlx_::ColumnIndex<<S as Database>::Row>,
    // <S as DatabaseDefaultPrimaryKey>::KeyType: Type<S> + for<'c> Decode<'c, S> + for<'e> Encode<'e, S>,
    // #(
    //     S: Accept<#member_ty>,
    //     #member_ty: Type<S> + for<'c> Decode<'c, S> + for<'e> Encode<'e, S>,
    // )*
    // {
    //     // fn on_insert(&self, this: Self::Data, stmt: &mut InsertOneSt<S>)
    //     // where
    //     //     S: sqlx::Database,
    //     // {
    //     //     #(stmt.col(
    //     //         stringify!(#member_name).to_string(),
    //     //         this.#member_name
    //     //     );)*
    //     // }

    //     fn on_select(&self, stmt: &mut SelectSt<S>)
    //     {
    //         #(
    //            stmt.select(col(stringify!(#member_name)).
    //             table(stringify!(#table_name_camel_case)).
    //             alias(#member_name_scoped)
    //            );
    //         )*
    //     }

    //     // fn on_update(
    //     //     &self,
    //     //     this: Self::Partial,
    //     //     stmt: &mut UpdateSt<S>,
    //     // ) where
    //     //     S: claw_ql::QueryBuilder,
    //     // {
    //     //             #(
    //     //     match this.#member_name {
    //     //         update::keep => {}
    //     //         update::set(set) => stmt.set_col(stringify!(#member_name).to_string(), set),
    //     //     };
    //     //         )*
    //     // }

    //     // fn from_row_noscope(&self, row: &<S as Database>::Row) -> Self::Data
    //     // {
    //     //     Self::Data { #(
    //     //         #member_name: row.get(stringify!(#member_name)),
    //     //     )*}
    //     // }

    //     // fn from_row_scoped(&self, row: &<S as Database>::Row) -> Self::Data
    //     // {
    //     //     Self::Data { #(
    //     //             #member_name: row.get(#member_name_scoped),
    //     //     )*}
    //     // }
    // }

    let members_mod = Ident::new(
        &format!("{}_members", this_lowercase.to_string()),
        this.span(),
    );

    ts.extend(quote!(
        #[allow(non_camel_case_types)]
        pub mod #members_mod {
        use ::claw_ql::prelude::macro_derive_collection::*;
        use super::#this_lowercase;

        #(
            #[derive(Clone, Default)]
            pub struct #mem_name;
            impl MemberBasic for #mem_name {
                fn name(&self) -> &str {
                    stringify!(#mem_name)
                }
            }
            impl Member for #mem_name {
                type Collection = #this_lowercase;
                type Data = #mem_ty;
            }
        )*
    }));

    ts.extend(quote!( const _: () = {
        use ::claw_ql::prelude::macro_derive_collection::*;
        use #members_mod::*;

        impl CollectionBasic for  #this_lowercase {
            fn table_name_lower_case(&self) -> &'static str {
                stringify!(#this_lowercase)
            }
            fn table_name(&self) -> &'static str {
                stringify!(#this)
            }
        }

        impl Collection for #this_lowercase {
            type Partial = #partial;
            type Data = #this;
            type Members = (#(#mem_name,)*);
            type Id = SingleIncremintalInt;

            fn members(&self) -> &Self::Members {
                &(#(#mem_name,)*)
            }
            fn id(&self) -> &Self::Id {
                &SingleIncremintalInt
            }
        }

        impl HasHandler for #this {
            type Handler = #this_lowercase;
        }

        impl HasHandler for #partial {
            type Handler = #this_lowercase;
        }
    };));

    if cfg!(feature = "inventory") {
        ts.extend(quote::quote!(
            const _: () = {
                use ::claw_ql::prelude::inventory::*;

                submit!(Migration { obj: || Box::new(#this_lowercase) });
                submit!(Collection { obj: || Box::new(#this_lowercase) });
            };
        ));
    }

    return ts;
}

#[test]
fn test_collection_derive() {}
