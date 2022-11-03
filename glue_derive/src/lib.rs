use proc_macro2::TokenStream;
use quote::{__private::Span, quote};
use syn::{
    parenthesized, parse::Parse, parse_macro_input, punctuated::Punctuated, token::Comma, Block,
    FnArg, Ident, PatType, Path, ReturnType, Token, TypePath,
};

struct Expose {
    name: Ident,
    params: Option<Punctuated<FnArg, Token![,]>>,
    block: Block,
    rtype: ReturnType,
}

impl Parse for Expose {
    fn parse(input: syn::parse::ParseStream) -> syn::Result<Self> {
        input.parse::<Token![pub]>()?;
        input.parse::<Token![fn]>()?;
        let name: Ident = input.parse()?;

        let content;
        parenthesized!(content in input);
        let params = if content.is_empty() {
            None
        } else {
            Some(content.parse_terminated(FnArg::parse)?)
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
    _attr: proc_macro::TokenStream,
    item: proc_macro::TokenStream,
) -> proc_macro::TokenStream {
    let Expose {
        name,
        params,
        block,
        rtype,
    } = parse_macro_input!(item as Expose);

    let extern_params = params.map(|params| get_extern_params(&params));
    let extern_params_stream = extern_params.as_ref().map(get_extern_params_stream);

    let extern_parse = extern_params
        .as_ref()
        .map(|params| {
            params
                .iter()
                .map(|param| {
                    let return_ident = &param.pat.pat;
                    let return_ty = &param.pat.ty;
                    let ptr_ident = &param.ident;

                    macro_rules! match_rtype {
                        ($($ty:ident => $ac:block),+, _ => $el:block) => {
                            match return_ty.as_ref() {
                                $(
                                    syn::Type::Path(TypePath {qself: _, path: Path { leading_colon: _, segments }}) if segments.last().unwrap().ident == "$ty" => $ac
                                ),*
                                syn::Type::Path(TypePath {qself: _, path: _}) => $el,
                                _ => panic!("'{}' is not supported in exposed function", quote!(#return_ty)),
                            }
                        };
                    }

                    match_rtype! {
                        String => {
                            quote! {
                                let #return_ident: #return_ty = unsafe { std::ffi::CString::from_raw(#ptr_ident).into_string().unwrap() };
                            }
                        },
                        _ => {
                            quote! {
                                let #return_ident: #return_ty = unsafe {
                                    let __c = std::ffi::CString::from_raw(#ptr_ident).into_string().unwrap();
                                    serde_json::from_str(&__c).unwrap()
                                };
                            }
                        }
                    }
                })
                .collect::<Vec<_>>()
        })
        .map(|streams| {
            quote! {
                #(#streams)*
            }
        })
        .unwrap_or(quote!());

    let extern_return = {
        match &rtype {
            ReturnType::Default => quote!(),
            ReturnType::Type(_, _) => quote!( -> *mut std::os::raw::c_char ),
        }
    };

    let extern_block = {
        match &rtype {
            ReturnType::Default => {
                let stmts = &block.stmts;
                quote!( #(#stmts)* )
            }
            ReturnType::Type(_, ty) => quote!( let __original_return: #ty = #block; ),
        }
    };

    let extern_rserial = {
        match &rtype {
            ReturnType::Default => quote!(),
            ReturnType::Type(_, _) => quote! {
                let __new_return = serde_json::to_string(&__original_return).unwrap();
                let __new_return = std::ffi::CString::new(__new_return).unwrap();
                __new_return.into_raw()
            },
        }
    };

    let expanded = quote! {
        #[no_mangle]
        pub extern "C" fn #name(#extern_params_stream) #extern_return {
            #extern_parse
            #extern_block
            #extern_rserial
        }
    };

    println!("{expanded}");
    expanded.into()
}

/// Minimal struct to hold extern function arg
struct ExternArg {
    ident: Ident,
    ty: TokenStream,
    pat: PatType,
}

fn get_extern_params(params: &Punctuated<FnArg, Comma>) -> Vec<ExternArg> {
    params
        .iter()
        .map(|arg| match arg {
            FnArg::Receiver(_) => panic!("'self' is not supported in exposed functions"),
            FnArg::Typed(p) => p,
        })
        .enumerate()
        .map(|(i, p)| ExternArg {
            ident: Ident::new(&format!("ptr{i}"), Span::call_site()),
            ty: quote!(*mut std::os::raw::c_char),
            pat: p.clone(),
        })
        .collect::<Vec<_>>()
}

fn get_extern_params_stream(params: &Vec<ExternArg>) -> Punctuated<TokenStream, Comma> {
    params
        .iter()
        .map(|arg| {
            let ExternArg { ident, ty, pat: _ } = arg;
            quote!(#ident: #ty)
        })
        .collect()
}
