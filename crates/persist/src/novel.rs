use std::{
    collections::HashMap,
    fs::{self, File, OpenOptions},
    io::{BufReader, BufWriter},
    path::{Path, PathBuf},
};

use chrono::{DateTime, Utc};
use quelle_core::prelude::{Chapter, Novel};
use serde::{Deserialize, Serialize};

use crate::{error::PersistResult, event::EventLog, Event, EventKind, Persist};

#[derive(Debug)]
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

    pub fn event_log(&self) -> PersistResult<EventLog> {
        let path = self.dir.join(&self.persist.options.novel.events);
        EventLog::new(path)
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

    #[inline]
    pub fn chapters_dir(&self) -> PathBuf {
        self.dir.join("chapters")
    }

    /// Directory should exist
    pub fn save_chapter(&self, chapter: &Chapter, content: String) -> PersistResult<PathBuf> {
        let name = format!("{}.html", chapter.index);
        let path = self.chapters_dir().join(name);

        fs::write(&path, content)?;
        Ok(path)
    }

    pub fn relative_path(&self, path: PathBuf) -> PathBuf {
        pathdiff::diff_paths(&path, &self.dir).unwrap_or(path)
    }

    pub fn cover_path(&self, file_type: Option<&str>) -> PathBuf {
        let name = match file_type {
            Some(s) => format!("cover.{s}"),
            None => String::from("cover"),
        };

        self.dir.join(name)
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

    pub fn is_cover_downloaded(&self) -> bool {
        match &self.cover {
            Some(cover) => cover.path.exists() && cover.path.is_file(),
            None => false,
        }
    }

    pub fn commit_events(&mut self, events: Vec<Event>) {
        for event in events {
            match event.kind {
                EventKind::Downloaded { url, path } => {
                    self.downloaded.insert(url, path);
                }
            }
        }
    }
}
