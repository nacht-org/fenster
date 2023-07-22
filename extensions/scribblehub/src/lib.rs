#[allow(unused_imports)]
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
        id: "en.scribblehub",
        name: "ScribbleHub",
        langs: ["en"],
        base_urls: ["https://www.scribblehub.com"],
        rds: [Ltr],
        attrs: [],
    };
}

#[expose]
pub fn fetch_novel(url: String) -> Result<Novel, QuelleError> {
    let response = Request::get(url.clone()).send()?;

    let doc = kuchiki::parse_html().one(response.text().unwrap());

    let id = url
        .split("/")
        .nth(4)
        .ok_or_else(|| ParseError::Other(String::from("The url does not have an id")))?;

    let novel = Novel {
        title: doc.select_first("div.fic_title").get_text()?,
        authors: vec![doc.select_first("span.auth_name_fic").get_text()?],
        description: doc.select(".wi_fic_desc > p").collect_text(),
        langs: META.langs.clone(),
        cover: doc.select_first(".fic_image img").get_attribute("src"),
        status: doc
            .select_first(".widget_fic_similar > li:last-child > span:last-child")
            .map(|node| node.get_text().as_str().into())
            .unwrap_or_default(),
        volumes: volumes(id)?,
        metadata: metadata(&doc)?,
        url,
    };

    Ok(novel)
}

fn metadata(doc: &NodeRef) -> Result<Vec<Metadata>, QuelleError> {
    let mut metadata = vec![];

    if let Ok(nodes) = doc.select("a.fic_genre") {
        for node in nodes {
            metadata.push(Metadata::new(
                String::from("subject"),
                node.get_text(),
                None,
            ));
        }
    }

    if let Ok(nodes) = doc.select("a.stag") {
        for node in nodes {
            metadata.push(Metadata::new(String::from("tag"), node.get_text(), None));
        }
    }

    if let Ok(nodes) = doc.select(".mature_contains > a") {
        for node in nodes {
            metadata.push(Metadata::new(
                String::from("warning"),
                node.get_text(),
                None,
            ));
        }
    }

    let rating_element = doc.select_first("#ratefic_user > span");
    if let Some(element) = rating_element.ok() {
        metadata.push(Metadata::new(
            String::from("rating"),
            element.get_text(),
            None,
        ));
    }

    Ok(metadata)
}

fn volumes(id: &str) -> Result<Vec<Volume>, QuelleError> {
    let mut data = HashMap::new();
    data.insert(
        String::from("action"),
        String::from("wi_getreleases_pagination"),
    );
    data.insert(String::from("pagenum"), String::from("-a"));
    data.insert(String::from("mypostid"), id.to_string());

    let response = Request::post(String::from(
        "https://www.scribblehub.com/wp-admin/admin-ajax.php",
    ))
    .form(data)
    .send()?;

    let doc = kuchiki::parse_html().one(response.text().unwrap());
    let mut volume = Volume::default();

    if let Ok(nodes) = doc.select("li.toc_w") {
        for node in nodes.rev() {
            let Ok(a) = node.as_node().select_first("a") else { continue };
            let Some(href) = a.get_attribute("href") else { continue };

            let time = node
                .as_node()
                .select_first(".fic_date_pub")
                .get_attribute("title");

            // TODO: parse relative time
            let updated_at = time
                .map(|time| NaiveDateTime::parse_from_str(&time, "").ok())
                .flatten()
                .map(TaggedDateTime::Local);

            let chapter = Chapter {
                index: volume.chapters.len() as i32,
                title: a.get_text(),
                url: href,
                updated_at,
            };

            volume.chapters.push(chapter);
        }
    }

    Ok(vec![volume])
}

#[expose]
pub fn fetch_chapter_content(url: String) -> Result<String, QuelleError> {
    let response = Request::get(url).send()?;
    let doc = kuchiki::parse_html().one(response.text().unwrap());

    let content = doc
        .select_first("#chp_raw")
        .map(|node| node.as_node().outer_html())
        .ok()
        .transpose()?
        .ok_or(QuelleError::ParseFailed(ParseError::ElementNotFound))?;

    Ok(content)
}
