extern crate proc_macro;

use proc_macro::TokenStream;
use quote::{format_ident, quote};
use syn::{
    parse_macro_input, punctuated::Punctuated, token::Comma, FnArg,
    GenericParam, ItemFn, PathArguments, Type,
};
#[proc_macro_attribute]
pub fn auto_decode(_attr: TokenStream, item: TokenStream) -> TokenStream {
    let input_fn = parse_macro_input!(item as ItemFn);

    // Extract function name to use for the struct and impl
    let fn_name = &input_fn.sig.ident;
    let struct_name = fn_name;

    // Assuming the function signature follows a specific pattern, we need to extract
    // the generic types from the first argument (context). This example assumes that
    // EventHandlerContext is always the first argument and that it carries two generic types.
    let context_arg = match input_fn
        .sig
        .inputs
        .first()
        .expect("Expected at least one argument")
    {
        FnArg::Typed(pat_type) => &*pat_type.ty,
        _ => panic!("Expected a typed argument"),
    };

    // Extract the EventHandlerContext's generic types
    let generics = if let Type::Path(type_path) = context_arg {
        // This assumes the path's last segment is `EventHandlerContext` and has arguments.
        let args = &type_path
            .path
            .segments
            .last()
            .expect("Expected a type segment")
            .arguments;
        if let syn::PathArguments::AngleBracketed(args) = args {
            args.args.iter().collect::<Vec<_>>()
        } else {
            panic!("Expected generic arguments in EventHandlerContext");
        }
    } else {
        panic!("Expected EventHandlerContext to be a type path");
    };

    // The event type is expected to be the second argument of the function.
    let event_type = match input_fn
        .sig
        .inputs
        .iter()
        .nth(1)
        .expect("Expected at least two arguments")
    {
        FnArg::Typed(pat_type) => &*pat_type.ty,
        _ => panic!("Expected the second argument to be typed"),
    };

    let function_body = &input_fn.block;

    // Generate the struct and impl using the extracted information.
    let expanded = quote! {
        #[derive(Clone)]
        #[allow(non_camel_case_types)]
        pub struct #struct_name;

        #[::async_trait::async_trait]
        impl EventHandler<#(#generics),*> for #struct_name {
            async fn handle(
                &self,
                context: EventHandlerContext<'_, #(#generics),*>,
                event: Vec<u8>,
            ) -> Result<(), EventHandlerError> {
                let event: #event_type = scrypto_decode(&event).unwrap();
                #function_body
            }
        }
    };

    TokenStream::from(expanded)
}

#[proc_macro_attribute]
pub fn auto_transaction_handler(
    _attr: TokenStream,
    item: TokenStream,
) -> TokenStream {
    let input_fn = parse_macro_input!(item as ItemFn);

    // Extract function name to use for the struct and impl
    let fn_name = &input_fn.sig.ident;
    let struct_name = fn_name;

    // Assuming the function signature follows a specific pattern, we need to extract
    // the generic types from the context argument. This example assumes that
    // TransactionHandlerContext is the first argument and that it carries two generic types.
    let context_arg = match input_fn
        .sig
        .inputs
        .first()
        .expect("Expected at least one argument")
    {
        FnArg::Typed(pat_type) => &*pat_type.ty,
        _ => panic!("Expected a typed argument"),
    };

    // Extract the TransactionHandlerContext's generic types
    let generics = if let Type::Path(type_path) = context_arg {
        let args = &type_path
            .path
            .segments
            .last()
            .expect("Expected a type segment")
            .arguments;
        if let syn::PathArguments::AngleBracketed(args) = args {
            args.args.iter().collect::<Vec<_>>()
        } else {
            panic!("Expected generic arguments in TransactionHandlerContext");
        }
    } else {
        panic!("Expected TransactionHandlerContext to be a type path");
    };

    let function_body = &input_fn.block;

    // Generate the struct and impl using the extracted information.
    let expanded = quote! {
        #[derive(Clone)]
        #[allow(non_camel_case_types)]
        struct #struct_name;

        #[::async_trait::async_trait]
        impl TransactionHandler<#(#generics),*> for #struct_name {
            async fn handle(
                &self,
                context: TransactionHandlerContext<'_, #(#generics),*>,
            ) -> Result<(), TransactionHandlerError> {
                #function_body
            }
        }
    };

    TokenStream::from(expanded)
}
