use serde::{Deserialize, Serialize};

use super::{Metadata, NovelStatus, Volume};

#[derive(Serialize, Deserialize, Debug, Default)]
pub struct Novel {
    pub url: String,
    pub authors: Vec<String>,
    pub title: String,
    pub cover: Option<String>,
    pub description: Vec<String>,
    pub volumes: Vec<Volume>,
    pub metadata: Vec<Metadata>,
    pub status: NovelStatus,
    pub langs: Vec<String>,
}

#[derive(Serialize, Deserialize, Debug, Default)]
pub struct BasicNovel {
    pub title: String,
    pub cover: Option<String>,
    pub url: String,
}
