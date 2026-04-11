use proc_macro::TokenStream;
use proc_macro_error::proc_macro_error;

#[path = "lib_local.rs"]
mod local_lib;

mod on_migrate_derive;
#[proc_macro_derive(OnMigrate)]
#[proc_macro_error]
pub fn on_migrate(input: TokenStream) -> TokenStream {
    on_migrate_derive::main(input.into()).into()
}

mod collection_derive;
#[proc_macro_derive(Collection)]
#[proc_macro_error]
pub fn collection(input: TokenStream) -> TokenStream {
    collection_derive::main(input.into()).into()
}

mod from_row_alias_derive;
#[proc_macro_derive(FromRowAlias)]
#[proc_macro_error]
pub fn from_row_alias(input: TokenStream) -> TokenStream {
    from_row_alias_derive::main(input.into()).into()
}

mod simple_enum_mod;
#[proc_macro_attribute]
#[proc_macro_error]
pub fn simple_enum(_: TokenStream, input: TokenStream) -> TokenStream {
    simple_enum_mod::main(input.into()).into()
}

mod sql_mod;
#[proc_macro]
#[proc_macro_error]
pub fn sql(input: TokenStream) -> TokenStream {
    match syn::parse::<sql_mod::MainStatement>(input) {
        Ok(ok) => sql_mod::main_statment_to_token(ok).into(),
        Err(err) => err.to_compile_error().into(),
    }
}
