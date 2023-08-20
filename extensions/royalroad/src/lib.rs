mod search;

#[allow(unused_imports)]
#[macro_use]
extern crate quelle_glue;

use chrono::NaiveDateTime;
use kuchiki::{
    iter::{Descendants, Elements, Select},
    traits::TendrilSink,
};
use quelle_core::prelude::*;
use quelle_glue::prelude::*;

pub struct RoyalRoad;

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

expose_basic!(RoyalRoad);
impl FetchBasic for RoyalRoad {
    fn fetch_novel(url: String) -> Result<Novel, QuelleError> {
        let response = Request::get(url.clone()).send()?;
        let doc = kuchiki::parse_html().one(response.text()?.unwrap());

        let volume = Volume {
            chapters: doc
                .select("tbody > tr")
                .map(parse_chapter_list)
                .map_err(|_| ParseError::ElementNotFound)?
                .unwrap_or_default(),
            ..Default::default()
        };

        let author = doc.select_first(".fic-header h4 a").get_text()?;

        let novel = Novel {
            title: doc.select_first(".fic-header h1").get_text()?,
            authors: vec![author],
            cover: doc
                .select_first(".page-content-inner .thumbnail")
                .get_attribute("src"),
            description: doc.select(r#".description > div > p"#).collect_text(),
            status: doc
                .select_first(".fiction-info > .portlet.row span:nth-child(2)")
                .map(|node| node.get_text().as_str().into())
                .unwrap_or_default(),
            langs: META.langs.clone(),
            volumes: vec![volume],
            metadata: doc
                .select(r#"a.label[href*="tag"]"#)
                .map(|nodes| {
                    nodes
                        .map(|node| {
                            Metadata::new(String::from("subject"), node.text_contents(), None)
                        })
                        .collect::<Vec<_>>()
                })
                .unwrap_or_default(),
            url,
        };

        Ok(novel)
    }

    fn fetch_chapter_content(url: String) -> Result<Content, QuelleError> {
        let response = Request::get(url).send()?;
        let doc = kuchiki::parse_html().one(response.text()?.unwrap());

        let content = doc
            .select_first(".chapter-content")
            .map(|node| node.as_node().outer_html())
            .ok()
            .transpose()?
            .ok_or(QuelleError::ParseFailed(ParseError::ElementNotFound))?;

        Ok(content.into())
    }
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

expose_popular!(RoyalRoad);
impl PopularSearch for RoyalRoad {
    fn popular_url(page: i32) -> String {
        format!("https://www.royalroad.com/fictions/weekly-popular?page={page}")
    }

    fn popular(page: i32) -> Result<Vec<BasicNovel>, QuelleError> {
        let url = Self::popular_url(page);
        let response = Request::get(url.clone()).send()?;
        let doc = kuchiki::parse_html().one(response.text()?.unwrap());

        let mut novels = vec![];
        if let Ok(elements) = doc.select(".fiction-list-item") {
            for item in elements {
                let novel_url = item.as_node().select_first("a").get_attribute("href");
                let Some(novel_url) = novel_url else { continue };

                let novel = BasicNovel {
                    title: item.as_node().select_first(".fiction-title").get_text()?,
                    cover: item.as_node().select_first("img").get_attribute("src"),
                    url: META.convert_into_absolute_url(novel_url, Some(&url))?,
                };

                novels.push(novel);
            }
        }

        Ok(novels)
    }
}
