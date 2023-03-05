use kuchiki::{traits::TendrilSink, NodeRef};
use quelle_core::prelude::*;
use quelle_glue::prelude::*;
use serde_json::json;

define_meta! {
    let META = {
        id: "en.1stkissnovel",
        name: "1stkissnovel",
        langs: ["en"],
        base_urls: ["https://1stkissnovel.love"],
        rds: [Ltr],
        attrs: [Mtl],
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

    let title = doc
        .select_first(".post-title h1")
        .map_err(|_| ParseError::ElementNotFound)?;
    title.as_node().select("span").detach_all();
    let title = title.get_text();

    Ok(Novel {
        title,
        authors: doc
            .select(".author-content a[href*='manga-author']")
            .collect_text(),
        cover: doc
            .select_first(".summary_image a img")
            .get_attribute("data-src")
            .map(|src| META.convert_into_absolute_url(src, Some(&url)))
            .transpose()?,
        description: doc
            .select_first(".summary__content")
            .get_text()?
            .lines()
            .map(|str| str.trim().to_string())
            .collect(),
        volumes: collect_volumes(&url, &doc)?,
        metadata: doc
            .select(".genres-content > a")
            .map(|nodes| {
                nodes
                    .map(|node| Metadata::new(String::from("subject"), node.get_text(), None))
                    .collect()
            })
            .unwrap_or_default(),
        status: doc
            .select_first(".post-status .post-content_item:nth-child(2) .summary-content")
            .map(|node| node.get_text().as_str().into())
            .unwrap_or_default(),
        langs: META.langs.clone(),
        url,
    })
}

fn collect_volumes(url: &str, doc: &NodeRef) -> Result<Vec<Volume>, QuelleError> {
    let mut volume = Volume::default();

    let id = doc
        .select_first("#manga-chapters-holder")
        .get_attribute("data-id")
        .ok_or(ParseError::ElementNotFound)?;

    let data = json!({
        "action": "manga_get_chapters",
        "manga": id,
    });

    let response = Request::post(String::from(
        "https://1stkissnovel.love/wp-admin/admin-ajax.php",
    ))
    .json_data(&data)
    .map_err(|_| QuelleError::JsonError)?
    .send()?;

    let doc = kuchiki::parse_html().one(response.text().unwrap());
    let nodes = doc.select(".wp-manga-chapter a");
    if let Ok(nodes) = nodes {
        for node in nodes {
            let Some(href) = node.get_attribute("href") else { continue };

            let chapter = Chapter {
                index: volume.chapters.len() as i32,
                title: node.get_text(),
                url: META.convert_into_absolute_url(href, Some(url))?,
                updated_at: None, // TODO: relative time
            };

            volume.chapters.push(chapter);
        }
    }

    Ok(vec![volume])
}
