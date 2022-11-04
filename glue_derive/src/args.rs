use proc_macro2::TokenStream;
use quote::quote;
use syn::{punctuated::Punctuated, token::Comma, FnArg, PatType, Path, TypePath};

use crate::SUPPORTED_TYPES;

/// Minimal struct to hold extern function arg
pub struct ExternArg {
    pub ty: TokenStream,
    pub pat: PatType,
}

pub(crate) fn get_extern_params(params: &Punctuated<FnArg, Comma>) -> Vec<ExternArg> {
    params
        .iter()
        .map(|arg| match arg {
            FnArg::Receiver(_) => panic!("'self' is not supported in exposed functions"),
            FnArg::Typed(p) => p,
        })
        .map(|pat| ExternArg {
            ty: quote!(*mut std::os::raw::c_char),
            pat: pat.clone(),
        })
        .collect::<Vec<_>>()
}

pub(crate) fn get_extern_params_stream(params: &Vec<ExternArg>) -> Punctuated<TokenStream, Comma> {
    params
        .iter()
        .map(|arg| {
            let ptr_ty = &arg.ty;

            let ident = &arg.pat.pat;
            let orig_ty = &arg.pat.ty;

            match orig_ty.as_ref() {
                syn::Type::Path(TypePath {
                    qself: _,
                    path:
                        Path {
                            leading_colon: _,
                            segments,
                        },
                }) if SUPPORTED_TYPES
                    .contains(&segments.last().unwrap().ident.to_string().as_str()) =>
                {
                    quote!(#ident: #orig_ty)
                }
                syn::Type::Path(TypePath {
                    qself: _,
                    path:
                        Path {
                            leading_colon: _,
                            segments,
                        },
                }) if segments.last().unwrap().ident == "bool" => {
                    quote!(#ident: usize)
                }
                syn::Type::Path(TypePath { qself: _, path: _ }) => quote!(#ident: #ptr_ty),
                _ => panic!(
                    "'{}' is not supported in exposed function",
                    quote!(#orig_ty)
                ),
            }
        })
        .collect()
}
