use std::{
    collections::HashMap,
    fs::{File, OpenOptions},
    io::{BufReader, BufWriter},
    path::{Path, PathBuf},
};

use serde::{Deserialize, Serialize};

use crate::{create_parent_all, error::PersistResult};

#[derive(Serialize, Deserialize, Debug, Default)]
pub struct Global {
    novels: HashMap<String, PathBuf>,
}

impl Global {
    pub fn open(path: &Path) -> PersistResult<Self> {
        let data = if path.exists() {
            let file = File::open(path)?;
            let reader = BufReader::new(file);
            serde_json::from_reader(reader)?
        } else {
            Default::default()
        };

        Ok(data)
    }

    pub fn save(&self, path: &Path) -> PersistResult<()> {
        create_parent_all(path)?;

        let file = OpenOptions::new()
            .create(true)
            .truncate(true)
            .write(true)
            .open(path)?;

        let writer = BufWriter::new(file);
        serde_json::to_writer(writer, self)?;

        Ok(())
    }

    pub fn novel_path_from_url(&self, url: &str) -> Option<&Path> {
        self.novels.get(url).map(AsRef::as_ref)
    }

    pub fn insert_novel(&mut self, url: String, path: PathBuf) {
        self.novels.insert(url, path);
    }
}
