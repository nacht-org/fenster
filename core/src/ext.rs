use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Debug)]
pub struct Meta<'a> {
    pub id: &'a str,
    pub name: &'a str,
    pub lang: &'a str,
    pub version: [usize; 3],
    pub base_urls: Vec<&'a str>,
    pub rds: Vec<ReadingDirection>,
    pub attrs: Vec<Attribute>,
}

#[derive(Serialize, Deserialize, Debug)]
pub enum ReadingDirection {
    Ltr,
    Rtl,
}

#[derive(Serialize, Deserialize, Debug)]
pub enum Attribute {
    Fanfiction,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct Novel {
    pub title: String,
    pub author: Vec<String>,
    pub url: String,
    pub thumb: Option<String>,
    pub desc: Vec<String>,
    pub lang: Vec<String>,
}
