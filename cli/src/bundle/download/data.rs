use std::{
    collections::HashMap,
    fs::File,
    io::{BufReader, BufWriter},
    path::PathBuf,
};

use chrono::{DateTime, Utc};
use fenster_core::prelude::*;
use serde::{Deserialize, Serialize};

use super::log::{EventKind, LogEvent};

pub struct Tracking {
    pub data: TrackingData,
    pub path: PathBuf,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct TrackingData {
    pub novel: Novel,
    pub downloaded: HashMap<String, PathBuf>,
    pub updated_at: DateTime<Utc>,
}

impl Tracking {
    pub fn new(novel: Novel, path: PathBuf) -> anyhow::Result<Self> {
        let data = if path.exists() {
            let file = File::open(&path)?;
            let file = BufReader::new(file);
            serde_json::from_reader(file)?
        } else {
            TrackingData {
                novel,
                downloaded: HashMap::new(),
                updated_at: Utc::now(),
            }
        };

        Ok(Self { data, path })
    }

    pub fn commit_events(&mut self, events: Vec<LogEvent>) {
        for event in events {
            match event.kind {
                EventKind::Downloaded { url, path } => {
                    self.data.downloaded.insert(url, path);
                }
            }
        }
    }

    pub fn save(&mut self) -> anyhow::Result<()> {
        self.data.updated_at = Utc::now();
        self.write_to_disk()
    }

    pub fn write_to_disk(&self) -> anyhow::Result<()> {
        let mut file = BufWriter::new(File::create(&self.path)?);
        serde_json::to_writer(&mut file, &self.data)?;
        Ok(())
    }

    pub fn is_downloaded(&self, url: &str) -> bool {
        self.data.downloaded.contains_key(url)
    }
}
