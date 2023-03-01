use std::{collections::HashMap, path::PathBuf};

use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Debug, Default)]
pub struct Global {
    novels: HashMap<String, PathBuf>,
}

impl Global {
    pub fn novel_path_by_url(&self, url: &str) -> Option<&PathBuf> {
        self.novels.get(url)
    }
}
