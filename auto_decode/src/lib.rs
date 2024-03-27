extern crate proc_macro;

use proc_macro::TokenStream;
use quote::quote;
use syn::{parse_macro_input, FnArg, ItemFn, PatType};

#[proc_macro_attribute]
pub fn auto_decode(_attr: TokenStream, item: TokenStream) -> TokenStream {
    let mut function = parse_macro_input!(item as ItemFn);

    // Check if the first argument is of type EventHandlerInput<T>
    let is_first_arg_event_handler_input =
        if let Some(FnArg::Typed(PatType { ty, .. })) =
            function.sig.inputs.iter().nth(0)
        {
            // Here, we need to check if `ty` is EventHandlerInput<T>. This is a bit tricky because `ty` is a `Type`,
            // and we need to match it against a specific path (i.e., EventHandlerInput<T>).
            // One approach is to convert `ty` to a string and check if it contains "EventHandlerInput".
            // Note: This is a simplistic approach and might not work in all cases, especially with complex type paths or generics.
            let ty_string = quote!(#ty).to_string();
            ty_string.starts_with("EventHandlerContext")
        } else {
            false
        };

    if !is_first_arg_event_handler_input {
        panic!(
            "Expected the first argument to be of type EventHandlerContext<T>"
        );
    }

    // Extract and modify the second argument as before
    let second_arg_type = if let Some(FnArg::Typed(PatType { ty, .. })) =
        function.sig.inputs.iter().nth(1)
    {
        ty.as_ref().clone()
    } else {
        panic!("Expected the second argument to be a typed argument");
    };

    if let Some(second_arg) = function.sig.inputs.iter_mut().nth(1) {
        *second_arg = syn::parse_quote!(event: Vec<u8>);
    }

    let original_body = function.block;
    function.block = syn::parse_quote!({
        fn assert_decodable<T: radix_event_stream::ScryptoDecode>() {}
        assert_decodable::<#second_arg_type>();

        let event = radix_event_stream::scrypto_decode::<#second_arg_type>(&event).unwrap();
        #original_body
    });

    TokenStream::from(quote!(#function))
}
