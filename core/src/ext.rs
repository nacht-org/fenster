use std::collections::HashMap;

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
    pub authors: Vec<String>,
    pub url: String,
    pub thumb: Option<String>,
    pub desc: Vec<String>,
    pub metadata: Vec<Metadata>,
    pub lang: Vec<String>,
}

const DUBLIN_CORE: [&'static str; 10] = [
    "title",
    "language",
    "subject",
    "creator",
    "contributor",
    "publisher",
    "rights",
    "coverage",
    "date",
    "description",
];

#[derive(Serialize, Deserialize, Debug)]
pub struct Metadata {
    name: String,
    value: String,
    ns: Namespace,
    others: HashMap<String, String>,
}

#[derive(Serialize, Deserialize, Debug)]
pub enum Namespace {
    DC,
    OPF,
}

impl Metadata {
    pub fn new(name: String, value: String, others: Option<HashMap<String, String>>) -> Self {
        let ns = if DUBLIN_CORE.contains(&name.as_str()) {
            Namespace::DC
        } else {
            Namespace::OPF
        };

        Metadata {
            name,
            value,
            ns,
            others: others.unwrap_or_default(),
        }
    }
}
