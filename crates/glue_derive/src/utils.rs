use proc_macro2::TokenTree;
use syn::parse::ParseStream;

pub fn skip_until_ident(input: ParseStream, name: &str) -> syn::Result<()> {
    input.step(|cursor| {
        let mut rest = *cursor;
        while let Some((tt, next)) = rest.token_tree() {
            match &tt {
                TokenTree::Ident(ident) if ident.to_string() == name => {
                    return Ok(((), rest));
                }
                _ => rest = next,
            }
        }
        Err(cursor.error("no `pub` was found after this point"))
    })
}
