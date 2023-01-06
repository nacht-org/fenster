use std::path::PathBuf;

use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Debug)]
pub struct CoverData {
    pub path: PathBuf,
    pub content_type: String,
}
