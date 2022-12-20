use std::{
    fs::{File, OpenOptions},
    io::{BufRead, BufReader, LineWriter, Write},
    path::{Path, PathBuf},
};

use anyhow::Context;
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

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
