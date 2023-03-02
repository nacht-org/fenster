use std::{
    collections::HashMap,
    fs::{self, File, OpenOptions},
    io::{BufReader, BufWriter},
    path::{Path, PathBuf},
};

use chrono::{DateTime, Utc};
use quelle_core::prelude::Novel;
use serde::{Deserialize, Serialize};

use crate::{error::PersistResult, Persist};

pub struct PersistNovel<'a> {
    dir: PathBuf,
    persist: &'a Persist,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct SavedNovel {
    pub novel: Novel,
    pub cover: Option<CoverLoc>,
    pub downloaded: HashMap<String, PathBuf>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct CoverLoc {
    pub path: PathBuf,
    pub content_type: String,
}

impl<'a> PersistNovel<'a> {
    pub fn new(dir: PathBuf, persist: &'a Persist) -> Self {
        PersistNovel { dir, persist }
    }

    pub fn dir(&self) -> &Path {
        &self.dir
    }

    pub fn read_data(&self) -> PersistResult<Option<SavedNovel>> {
        let path = self.persist.options.novel.file_path();

        let data = if path.exists() {
            let file = File::open(&path)?;
            let file = BufReader::new(file);
            Some(serde_json::from_reader(file)?)
        } else {
            None
        };

        Ok(data)
    }

    pub fn write_data(&self, data: &SavedNovel) -> PersistResult<()> {
        let path = self.persist.options.novel.file_path();

        if let Some(parent) = path.parent() {
            if !parent.exists() {
                fs::create_dir_all(parent)?;
            }
        }

        let file = OpenOptions::new()
            .create(true)
            .truncate(true)
            .write(true)
            .open(path)?;

        let writer = BufWriter::new(file);
        serde_json::to_writer(writer, data)?;

        Ok(())
    }
}

impl SavedNovel {
    pub fn new(novel: Novel) -> Self {
        Self {
            novel,
            cover: None,
            downloaded: Default::default(),
            updated_at: Utc::now(),
        }
    }
}
