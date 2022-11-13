use std::collections::HashMap;

use chrono::{DateTime, NaiveDateTime, Utc};
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
    pub volumes: Vec<Volume>,
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

#[derive(Serialize, Deserialize, Debug)]
pub struct Volume {
    pub index: i32,
    pub name: String,
    pub chapters: Vec<Chapter>,
}

impl Default for Volume {
    fn default() -> Self {
        Self {
            index: -1,
            name: String::from("_default"),
            chapters: vec![],
        }
    }
}

#[derive(Serialize, Deserialize, Debug)]
pub struct Chapter {
    pub index: i32,
    pub title: String,
    pub url: String,
    pub updated_at: Option<TaggedDateTime>,
}

#[derive(Serialize, Deserialize, Debug)]
pub enum TaggedDateTime {
    Utc(DateTime<Utc>),
    Local(NaiveDateTime),
}

impl From<DateTime<Utc>> for TaggedDateTime {
    #[inline]
    fn from(value: DateTime<Utc>) -> Self {
        Self::Utc(value)
    }
}

impl From<NaiveDateTime> for TaggedDateTime {
    #[inline]
    fn from(value: NaiveDateTime) -> Self {
        Self::Local(value)
    }
}
