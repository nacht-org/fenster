#[allow(unused_imports)]
#[macro_use]
extern crate quelle_glue;

use kuchiki::{traits::TendrilSink, NodeRef};
use quelle_core::prelude::*;
use quelle_glue::prelude::*;

define_meta! {
    let META = {
        id: "en.novelfull",
        name: "NovelFull",
        langs: ["en"],
        base_urls: ["https://novelfull.com", "http://novelfull.com"],
        rds: [Ltr],
        attrs: [],
    };
}

#[expose]
pub fn fetch_novel(url: String) -> Result<Novel, QuelleError> {
    let response = Request::get(url.clone()).send()?;
    let content = response.text()?.unwrap();
    let has_chapter_option_url = content.find("var ajaxChapterOptionUrl =").is_some();
    let doc = kuchiki::parse_html().one(content);

    let novel = Novel {
        title: doc.select_first(".title").get_text()?,
        authors: doc.select("a[href^='/author']").collect_text(),
        cover: doc.select_first(".book img").get_attribute("src"),
        description: doc.select(".desc-text").collect_text(),
        volumes: volumes(&url, &doc, has_chapter_option_url)?,
        metadata: metadata(&doc)?,
        status: doc
            .select_first("a[href^='/status/']")
            .get_text()
            .map(|value| NovelStatus::from(value.as_ref()))
            .unwrap_or_default(),
        langs: META.langs.clone(),
        url: url,
    };

    Ok(novel)
}

fn metadata(doc: &NodeRef) -> Result<Vec<Metadata>, QuelleError> {
    let mut metadata = vec![];

    let elements = doc.select(".info a[href^='/genre/']");
    if let Ok(elements) = elements {
        for element in elements {
            metadata.push(Metadata::new(
                String::from("subject"),
                element.get_text(),
                None,
            ));
        }
    }

    Ok(metadata)
}

fn volumes(
    novel_url: &str,
    doc: &NodeRef,
    has_chapter_option_url: bool,
) -> Result<Vec<Volume>, QuelleError> {
    let mut volume = Volume::default();

    let novel_id = doc
        .select_first("#rating[data-novel-id]")
        .get_attribute("data-novel-id");

    let Some(novel_id) = novel_id else {
        return Err(QuelleError::ParseFailed(ParseError::Other(String::from("novel id not found"))))
    };

    let home_url = &META.base_urls[0];
    let url = if has_chapter_option_url {
        format!("{}/ajax-chapter-option?novelId={}", home_url, novel_id)
    } else {
        format!("{}/ajax/chapter-archive?novelId={}", home_url, novel_id)
    };

    let response = Request::get(url.clone()).send()?;
    let chapterlist_doc = kuchiki::parse_html().one(response.text()?.unwrap());

    let elements = chapterlist_doc.select("ul.list-chapter > li > a[href], select > option[value]");
    if let Ok(elements) = elements {
        for element in elements {
            let url = element
                .get_attribute("href")
                .map(Some)
                .unwrap_or_else(|| element.get_attribute("value"));

            let Some(url) = url else { continue };

            let chapter = Chapter {
                index: volume.chapters.len() as i32,
                title: element.get_text(),
                url: META.convert_into_absolute_url(url, Some(novel_url))?,
                updated_at: None,
            };

            volume.chapters.push(chapter);
        }
    }

    Ok(vec![volume])
}

#[expose]
pub fn fetch_chapter_content(url: String) -> Result<Content, QuelleError> {
    let response = Request::get(url).send()?;
    let doc = kuchiki::parse_html().one(response.text()?.unwrap());

    let content = doc
        .select_first("#chr-content, #chapter-content")
        .map_err(|_| ParseError::ElementNotFound)?;

    content.attributes.borrow_mut().map.clear();

    let bad_selectors = [
        ".ads, .ads-holder, .ads-middle",
        "div[align='left']",
        "img[src*='proxy?container=focus']",
        "div[id^='pf-']",
    ];

    for selector in bad_selectors {
        content.as_node().select(selector).detach_all();
    }

    Ok(Content {
        data: content.as_node().outer_html()?,
        ..Default::default()
    })
}
