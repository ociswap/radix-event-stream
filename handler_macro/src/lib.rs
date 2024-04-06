extern crate proc_macro;

use proc_macro::TokenStream;
use quote::quote;
use syn::{parse_macro_input, FnArg, ItemFn, PathArguments, Type};

#[proc_macro_attribute]
pub fn event_handler(_attr: TokenStream, item: TokenStream) -> TokenStream {
    let input_fn = parse_macro_input!(item as ItemFn);

    let fn_name = &input_fn.sig.ident;
    let struct_name = fn_name;

    let context_arg = match input_fn
        .sig
        .inputs
        .first()
        .expect("Expected at least one argument")
    {
        FnArg::Typed(pat_type) => &*pat_type.ty,
        _ => panic!("Expected a typed argument"),
    };

    let generics = if let Type::Path(type_path) = context_arg {
        let args = &type_path
            .path
            .segments
            .last()
            .expect("Expected a type segment")
            .arguments;
        if let PathArguments::AngleBracketed(args) = args {
            args.args.iter().collect::<Vec<_>>()
        } else {
            panic!("Expected generic arguments in EventHandlerContext");
        }
    } else {
        panic!("Expected EventHandlerContext to be a type path");
    };

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

    // Adjust here to include unit type `()` as default for the second generic type if only one generic type is provided.
    let generics_handling = if generics.len() == 1 {
        quote! { #(#generics),*, () }
    } else {
        quote! { #(#generics),* }
    };

    // Generate the struct and impl using the extracted and adjusted information.
    let expanded = quote! {
        #[derive(Clone)]
        #[allow(non_camel_case_types)]
        pub struct #struct_name;

        #[radix_event_stream::async_trait]
        impl radix_event_stream::event_handler::EventHandler<#generics_handling> for #struct_name {
            async fn handle(
                &self,
                context: radix_event_stream::event_handler::EventHandlerContext<'_, #generics_handling>,
                event: Vec<u8>,
            ) -> Result<(), radix_event_stream::error::EventHandlerError> {
                let event: #event_type = radix_event_stream::scrypto_decode(&event).unwrap();
                #function_body
            }
        }
    };

    TokenStream::from(expanded)
}

#[proc_macro_attribute]
pub fn transaction_handler(
    _attr: TokenStream,
    item: TokenStream,
) -> TokenStream {
    let input_fn = parse_macro_input!(item as ItemFn);

    // Extract function name to use for the struct and impl
    let fn_name = &input_fn.sig.ident;
    let struct_name = fn_name;

    // Extract the generic types from the context argument. This example assumes that
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

        #[radix_event_stream::async_trait]
        impl radix_event_stream::transaction_handler::TransactionHandler<#(#generics),*> for #struct_name {
            async fn handle(
                &self,
                context: radix_event_stream::transaction_handler::TransactionHandlerContext<'_, #(#generics),*>,
            ) -> Result<(), radix_event_stream::error::TransactionHandlerError> {
                #function_body
            }
        }
    };

    TokenStream::from(expanded)
}
