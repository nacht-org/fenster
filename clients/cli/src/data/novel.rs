use std::{
    collections::HashMap,
    fs::{File, OpenOptions},
    io::{BufRead, BufReader, BufWriter, LineWriter, Write},
    path::{Path, PathBuf},
};

use anyhow::Context;
use chrono::{DateTime, Utc};
use fenster_core::prelude::*;
use serde::{Deserialize, Serialize};

pub struct NovelTracking {
    pub data: TrackingData,
    pub path: PathBuf,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct TrackingData {
    pub novel: Novel,
    pub downloaded: HashMap<String, PathBuf>,
    pub updated_at: DateTime<Utc>,
}

impl NovelTracking {
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

#[derive(Debug)]
pub struct DownloadLog {
    pub events: Vec<LogEvent>,
    pub file: LineWriter<File>,
    pub path: PathBuf,
    pub written: bool,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct LogEvent {
    pub kind: EventKind,
    pub added_at: DateTime<Utc>,
}

#[derive(Serialize, Deserialize, Debug)]
pub enum EventKind {
    Downloaded { url: String, path: PathBuf },
}

impl DownloadLog {
    pub fn open(path: PathBuf) -> anyhow::Result<Self> {
        let events = if path.exists() {
            Self::read_events_from_file(&path)?
        } else {
            vec![]
        };

        Self::new(path, events)
    }

    pub fn new(path: PathBuf, events: Vec<LogEvent>) -> anyhow::Result<Self> {
        let file = OpenOptions::new()
            .create(true)
            .write(true)
            .truncate(true)
            .open(&path)?;

        Ok(Self {
            events,
            file: LineWriter::new(file),
            path,
            written: false,
        })
    }

    fn read_events_from_file(path: &Path) -> anyhow::Result<Vec<LogEvent>> {
        let file = File::open(path).with_context(|| "failed to open tracking file.")?;
        let file = BufReader::new(file);
        let mut events = vec![];
        for line in file.lines() {
            let line = line?;
            if line.is_empty() {
                continue;
            };
            let event = serde_json::from_str(&line)?;
            events.push(event);
        }
        Ok(events)
    }

    pub fn push_event(&mut self, kind: EventKind) -> anyhow::Result<()> {
        let event = LogEvent {
            kind,
            added_at: Utc::now(),
        };

        let bytes = serde_json::to_vec(&event)?;
        self.file.write(&bytes)?;
        self.file.write(b"\n")?;
        self.events.push(event);

        if !self.written {
            self.written = true;
        }

        Ok(())
    }
}
