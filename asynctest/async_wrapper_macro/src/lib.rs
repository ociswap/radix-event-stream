use proc_macro::TokenStream;
use quote::quote;
use syn::{parse_macro_input, ItemFn};

#[proc_macro_attribute]
pub fn async_wrapper(_attr: TokenStream, item: TokenStream) -> TokenStream {
    let input_fn = parse_macro_input!(item as ItemFn);

    // Implement the logic to transform the input function here.
    // This example assumes the transformation is straightforward and doesn't handle generics or other complex features.

    let fn_name = &input_fn.sig.ident;
    let fn_body = &input_fn.block;
    let output = quote! {
        fn #fn_name<S>(src: &mut i32, state: S) -> std::pin::Pin<Box<dyn std::future::Future<Output = ()> + Send + '_>> {
            Box::pin(async move {
                #fn_body
            })
        }
    };

    TokenStream::from(output)
}
