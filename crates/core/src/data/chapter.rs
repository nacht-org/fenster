use chrono::{DateTime, NaiveDateTime, Utc};
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Debug)]
pub struct Chapter {
    pub index: i32,
    pub title: String,
    pub url: String,
    pub updated_at: Option<TaggedDateTime>,
}

#[derive(Serialize, Deserialize, Debug)]
#[serde(rename_all = "snake_case")]
pub enum TaggedDateTime {
    Utc(DateTime<Utc>),
    Local(NaiveDateTime),
}

#[derive(Serialize, Deserialize, Default, Debug)]
pub struct Content {
    pub data: String,
}

impl From<String> for Content {
    fn from(value: String) -> Self {
        Content { data: value }
    }
}
