use std::{
    collections::HashMap,
    fs::{self, File},
    io::{BufReader, BufWriter},
    path::{Path, PathBuf},
};

use anyhow::Context;
use serde::{Deserialize, Serialize};

#[derive(Debug)]
pub struct GlobalTracker {
    pub data: GlobalData,
    pub path: PathBuf,
}

#[derive(Serialize, Deserialize, Debug, Default)]
pub struct GlobalData {
    novels: HashMap<String, PathBuf>,
}

impl GlobalTracker {
    pub fn in_dir(dir: &Path) -> anyhow::Result<Self> {
        let path = dir.join("global.json");
        Self::open(path)
    }

    pub fn open(path: PathBuf) -> anyhow::Result<Self> {
        let data = if path.exists() {
            let file = File::open(&path).with_context(|| "Failed to open global file")?;
            let reader = BufReader::new(file);
            serde_json::from_reader(reader)?
        } else {
            Default::default()
        };

        Ok(GlobalTracker { data, path })
    }

    pub fn save(&self) -> anyhow::Result<()> {
        if let Some(parent) = self.path.parent() {
            if !parent.exists() {
                fs::create_dir_all(parent)?;
            }
        }

        let mut file = BufWriter::new(File::create(&self.path)?);
        serde_json::to_writer(&mut file, &self.data)?;
        Ok(())
    }
}

impl GlobalData {
    pub fn insert_novel(&mut self, key: String, value: PathBuf) {
        self.novels.insert(key, value);
    }

    pub fn get_path_for_url(&self, url: &str) -> Option<&Path> {
        self.novels.get(url).map(|v| v.as_path())
    }
}
