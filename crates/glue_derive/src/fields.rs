use proc_macro2::{Ident, Span};
use quote::quote;
use syn::{
    parse::Parse, parse_macro_input, punctuated::Punctuated, token::Comma, Field, ItemStruct,
    Visibility,
};

struct Input {
    vis: Visibility,
    name: Ident,
    fields: Punctuated<Field, Comma>,
}

impl Parse for Input {
    fn parse(input: syn::parse::ParseStream) -> syn::Result<Self> {
        let item = input.parse::<ItemStruct>()?;

        let named = match item.fields {
            syn::Fields::Named(fields) => fields.named,
            syn::Fields::Unnamed(_) => {
                return Err(input.error("Named struct expected, got Unnamed struct"))
            }
            syn::Fields::Unit => return Err(input.error("Named struct expected, got Unit struct")),
        };

        Ok(Self {
            vis: item.vis,
            name: item.ident,
            fields: named,
        })
    }
}

pub fn derive_input(item: proc_macro::TokenStream) -> proc_macro::TokenStream {
    let Input { vis, name, fields } = parse_macro_input!(item as Input);

    let name_string = name.to_string();

    let result_name = name_string.strip_suffix("Options").unwrap_or(&name_string);
    let result_name = Ident::new(&format!("{result_name}Result",), Span::call_site());

    let result_fields = fields
        .iter()
        .map(|field| {
            let vis = &field.vis;
            let name = field.ident.as_ref().unwrap(); // This is an enforced named struct
            let ty = &field.ty;

            quote!(#vis #name: Option<<#ty as InputField>::Type>)
        })
        .collect::<Vec<_>>();

    let result_checks = fields
        .iter()
        .map(|field| {
            let name = field.ident.as_ref().unwrap(); // This is an enforced named struct
            let name_str = name.to_string();
            quote! {
                if let Some(value) = value.#name.as_ref() {
                    self.#name.verify_input(value).map_err(|e| format!("{}: {}", #name_str, e))?;
                }
            }
        })
        .collect::<Vec<_>>();

    let expanded = quote! {
        #[derive(serde::Deserialize)]
        #vis struct #result_name {
            #(#result_fields),*
        }

        impl InputField for #name {
            type Type = #result_name;

            fn verify_input(&self, value: &Self::Type) -> Result<(), String> {
                #(#result_checks)*
                Ok(())
            }
        }
    };

    expanded.into()
}
