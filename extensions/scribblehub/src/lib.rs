use fenster_core::ext::{Meta, ReadingDirection};
use fenster_glue_derive::expose;
use lazy_static::lazy_static;

lazy_static! {
    static ref META: Meta<'static> = Meta {
        id: "com.scribblehub",
        name: "ScribbleHUb",
        lang: "en",
        version: [0, 1, 0],
        base_urls: vec!["https://www.scribblehub.com"],
        rds: vec![ReadingDirection::Ltr],
        attrs: vec![],
    };
}

#[expose]
pub fn meta() -> &Meta {
    &META
}
