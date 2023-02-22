#[macro_use]
extern crate quelle_glue;

use std::collections::HashMap;

use chrono::NaiveDateTime;
use kuchiki::{traits::TendrilSink, NodeRef};
use once_cell::sync::Lazy;
use quelle_core::prelude::*;
use quelle_glue::prelude::*;

define_meta! {
    let META = {
        id: "en.novelpub",
        name: "NovelPub",
        langs: ["en"],
        base_urls: ["https://www.novelpub.com"],
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

    let mut status = NovelStatus::default();
    if let Some(nodes) = doc.select(".header-stats span").ok() {
        for node in nodes {
            let label = node.as_node().select_first("small");
            if let Ok(label) = label {
                let text = label.text_contents().to_ascii_lowercase();
                if text == "status" {
                    status = node
                        .as_node()
                        .select_first("strong")
                        .map(|node| node.text_contents().as_str().into())
                        .unwrap_or_default();
                }
            }
        }
    }

    let novel = Novel {
        title: doc.select_first(".novel-title").get_text()?,
        authors: doc.select(".author a").collect_text(),
        description: doc.select(".summary .content p").collect_text(),
        cover: doc.select_first(".cover img").get_attribute("data-src"),
        status,
        volumes: collect_toc(&url)?,
        metadata: collect_metadata(&doc),
        langs: META.langs.clone(),
        url,
    };

    Ok(novel)
}

fn collect_metadata(doc: &NodeRef) -> Vec<Metadata> {
    let mut metadata = vec![];

    if let Some(node) = doc.select_first(".alternative-title").ok() {
        let text = node.text_contents().clean_text();
        if !text.is_empty() {
            let map = HashMap::from([(String::from("role"), String::from("alt"))]);
            metadata.push(Metadata::new(String::from("title"), text, Some(map)))
        }
    }

    let genres = doc.select(".categories > ul > li").ok();
    if let Some(genres) = genres {
        for genre in genres {
            metadata.push(Metadata::new(
                String::from("subject"),
                genre.get_text(),
                None,
            ));
        }
    }

    let tags = doc.select(".content .tag").ok();
    if let Some(tags) = tags {
        for tag in tags {
            metadata.push(Metadata::new(String::from("tag"), tag.get_text(), None));
        }
    }

    metadata
}

fn collect_toc(url: &str) -> Result<Vec<Volume>, QuelleError> {
    let mut volume = Volume::default();

    // parse the first page
    let curl = toc_url(url, 1);
    let response = Request::get(curl).send()?;
    let doc = kuchiki::parse_html().one(response.text().unwrap());
    extract_toc(&doc, &mut volume)?;

    // get page count
    let pages = doc
        .select(".pagenav .pagination > li:not(.PagedList-skipToNext)")
        .map_err(|_| ParseError::ElementNotFound)?
        .collect::<Vec<_>>();

    if pages.len() > 1 {
        let end: usize = pages
            .last()
            .unwrap()
            .as_node()
            .select_first("a")
            .map_err(|_| ParseError::ElementNotFound)?
            .attributes
            .borrow()
            .get("href")
            .unwrap_or("0")
            .split('-')
            .last()
            .unwrap()
            .trim()
            .parse::<usize>()?;

        println!("end: {end}");

        for page in 2..=end {
            let curl = toc_url(url, page);
            let response = Request::get(curl).send()?;
            let doc = kuchiki::parse_html().one(response.text().unwrap());
            extract_toc(&doc, &mut volume)?;
        }
    }

    Ok(vec![volume])
}

fn extract_toc(doc: &NodeRef, volume: &mut Volume) -> Result<(), QuelleError> {
    for li in doc
        .select(".chapter-list > li")
        .map_err(|_| ParseError::ElementNotFound)?
    {
        let Some(a) = li.as_node().select_first("a").ok() else { continue };

        let index = li
            .attributes
            .borrow()
            .get("data-orderno")
            .map(|v| v.parse::<i32>())
            .transpose()?
            .unwrap_or_default();

        let url = a.get_attribute("href").unwrap_or_default();

        let updated_at = li
            .as_node()
            .select_first("time")
            .map(|node| {
                node.attributes.borrow().get("datetime").map(|s| {
                    NaiveDateTime::parse_from_str(s, "%F %R")
                        .map(TaggedDateTime::Local)
                        .ok()
                })
            })
            .ok()
            .flatten()
            .flatten();

        let chapter_no = a.as_node().select_first(".chapter-no").get_text()?;
        let chapter_title = a.as_node().select_first(".chapter-title").get_text()?;

        let chapter = Chapter {
            index,
            title: format!("{} {}", chapter_no.trim(), chapter_title.clean_text()),
            url: META.convert_into_absolute_url(url, None)?,
            updated_at,
        };

        volume.chapters.push(chapter);
    }

    Ok(())
}

fn toc_url(current: &str, page: usize) -> String {
    let stripped = current.strip_suffix("/").unwrap_or(current);
    format!("{stripped}/chapters/page-{page}")
}

#[expose]
pub fn fetch_chapter_content(url: String) -> Result<String, QuelleError> {
    let response = Request::get(url).send()?;
    let doc = kuchiki::parse_html().one(response.text().unwrap());

    let content = doc
        .select_first("#chapter-container")
        .map_err(|_| ParseError::ElementNotFound)?;

    doc.select(".adsbox, .adsbygoogle").detach_all();
    doc.select("strong > strong").detach_all();
    doc.select("strong i i").detach_all();
    doc.select("p > sub").detach_all();

    content.as_node().outer_html().map_err(Into::into)
}
