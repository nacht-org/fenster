use kuchiki::{traits::TendrilSink, NodeRef};
use quelle_core::prelude::*;
use quelle_glue::prelude::*;

use crate::META;

#[expose]
pub fn text_search(query: String, page: i32) -> Result<Vec<BasicNovel>, QuelleError> {
    let home_url = META.home_url();
    let url = format!("{home_url}/search?keyword={query}&page={page}",);
    let response = Request::get(url.clone()).send()?;
    let doc = kuchiki::parse_html().one(response.text()?.unwrap());
    parse_search(url, doc)
}

fn parse_search(url: String, doc: NodeRef) -> Result<Vec<BasicNovel>, QuelleError> {
    // The search is limited to 20 novels per page
    let mut novels = Vec::with_capacity(20);

    let elements = doc
        .select("#list-page .row")
        .map_err(|_| ParseError::other("error while selecting novels"))?;

    for element in elements {
        let title_element = element.as_node().select_first("h3[class*='title'] > a");
        let Some(href) = title_element.get_attribute("href") else { continue };

        let cover = element
            .as_node()
            .select_first(".cover")
            .get_attribute("src")
            .map(|src| META.abs_url(src, &url))
            .transpose()?;

        let novel = BasicNovel {
            title: title_element.get_text()?,
            cover,
            url: META.abs_url(href, &url)?,
        };

        novels.push(novel);
    }

    Ok(novels)
}
