use std::{collections::HashMap, path::PathBuf};

use quelle_core::prelude::*;
use serde::{Deserialize, Serialize};

pub struct Bundle {
    pub meta: Option<Meta>,
    pub novel: Novel,
    pub cover: Option<CoverData>,
    pub chapter_content: HashMap<String, PathBuf>,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct CoverData {
    pub path: PathBuf,
    pub content_type: String,
}
