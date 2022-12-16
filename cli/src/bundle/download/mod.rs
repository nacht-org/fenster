mod data;
mod log;

use std::{
    fs::{self},
    mem,
    path::PathBuf,
};

use fenster_core::prelude::Meta;
use fenster_engine::Runner;
use url::Url;

use self::{
    data::Tracking,
    log::{DownloadLog, EventKind},
};

pub struct DownloadHandler {
    runner: Runner,
    meta: Meta,
    save_dir: PathBuf,
    log: DownloadLog,
    tracking: Tracking,
}

impl DownloadHandler {
    pub fn new(url: Url, wasm_path: PathBuf) -> anyhow::Result<Self> {
        let mut runner = Runner::new(&wasm_path)?;

        let novel = runner.fetch_novel(url.as_str())?;
        let meta = runner.meta()?;

        let mut save_dir = PathBuf::from("data");
        save_dir.push(&meta.id);
        save_dir.push(slug::slugify(&novel.title));
        if !save_dir.exists() {
            fs::create_dir_all(&save_dir)?;
        }

        let tracking_path = save_dir.join("tracking.json");
        let tracking = Tracking::new(novel, tracking_path)?;

        let log_path = save_dir.join("log.jsonl");
        let log = DownloadLog::open(log_path)?;

        Ok(Self {
            runner,
            meta,
            save_dir,
            tracking,
            log,
        })
    }

    pub fn save(&mut self) -> anyhow::Result<()> {
        // Commit and clear events
        if !self.log.events.is_empty() {
            let events = mem::take(&mut self.log.events);
            self.tracking.commit_events(events);
        }

        if self.log.written {
            self.log = DownloadLog::new(mem::take(&mut self.log.path), vec![])?;
        }

        self.tracking.save()?;

        Ok(())
    }

    pub fn download(&mut self) -> anyhow::Result<()> {
        let chapter_dir = self.save_dir.join("chapters");
        if !chapter_dir.exists() {
            fs::create_dir_all(&chapter_dir)?;
        }

        for volume in &self.tracking.data.novel.volumes {
            for chapter in &volume.chapters {
                if self.tracking.is_downloaded(&chapter.url) {
                    continue;
                }

                let content = self.runner.fetch_chapter_content(&chapter.url)?;
                let Some(content) = content else { continue };

                let path = chapter_dir.join(format!("{}.html", chapter.index));
                fs::write(&path, content)?;
                self.log.push_event(EventKind::Downloaded {
                    url: chapter.url.clone(),
                    path,
                })?;
            }
        }

        Ok(())
    }
}
