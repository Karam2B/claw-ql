use proc_macro::TokenStream;
use proc_macro_error::proc_macro_error;
use syn::{Item, parse};
mod relation;

// #[proc_macro]
// #[proc_macro_error]
// pub fn relation(input: TokenStream) -> TokenStream {
//     let input = match syn::parse::<relation::Input>(input) {
//         Ok(data) => data,
//         Err(err) => {
//             return err.to_compile_error().into();
//         }
//     };
//
//     relation::main(input).into()
// }
