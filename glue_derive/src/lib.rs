mod args;

use quote::quote;
use syn::{
    parenthesized, parse::Parse, parse_macro_input, punctuated::Punctuated, Block, FnArg, Ident,
    Path, ReturnType, Token, TypePath,
};

use crate::args::{get_extern_params, get_extern_params_stream};

const SUPPORTED_TYPES: [&'static str; 10] = [
    "u8", "u16", "u32", "u64", "i8", "i16", "i32", "i64", "f32", "f64",
];

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
                .map(|arg| {
                    let pat = &arg.pat.pat;
                    let ty = &arg.pat.ty;

                    macro_rules! match_rtype {
                        ($($ty:expr => $ac:block),+, _ => $el:block) => {
                            match ty.as_ref() {
                                $(
                                    syn::Type::Path(TypePath {qself: _, path: Path { leading_colon: _, segments }}) if $ty.contains(&segments.last().unwrap().ident.to_string().as_str()) => $ac
                                ),*
                                syn::Type::Path(TypePath {qself: _, path: _}) => $el,
                                _ => panic!("'{}' is not supported in exposed function", quote!(#ty)),
                            }
                        };
                    }

                    match_rtype! {
                        SUPPORTED_TYPES => {
                            quote! {}
                        },
                        ["bool"] => {
                            quote! {
                                let #pat: #ty = #pat != 0;
                            }
                        },
                        ["String"] => {
                            quote! {
                                let #pat: #ty = unsafe { std::ffi::CString::from_raw(#pat).into_string().unwrap() };
                            }
                        },
                        _ => {
                            quote! {
                                let #pat: #ty = unsafe {
                                    let __c = std::ffi::CString::from_raw(#pat).into_string().unwrap();
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
