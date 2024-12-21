extern crate proc_macro;
extern crate quote;
extern crate syn;

use proc_macro::TokenStream;
use quote::quote;
use syn::{parse_macro_input, ItemFn};

#[proc_macro_attribute]
pub fn handler(_attr: TokenStream, item: TokenStream) -> TokenStream {
    let input_fn = parse_macro_input!(item as ItemFn);
    let fn_name = &input_fn.sig.ident;
    let fn_name_str = fn_name.to_string();
    let expanded = quote! {
        #input_fn

        inventory::submit! {
            ::helix_gateway::router::router::HandlerSubmission(
                ::helix_gateway::router::router::Handler::new(
                    #fn_name_str,
                    #fn_name  // No Arc::new needed
                )
            )
        }
    };

    expanded.into()
}
