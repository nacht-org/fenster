use std::{
    fs::{File, OpenOptions},
    io::{self, BufRead, BufReader, LineWriter, Write},
    path::PathBuf,
};

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

use crate::{create_parent_all, error::PersistResult};

#[derive(Debug)]
pub struct EventLog {
    events: Option<Vec<Event>>,
    file: LineWriter<File>,
    path: PathBuf,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct Event {
    pub kind: EventKind,
    pub added_at: DateTime<Utc>,
}

#[derive(Serialize, Deserialize, Debug)]
pub enum EventKind {
    Downloaded { url: String, path: PathBuf },
}

impl EventLog {
    pub fn new(path: PathBuf) -> PersistResult<Self> {
        create_parent_all(&path)?;

        let file = OpenOptions::new().create(true).append(true).open(&path)?;
        let file = LineWriter::new(file);

        Ok(Self {
            events: None,
            file,
            path,
        })
    }

    pub fn read_events(&mut self) -> PersistResult<Option<Vec<Event>>> {
        let file = BufReader::new(File::open(&self.path)?);
        let mut events = vec![];
        for line in file.lines() {
            let line = line?;
            if line.is_empty() {
                continue;
            };
            let event = serde_json::from_str(&line)?;
            events.push(event);
        }

        if events.is_empty() {
            Ok(self.events.take())
        } else {
            Ok(self.events.replace(events))
        }
    }

    pub fn push_event(&mut self, kind: EventKind) -> PersistResult<()> {
        let event = Event {
            kind,
            added_at: Utc::now(),
        };

        let bytes = serde_json::to_vec(&event)?;
        self.file.write(&bytes)?;
        self.file.write(b"\n")?;

        match self.events.as_mut() {
            Some(events) => {
                events.push(event);
            }
            None => {
                self.events = Some(vec![event]);
            }
        }

        Ok(())
    }

    pub fn take_events(&mut self) -> Option<Vec<Event>> {
        self.events.take()
    }

    pub fn truncate(&mut self) -> io::Result<()> {
        let file = OpenOptions::new()
            .create(true)
            .append(true)
            .truncate(true)
            .open(&self.path)?;

        self.file = LineWriter::new(file);
        Ok(())
    }
}
