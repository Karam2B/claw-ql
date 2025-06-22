use proc_macro::TokenStream;
use proc_macro_error::proc_macro_error;

mod collection_derive;
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
