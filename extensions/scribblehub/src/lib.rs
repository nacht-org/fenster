use fenster_core::prelude::*;
use fenster_glue::prelude::*;
use lazy_static::lazy_static;

lazy_static! {
    static ref META: Meta<'static> = Meta {
        id: "com.scribblehub",
        name: "ScribbleHub",
        lang: "en",
        version: [0, 1, 0],
        base_urls: vec!["https://www.scribblehub.com"],
        rds: vec![ReadingDirection::Ltr],
        attrs: vec![],
    };
}

#[expose]
pub fn meta() -> &Meta {
    set_panic_hook();
    &META
}

#[expose]
pub fn fetch_novel(url: String) -> Result<Novel, FensterError> {
    Ok(Novel {
        title: String::from("Unknown"),
        url,
    })
}
