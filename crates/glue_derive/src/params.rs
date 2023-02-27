use proc_macro2::TokenStream;
use quote::quote;
use syn::{punctuated::Punctuated, token::Comma, FnArg, PatType};

pub(crate) fn get_extern_params(params: &Punctuated<FnArg, Comma>) -> Vec<PatType> {
    params
        .iter()
        .map(|arg| match arg {
            FnArg::Receiver(_) => panic!("'self' is not supported in exposed functions"),
            FnArg::Typed(p) => p,
        })
        .cloned()
        .collect::<Vec<_>>()
}

pub(crate) fn get_extern_params_stream(params: &Vec<PatType>) -> Punctuated<TokenStream, Comma> {
    params
        .iter()
        .map(|param| {
            let ident = &param.pat;
            let ty = &param.ty;
            quote!(#ident: <#ty as quelle_glue::abi::FromWasmAbi>::Type)
        })
        .collect()
}
