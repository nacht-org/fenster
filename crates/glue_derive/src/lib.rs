mod expose;
mod fields;
mod params;
mod utils;

#[proc_macro_attribute]
pub fn expose(
    attr: proc_macro::TokenStream,
    item: proc_macro::TokenStream,
) -> proc_macro::TokenStream {
    expose::expose(attr, item)
}

#[proc_macro_derive(InputField)]
pub fn derive_input(item: proc_macro::TokenStream) -> proc_macro::TokenStream {
    fields::derive_input(item)
}
