#[macro_use]
extern crate fenster_glue;

use chrono::{DateTime, NaiveDateTime, Utc};
use fenster_core::prelude::*;
use fenster_glue::prelude::*;
use kuchiki::{
    iter::{Descendants, Elements, Select},
    traits::TendrilSink,
};
use lazy_static::lazy_static;

lazy_static! {
    static ref META: Meta = Meta {
        id: String::from("en.royalroad"),
        name: String::from("RoyalRoad"),
        lang: String::from("en"),
        version: String::from(env!("CARGO_PKG_VERSION")),
        base_urls: vec![String::from("https://www.royalroad.com")],
        rds: vec![ReadingDirection::Ltr],
        attrs: vec![],
    };
}

#[expose]
pub fn meta() -> &'static Meta {
    set_panic_hook();
    &META
}

#[expose]
pub fn fetch_novel(url: String) -> Result<Novel, FensterError> {
    let response = Request::get(url.clone()).send()?;
    let doc = kuchiki::parse_html().one(response.body.unwrap());

    let volume = Volume {
        chapters: doc
            .select("tbody > tr")
            .map(parse_chapter_list)
            .map_err(|_| ParseError::ElementNotFound)?
            .unwrap_or_default(),
        ..Default::default()
    };

    let author = doc.select_first("span[property='name']").get_text()?;

    let novel = Novel {
        title: doc.select_first("h1[property='name']").get_text()?,
        authors: vec![author],
        thumb: doc
            .select_first(".page-content-inner .thumbnail")
            .get_attribute("src"),
        desc: doc.select_text(r#".description > [property="description"] > p"#),
        status: doc
            .select_first(".widget_fic_similar > li:last-child > span:last-child")
            .map(|node| node.text_contents().as_str().into())
            .unwrap_or_default(),
        lang: META.lang.to_string(),
        volumes: vec![volume],
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

#[expose]
pub fn fetch_chapter_content(url: String) -> Result<Option<String>, FensterError> {
    let response = Request::get(url).send()?;
    let doc = kuchiki::parse_html().one(response.body.unwrap());

    let content = doc
        .select_first(".chapter-content")
        .map(|node| -> Result<String, ParseError> {
            let mut out = Vec::new();
            node.as_node()
                .serialize(&mut out)
                .map_err(|_| ParseError::SerializeFailed)?;
            Ok(String::from_utf8_lossy(&out).to_string())
        })
        .ok()
        .transpose()?;

    Ok(content)
}

fn parse_chapter_list(nodes: Select<Elements<Descendants>>) -> Result<Vec<Chapter>, FensterError> {
    let mut chapters = vec![];

    for tr in nodes {
        let link = tr.as_node().select_first("a[href]").ok();
        let Some(link) = link else { continue };

        let updated_at = tr
            .as_node()
            .select_first("time")
            .map(|node| {
                node.attributes
                    .borrow()
                    .get("unixtime")
                    .map(|s| s.parse::<i64>().ok())
            })
            .ok()
            .flatten()
            .flatten()
            .map(|timestamp| NaiveDateTime::from_timestamp(timestamp, 0))
            .map(|naive| DateTime::from_utc(naive, Utc))
            .map(|dt| dt.into());

        let url = link
            .attributes
            .borrow()
            .get("href")
            .map(|s| s.to_string())
            .unwrap_or_default();

        let chapter = Chapter {
            index: chapters.len() as i32,
            title: link.text_contents().trim().to_string(),
            url: META.derive_abs_url(url, None)?,
            updated_at,
        };

        chapters.push(chapter);
    }

    Ok(chapters)
}
