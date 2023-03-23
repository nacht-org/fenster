#[allow(unused_imports)]
#[macro_use]
extern crate quelle_glue;

use kuchiki::{traits::TendrilSink, NodeRef};
use once_cell::sync::Lazy;
use quelle_core::prelude::*;
use quelle_glue::prelude::*;

define_meta! {
    let META = {
        id: "en.creativenovels",
        name: "CreativeNovels",
        langs: ["en"],
        base_urls: ["https://creativenovels.com"],
        rds: [Ltr],
        attrs: [],
    };
}

#[cfg(debug_assertions)]
#[expose]
pub fn setup() {
    set_panic_hook();
}

#[expose]
pub fn fetch_novel(url: String) -> Result<Novel, QuelleError> {
    let response = Request::get(url.clone()).send()?;
    let doc = kuchiki::parse_html().one(response.text().unwrap());

    println!("doc created");

    let author = doc
        .select_first(".x-bar-container > [class*='14']")
        .get_text()?;

    println!("got author");

    let cover_element = doc.select_first("img.book_cover").ok();
    let cover = cover_element
        .map(|node| match node.get_attribute("src") {
            Some(value) => Some(value),
            None => node.get_attribute("data-cfsrc"),
        })
        .flatten();

    println!("got cover");

    let novel = Novel {
        title: doc
            .select_first(".x-bar-container > [class*='12']")
            .get_text()?,
        authors: vec![author],
        cover,
        description: doc.select(".novel_page_synopsis > p").collect_text(),
        volumes: collect_volumes(&doc)?,
        metadata: collect_metadata(&doc)?,
        langs: META.langs.clone(),
        status: doc
            .select_first(".novel_status")
            .get_text()?
            .as_str()
            .into(),
        url,
    };

    Ok(novel)
}

fn collect_volumes(doc: &NodeRef) -> Result<Vec<Volume>, QuelleError> {
    let mut volume = Volume::default();

    let shortlink = doc
        .select_first("link[rel='shortlink']")
        .get_attribute("href")
        .ok_or_else(|| QuelleError::ParseFailed(ParseError::ElementNotFound))?;

    println!("got shortlink");

    Ok(vec![volume])
}

fn collect_metadata(doc: &NodeRef) -> Result<Vec<Metadata>, QuelleError> {
    let mut metadata = vec![];

    let genres = doc.select(".genre_novel > a");
    if let Ok(elements) = genres {
        for element in elements {
            metadata.push(Metadata::new(
                String::from("subject"),
                element.get_text(),
                None,
            ))
        }
    }

    let tags = doc.select(".suggest_tag > a");
    if let Ok(elements) = tags {
        for element in elements {
            metadata.push(Metadata::new(String::from("tag"), element.get_text(), None))
        }
    }

    Ok(metadata)
}

#[expose]
pub fn fetch_chapter_content(url: String) -> Result<String, QuelleError> {
    Ok(String::new())
}
