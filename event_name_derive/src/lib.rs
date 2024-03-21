use proc_macro::TokenStream;
use quote::quote;
use syn::{parse_macro_input, DeriveInput};

#[proc_macro_derive(EventName, attributes(event_name_override))]
pub fn event_name_derive(input: TokenStream) -> TokenStream {
    let input = parse_macro_input!(input as DeriveInput);

    let name = &input.ident;
    let gen = quote! {
        impl EventName for #name {
            fn event_name() -> &'static str {
                stringify!(#name)
            }
        }
    };

    gen.into()
}
