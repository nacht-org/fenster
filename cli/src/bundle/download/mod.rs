mod data;
mod log;
mod options;

use std::{
    fs::{self},
    mem,
    path::{Path, PathBuf},
};

use fenster_core::prelude::{Chapter, Meta};
use fenster_engine::Runner;
use url::Url;

use self::{
    data::Tracking,
    log::{DownloadLog, EventKind},
};

pub use options::DownloadOptions;

pub fn download(
    url: Url,
    wasm_path: PathBuf,
    options: DownloadOptions,
) -> anyhow::Result<Tracking> {
    let mut handler = DownloadHandler::new(url, wasm_path, options)?;

    handler.save()?;
    handler.download()?;
    handler.save()?;

    Ok(handler.tracking)
}

pub struct DownloadHandler {
    runner: Runner,
    meta: Meta,
    save_dir: PathBuf,
    log: DownloadLog,
    tracking: Tracking,
    options: DownloadOptions,
}

impl DownloadHandler {
    pub fn new(url: Url, wasm_path: PathBuf, options: DownloadOptions) -> anyhow::Result<Self> {
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
            options,
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

        let chapters = self
            .tracking
            .data
            .novel
            .volumes
            .iter()
            .flat_map(|v| &v.chapters)
            .collect::<Vec<_>>();

        if let Some(range) = self.options.range.as_ref() {
            Self::download_chapters(
                &mut self.runner,
                &self.tracking,
                &mut self.log,
                &chapter_dir,
                &chapters[range.clone()],
            )?;
        } else {
            Self::download_chapters(
                &mut self.runner,
                &self.tracking,
                &mut self.log,
                &chapter_dir,
                &chapters,
            )?;
        }

        Ok(())
    }

    fn download_chapters(
        runner: &mut Runner,
        tracking: &Tracking,
        log: &mut DownloadLog,
        chapter_dir: &Path,
        chapters: &[&Chapter],
    ) -> anyhow::Result<()> {
        for chapter in chapters {
            if tracking.is_downloaded(&chapter.url) {
                continue;
            }

            let content = runner.fetch_chapter_content(&chapter.url)?;
            let Some(content) = content else { continue };

            let filename = format!("{}.html", chapter.index);
            let path = chapter_dir.join(&filename);
            fs::write(&path, content)?;

            log.push_event(EventKind::Downloaded {
                url: chapter.url.clone(),
                path: Path::new("chapters").join(&filename),
            })?;
        }

        Ok(())
    }
}
