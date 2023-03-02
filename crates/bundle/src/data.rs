use std::{collections::HashMap, path::PathBuf};

use quelle_core::prelude::*;
use serde::{Deserialize, Serialize};

pub struct Bundle {
    pub meta: Option<Meta>,
    pub novel: Novel,
    pub cover: Option<Cover>,
    pub chapter_content: HashMap<String, PathBuf>,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct Cover {
    pub path: PathBuf,
    pub content_type: String,
}

#[cfg(feature = "persist")]
impl From<quelle_persist::CoverLoc> for Cover {
    fn from(value: quelle_persist::CoverLoc) -> Self {
        Cover {
            path: value.path,
            content_type: value.content_type,
        }
    }
}
