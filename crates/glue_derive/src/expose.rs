use proc_macro2::TokenStream;
use quote::quote;
use syn::{
    braced, parenthesized,
    parse::{Parse, ParseStream},
    parse_macro_input,
    punctuated::Punctuated,
    token::Comma,
    Block, FnArg, Ident, ReturnType, Token, Type,
};

use crate::{
    params::{get_extern_params, get_extern_params_stream},
    utils,
};

pub fn expose(
    attr: proc_macro::TokenStream,
    item: proc_macro::TokenStream,
) -> proc_macro::TokenStream {
    let item_clone = item.clone();
    let parsed = parse_macro_input!(item_clone as Expose);

    let expanded = match parsed {
        Expose::Trait(value) => {
            let attr = TokenStream::from(attr);
            let item = TokenStream::from(item);
            let value = TokenStream::from(value);
            quote!(
                #attr
                #item
                #value
            )
        }
        Expose::Fn(value) => {
            let attr = TokenStream::from(attr);
            let value = TokenStream::from(value);
            quote! {
                #attr
                #value
            }
        }
    };

    // println!("{:#}", expanded);
    expanded.into()
}

enum Expose {
    Trait(ExposeTrait),
    Fn(ExposeFn),
}

impl Parse for Expose {
    fn parse(input: ParseStream) -> syn::Result<Self> {
        utils::skip_until_ident_contains(input, &["pub", "impl"])?;

        if input.peek(Token![impl]) {
            input.parse::<Token![impl]>()?;
            let trait_ty = input.parse::<Type>()?;
            input.parse::<Token![for]>()?;
            let ty = input.parse()?;

            let content;
            braced!(content in input);

            let mut funcs = vec![];
            while content.peek(Token![fn]) {
                let expose_fn = ExposeFn::parse_inline(&content)?;
                funcs.push(expose_fn);
            }

            Ok(Expose::Trait(ExposeTrait {
                trait_ty,
                ty,
                funcs,
            }))
        } else if input.peek(Token![pub]) {
            input.parse::<Token![pub]>()?;
            ExposeFn::parse_inline(&input).map(Expose::Fn)
        } else {
            Err(input.error("'pub' (for functions) or 'impl' (for traits) expected"))
        }
    }
}

struct ExposeTrait {
    trait_ty: Type,
    ty: Type,
    funcs: Vec<ExposeFn>,
}

impl From<ExposeTrait> for TokenStream {
    fn from(value: ExposeTrait) -> Self {
        let ExposeTrait {
            trait_ty,
            ty,
            funcs,
        } = value;
        let mut extern_funcs = vec![];

        for func in funcs {
            let ExposeFn {
                name,
                params,
                block: _,
                rtype,
            } = func;

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

                        quote!( ToWasmAbi::to_wasm_abi(<#ty as #trait_ty>::#name(#args)) )
                    }
                }
            };

            let expanded = quote! {
                #[no_mangle]
                pub extern "C" fn #name(#extern_params_stream) #extern_return {
                    use quelle_glue::abi::{ToWasmAbi, FromWasmAbi};
                    #extern_parse
                    #extern_rserial
                }
            };

            extern_funcs.push(expanded);
        }

        quote! {
            #(#extern_funcs)*
        }
    }
}

struct ExposeFn {
    name: Ident,
    params: Option<Punctuated<FnArg, Token![,]>>,
    block: Block,
    rtype: ReturnType,
}

impl ExposeFn {
    fn parse_inline(input: ParseStream) -> syn::Result<Self> {
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

        Ok(ExposeFn {
            name,
            params,
            block,
            rtype,
        })
    }
}

impl From<ExposeFn> for TokenStream {
    fn from(value: ExposeFn) -> Self {
        let ExposeFn {
            name,
            params,
            block,
            rtype,
        } = value;

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

        let expanded = quote! {
            #[no_mangle]
            pub extern "C" fn #name(#extern_params_stream) #extern_return {
                use quelle_glue::abi::{ToWasmAbi, FromWasmAbi};
                #extern_parse
                #extern_block
                #extern_rserial
            }
        };

        expanded
    }
}
