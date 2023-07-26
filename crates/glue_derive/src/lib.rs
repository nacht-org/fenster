mod params;

use proc_macro2::{TokenStream, TokenTree};
use quote::quote;
use syn::{
    parenthesized,
    parse::{Parse, ParseStream},
    parse_macro_input,
    punctuated::Punctuated,
    token::Comma,
    Block, FnArg, Ident, ReturnType, Token,
};

use crate::params::{get_extern_params, get_extern_params_stream};

struct Expose {
    name: Ident,
    params: Option<Punctuated<FnArg, Token![,]>>,
    block: Block,
    rtype: ReturnType,
}

fn skip_until_pub(input: ParseStream) -> syn::Result<()> {
    input.step(|cursor| {
        let mut rest = *cursor;
        while let Some((tt, next)) = rest.token_tree() {
            match &tt {
                TokenTree::Ident(ident) if ident.to_string() == "pub" => {
                    return Ok(((), rest));
                }
                _ => rest = next,
            }
        }
        Err(cursor.error("no `pub` was found after this point"))
    })
}

impl Parse for Expose {
    fn parse(input: ParseStream) -> syn::Result<Self> {
        skip_until_pub(input)?;

        input.parse::<Token![pub]>()?;
        input.parse::<Token![fn]>()?;
        let name: Ident = input.parse()?;

        let content;
        parenthesized!(content in input);
        let params = if content.is_empty() {
            None
        } else {
            Some(content.parse_terminated(FnArg::parse, Token![,])?)
        };

        let rtype = input.parse()?;
        let block = input.parse()?;

        Ok(Expose {
            name,
            params,
            block,
            rtype,
        })
    }
}

#[proc_macro_attribute]
pub fn expose(
    attr: proc_macro::TokenStream,
    item: proc_macro::TokenStream,
) -> proc_macro::TokenStream {
    let Expose {
        name,
        params,
        block,
        rtype,
    } = parse_macro_input!(item as Expose);

    let extern_params = params.as_ref().map(get_extern_params);
    let extern_params_stream = extern_params.as_ref().map(get_extern_params_stream);

    let extern_parse = extern_params
        .as_ref()
        .map(|params| {
            params
                .iter()
                .map(|pattype| {
                    let ident = &pattype.pat;
                    let ty = &pattype.ty;

                    quote! {
                        let #ident: #ty = <#ty as FromWasmAbi>::from_wasm_abi(#ident);
                    }
                })
                .collect::<Vec<_>>()
        })
        .map(|streams| quote!( #(#streams)* ))
        .unwrap_or(quote!());

    let extern_return = {
        match &rtype {
            ReturnType::Default => quote!(),
            ReturnType::Type(_, ty) => quote!( -> <#ty as ToWasmAbi>::Type ),
        }
    };

    let extern_block = {
        match &rtype {
            ReturnType::Default => {
                let stmts = &block.stmts;
                quote!( #(#stmts)* )
            }
            ReturnType::Type(_, ty) => quote!( #[inline] fn __inner_fn(#params) -> #ty #block ),
        }
    };

    let extern_rserial = {
        match &rtype {
            ReturnType::Default => quote!(),
            ReturnType::Type(_, _) => {
                let args = extern_params.as_ref().map(|params| {
                    params
                        .iter()
                        .map(|arg| &arg.pat)
                        .collect::<Punctuated<_, Comma>>()
                });

                quote!( ToWasmAbi::to_wasm_abi(__inner_fn(#args)) )
            }
        }
    };

    let attr = TokenStream::from(attr);
    let expanded = quote! {
        #attr
        #[no_mangle]
        pub extern "C" fn #name(#extern_params_stream) #extern_return {
            use quelle_glue::abi::{ToWasmAbi, FromWasmAbi};
            #extern_parse
            #extern_block
            #extern_rserial
        }
    };

    // println!("{expanded}");
    expanded.into()
}
