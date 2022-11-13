#[macro_use]
extern crate fenster_glue;

use chrono::{DateTime, NaiveDateTime, Utc};
use fenster_core::prelude::*;
use fenster_glue::{http::SendRequest, prelude::*};
use kuchiki::{
    iter::{Descendants, Elements, Select},
    traits::TendrilSink,
};
use lazy_static::lazy_static;

lazy_static! {
    static ref META: Meta = Meta {
        id: String::from("com.scribblehub"),
        name: String::from("ScribbleHub"),
        lang: String::from("en"),
        version: [0, 1, 0],
        base_urls: vec![String::from("https://www.scribblehub.com")],
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
    println!("{}", response.status);

    let doc = kuchiki::parse_html().one(response.body.unwrap());
    println!("parsed doc");

    let volume = Volume {
        chapters: doc
            .select("tbody > tr")
            .map(parse_chapter_list)
            .unwrap_or_default(),
        ..Default::default()
    };

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

fn parse_chapter_list(nodes: Select<Elements<Descendants>>) -> Vec<Chapter> {
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

        let chapter = Chapter {
            index: chapters.len() as i32,
            title: link.text_contents().trim().to_string(),
            url: link
                .attributes
                .borrow()
                .get("href")
                .map(|s| s.to_string())
                .unwrap_or_default(),
            updated_at: updated_at,
        };

        chapters.push(chapter);
    }

    chapters
}
