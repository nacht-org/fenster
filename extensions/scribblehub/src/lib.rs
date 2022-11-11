#[macro_use]
extern crate fenster_glue;

use fenster_core::prelude::*;
use fenster_glue::{http::SendRequest, prelude::*};
use kuchiki::traits::TendrilSink;
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
pub fn meta() -> &'static Meta<'static> {
    set_panic_hook();
    &META
}

#[expose]
pub fn fetch_novel(url: String) -> Result<Novel, FensterError> {
    let response = Request::get(url.clone()).send()?;
    println!("{}", response.status);

    let doc = kuchiki::parse_html().one(response.body.unwrap());
    println!("parsed doc");

    let novel = Novel {
        title: doc
            .select_first("h1[property=\"name\"]")
            .map(|node| node.text_contents().trim().to_string())
            .unwrap_or_default(),
        authors: vec![doc
            .select_first(r#"span[property="name"]"#)
            .map(|node| node.text_contents().trim().to_string())
            .unwrap_or_default()],
        thumb: doc
            .select_first(".page-content-inner .thumbnail")
            .map(|node| node.attributes.borrow().get("src").map(|s| s.to_string()))
            .ok()
            .flatten(),
        desc: doc
            .select(r#".description > [property="description"] > p"#)
            .map(|nodes| nodes.map(|node| node.text_contents()).collect::<Vec<_>>())
            .unwrap_or(vec![]),
        lang: vec![META.lang.to_string()],
        metadata: doc
            .select(r#"a.label[href*="tag"]"#)
            .map(|nodes| {
                nodes
                    .map(|node| Metadata::new(String::from("subject"), node.text_contents(), None))
                    .collect::<Vec<_>>()
            })
            .unwrap_or_default(),
        url,
    };

    Ok(novel)
}
