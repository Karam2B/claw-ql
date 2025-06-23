use convert_case::{Case, Casing};
use proc_macro_error::abort;
use proc_macro2::Ident;
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
    let table_name_lower_case_ident = Ident::new(
        input.ident.to_string().to_case(Case::Snake).as_str(),
        input.ident.span(),
    );
    let partial_ident = Ident::new(
        &format!("{}Partial", table_name_camel_case),
        proc_macro2::Span::call_site(),
    );

    let mut main_derive = MainDerive {
        fields: vec![],
        table_lower_case: table_name_camel_case.to_string().to_case(Case::Snake),
    };
    main_derive.visit_derive_input(&input);

    let member_ty = main_derive
        .fields
        .iter()
        .map(|m| m.ty.clone())
        .collect::<Vec<_>>();
    let member_name = main_derive
        .fields
        .iter()
        .map(|m| m.name.clone())
        .collect::<Vec<_>>();
    let member_name_scoped = main_derive
        .fields
        .iter()
        .map(|m| m.name_scoped.clone())
        .collect::<Vec<_>>();

    ts.extend(quote!(
        #[cfg_attr(feature = "serde", derive(::claw_ql::prelude::macro_derive_collection::Deserialize))]
        #[derive(Default, Debug)]
        pub struct #partial_ident {
            #(pub #member_name: ::claw_ql::prelude::macro_derive_collection::update<#member_ty>,)*
        }
        #[derive(Clone, Default)]
        #[allow(non_camel_case_types)]
        pub struct #table_name_lower_case_ident;
    ));

    ts.extend(quote!( const _: () = {
        use ::claw_ql::prelude::macro_derive_collection::*;

        impl CollectionBasic for  #table_name_lower_case_ident {
            type LinkedData = #table_name_camel_case;
            fn members(&self) -> Vec<String> 
            {
                vec![
                    #(String::from(stringify!(#member_name)),)*
                ]
            }
            fn table_name_lower_case(&self) -> &'static str {
                stringify!(#table_name_lower_case_ident)
            }
            fn table_name(&self) -> &'static str {
                stringify!(#table_name_camel_case)
            }
        }

impl<S> OnMigrate<S> for #table_name_lower_case_ident
where
    S: QueryBuilder<Output = <S as Database>::Arguments<'static>>,
    for<'q> S::Arguments<'q>: IntoArguments<'q, S>,
    S: QueryBuilder + DatabaseDefaultPrimaryKey,
    <S as DatabaseDefaultPrimaryKey>::KeyType: Type<S> + for<'c> Decode<'c, S> + for<'e> Encode<'e, S>,
    #(
        #member_ty: Type<S> + for<'c> Decode<'c, S> + for<'e> Encode<'e, S>,
    )*
{
    fn custom_migration<'e>(
        &self,
        exec: impl for<'q> Executor<'q, Database = S> + Clone,
    ) -> impl Future<Output = ()>
    where
    {
        async move {
            let mut stmt = CreateTableSt::init(header::create, self.table_name());
            stmt.column_def("id", primary_key::<S>());
            #(
            stmt.column_def(
                stringify!(#member_name),
                col_type_check_if_null::<#member_ty>(),
            );
            )*
            stmt.execute(exec).await.unwrap();
        }
    }
}

impl HasHandler for #table_name_camel_case {
    type Handler = #table_name_lower_case_ident;
}

impl HasHandler for #partial_ident {
    type Handler = #table_name_lower_case_ident;
}

impl<S> Collection<S> for #table_name_lower_case_ident
    where 
S: QueryBuilder + DatabaseDefaultPrimaryKey,
for<'s> &'s str: sqlx_::ColumnIndex<<S as Database>::Row>,
<S as DatabaseDefaultPrimaryKey>::KeyType: Type<S> + for<'c> Decode<'c, S> + for<'e> Encode<'e, S>,
#(
    S: Accept<#member_ty>,
    #member_ty: Type<S> + for<'c> Decode<'c, S> + for<'e> Encode<'e, S>,
)*
{
    type Partial = #partial_ident;
    type Data = #table_name_camel_case;

    fn on_insert(&self, this: Self::Data, stmt: &mut InsertOneSt<S>)
    where
        S: sqlx::Database,
    {
        #(stmt.col(
            stringify!(#member_name).to_string(), 
            this.#member_name
        );)*
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

    fn on_update(
        &self,
        this: Self::Partial,
        stmt: &mut UpdateSt<S>,
    ) where
        S: claw_ql::QueryBuilder,
    {
                #(
        match this.#member_name {
            update::keep => {}
            update::set(set) => stmt.set_col(stringify!(#member_name).to_string(), set),
        };
            )*
    }



    fn from_row_noscope(&self, row: &<S as Database>::Row) -> Self::Data
    {
        Self::Data { #(
            #member_name: row.get(stringify!(#member_name)),
        )*}
    }

    fn from_row_scoped(&self, row: &<S as Database>::Row) -> Self::Data
    {
        Self::Data { #(
                #member_name: row.get(#member_name_scoped),
        )*}
    }

}
    };));

    return ts;
}

#[test]
fn test_collection_derive() {}
