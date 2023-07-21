use std::{
    collections::HashMap,
    path::{Path, PathBuf},
};

use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Debug, Default)]
pub struct Global {
    novels: HashMap<String, PathBuf>,
}

impl Global {
    pub fn novel_path_by_url(&self, url: &str) -> Option<&Path> {
        self.novels.get(url).map(AsRef::as_ref)
    }

    pub fn insert_novel(&mut self, key: String, value: PathBuf) {
        self.novels.insert(key, value);
    }
}
