#[allow(unused_imports)]
#[macro_use]
extern crate quelle_glue;

use chrono::NaiveDateTime;
use kuchiki::{
    iter::{Descendants, Elements, Select},
    traits::TendrilSink,
};
use once_cell::sync::Lazy;
use quelle_core::prelude::*;
use quelle_glue::prelude::*;

define_meta! {
    let META = {
        id: "en.royalroad",
        name: "RoyalRoad",
        langs: ["en"],
        base_urls: ["https://www.royalroad.com"],
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
        cover: doc
            .select_first(".page-content-inner .thumbnail")
            .get_attribute("src"),
        description: doc
            .select(r#".description > [property="description"] > p"#)
            .collect_text(),
        status: doc
            .select_first(".widget_fic_similar > li:last-child > span:last-child")
            .map(|node| node.text_contents().as_str().into())
            .unwrap_or_default(),
        langs: META.langs.clone(),
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
pub fn fetch_chapter_content(url: String) -> Result<Option<String>, QuelleError> {
    let response = Request::get(url).send()?;
    let doc = kuchiki::parse_html().one(response.text().unwrap());

    let content = doc
        .select_first(".chapter-content")
        .map(|node| node.as_node().outer_html())
        .ok()
        .transpose()?;

    Ok(content)
}

fn parse_chapter_list(nodes: Select<Elements<Descendants>>) -> Result<Vec<Chapter>, QuelleError> {
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
            .map(|timestamp| NaiveDateTime::from_timestamp_opt(timestamp, 0))
            .flatten()
            .map(TaggedDateTime::Local);

        let url = link
            .attributes
            .borrow()
            .get("href")
            .map(|s| s.to_string())
            .unwrap_or_default();

        let chapter = Chapter {
            index: chapters.len() as i32,
            title: link.text_contents().clean_text(),
            url: META.convert_into_absolute_url(url, None)?,
            updated_at,
        };

        chapters.push(chapter);
    }

    Ok(chapters)
}

#[expose]
pub fn query_search(query: String, page: i32) -> Result<Vec<BasicNovel>, QuelleError> {
    let url = format!("https://www.royalroad.com/fictions/search?title={query}&page={page}");
    let response = Request::get(url.clone()).send()?;
    let doc = kuchiki::parse_html().one(response.text().unwrap());

    let mut novels = vec![];
    if let Ok(elements) = doc.select(".fiction-list-item") {
        for div in elements {
            let Some(a) = div.as_node().select_first(".fiction-title a").ok() else { continue };
            let Some(link) = a.get_attribute("href") else { continue };

            let cover = div
                .as_node()
                .select_first("img")
                .get_attribute("src")
                .map(|src| META.convert_into_absolute_url(src, Some(&url)))
                .transpose()?;

            let novel = BasicNovel {
                title: a.get_text(),
                url: META.convert_into_absolute_url(link, Some(&url))?,
                cover,
            };

            novels.push(novel);
        }
    }

    Ok(novels)
}
