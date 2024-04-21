use serde::{Deserialize, Serialize};
use url::Url;

use super::{Attribute, ReadingDirection};
use crate::error::ParseError;

#[derive(Serialize, Deserialize, Debug, Clone, Default)]
pub struct Meta {
    pub id: String,
    pub name: String,
    pub langs: Vec<String>,
    pub version: String,
    pub base_urls: Vec<String>,
    pub rds: Vec<ReadingDirection>,
    pub attrs: Vec<Attribute>,
}

impl Meta {
    pub fn convert_into_absolute_url(
        &self,
        mut url: String,
        current: Option<&str>,
    ) -> Result<String, ParseError> {
        if url.starts_with("https://") || url.starts_with("http://") {
            return Ok(url);
        }

        let resolved_url = Url::parse(current.unwrap_or(&self.base_urls[0]))
            .map_err(|_| ParseError::FailedURLParse)?;

        if url.starts_with("//") {
            url.insert_str(0, &format!("{}:", resolved_url.scheme()));
        } else if url.starts_with('/') {
            url.insert_str(0, &base_url(resolved_url));
        } else if let Some(current) = current {
            let base_url = current.strip_suffix('/').unwrap_or(current);
            url.insert_str(0, "/");
            url.insert_str(0, base_url);
        }

        Ok(url)
    }

    #[inline]
    pub fn abs_url(&self, url: String, current: &str) -> Result<String, ParseError> {
        self.convert_into_absolute_url(url, Some(current))
    }

    pub fn home_url(&self) -> &str {
        &self.base_urls[0]
    }
}

fn base_url(url: Url) -> String {
    format!("{}://{}", url.scheme(), url.host_str().unwrap_or_default())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn should_convert_into_absolute_url_with_scheme() {
        let meta = Meta {
            base_urls: vec![String::from("https://base.example.com")],
            ..Default::default()
        };

        assert_eq!(
            String::from("https://example.com"),
            meta.convert_into_absolute_url(String::from("https://example.com"), None)
                .unwrap(),
        );

        assert_eq!(
            String::from("http://example.com"),
            meta.convert_into_absolute_url(String::from("http://example.com"), None)
                .unwrap(),
        );
    }

    #[test]
    fn should_convert_into_absolute_url_without_scheme() {
        let meta = Meta {
            base_urls: vec![String::from("https://base.example.com")],
            ..Default::default()
        };

        assert_eq!(
            String::from("https://example.com"),
            meta.convert_into_absolute_url(String::from("//example.com"), None)
                .unwrap(),
        );

        assert_eq!(
            String::from("http://example.com"),
            meta.convert_into_absolute_url(
                String::from("//example.com"),
                Some("http://current.example.com")
            )
            .unwrap(),
        );
    }

    #[test]
    fn should_convert_into_absolute_url_without_base() {
        let meta = Meta {
            base_urls: vec![String::from("https://base.example.com")],
            ..Default::default()
        };

        assert_eq!(
            String::from("https://base.example.com/page/1"),
            meta.convert_into_absolute_url(String::from("/page/1"), None)
                .unwrap(),
        );

        assert_eq!(
            String::from("http://current.example.com/page/1"),
            meta.convert_into_absolute_url(
                String::from("/page/1"),
                Some("http://current.example.com")
            )
            .unwrap(),
        );
    }

    #[test]
    fn should_convert_into_absolute_url_relative() {
        let meta = Meta {
            base_urls: vec![String::from("https://base.example.com")],
            ..Default::default()
        };

        assert_eq!(
            String::from("page/1"),
            meta.convert_into_absolute_url(String::from("page/1"), None)
                .unwrap(),
        );

        assert_eq!(
            String::from("http://current.example.com/extend/page/1"),
            meta.convert_into_absolute_url(
                String::from("page/1"),
                Some("http://current.example.com/extend")
            )
            .unwrap(),
        );

        assert_eq!(
            String::from("http://current.example.com/extend/page/1"),
            meta.convert_into_absolute_url(
                String::from("page/1"),
                Some("http://current.example.com/extend/")
            )
            .unwrap(),
        );
    }

    #[test]
    fn should_get_base_url() {
        assert_eq!(
            String::from("https://example.com"),
            base_url(Url::parse("https://example.com/page/1").unwrap())
        );
        assert_eq!(
            String::from("https://example.com"),
            base_url(Url::parse("https://example.com/").unwrap())
        );
        assert_eq!(
            String::from("http://example.com"),
            base_url(Url::parse("http://example.com/page/1").unwrap())
        );
    }
}
