use std::{
    fs::{self, File},
    io::BufWriter,
    mem,
    path::{Path, PathBuf},
    thread,
};

use anyhow::bail;
use log::info;
use quelle_core::prelude::{Chapter, Meta, Novel};
use quelle_engine::Runner;
use quelle_persist::{CoverLoc, Persist, PersistNovel, SavedNovel};
use reqwest::{blocking::Client, header::CONTENT_TYPE};
use url::Url;

use crate::download::event::EventKind;

use super::{
    event::{DownloadLog, LogEvent},
    DownloadOptions,
};

pub struct DownloadHandler<'a> {
    pub runner: Runner,
    pub meta: Meta,
    pub persist_novel: PersistNovel<'a>,
    pub data: SavedNovel,
    pub options: DownloadOptions,
    pub log: DownloadLog,
}

pub const LOG_FILENAME: &'static str = "log.jsonl";

fn get_novel_dir(root: &Path, meta: &Meta, novel: &Novel) -> PathBuf {
    let mut save_dir = root.to_path_buf();
    save_dir.push(&meta.id);
    save_dir.push(slug::slugify(&novel.title));
    save_dir
}

fn get_chapters_dir(root: &Path) -> PathBuf {
    root.join("chapters")
}

impl<'a> DownloadHandler<'a> {
    pub fn new(
        persist: &'a Persist,
        url: Url,
        wasm_path: PathBuf,
        options: DownloadOptions,
    ) -> anyhow::Result<Self> {
        let mut runner = Runner::new(&wasm_path)?;
        runner.setup()?;

        let novel = runner.fetch_novel(url.as_str())?;
        let meta = runner.meta()?;

        let save_dir = get_novel_dir(&options.dir, &meta, &novel);
        if !save_dir.exists() {
            fs::create_dir_all(&save_dir)?;
        }

        let persist_novel = persist.persist_novel(persist.novel_path(&meta, &novel.title));
        let data = persist_novel.read_data()?.unwrap_or(SavedNovel::new(novel));

        let log_path = save_dir.join(LOG_FILENAME);
        let log = DownloadLog::open(log_path)?;

        Ok(Self {
            runner,
            meta,
            persist_novel,
            data,
            log,
            options,
        })
    }

    pub fn save(&mut self) -> anyhow::Result<()> {
        // Commit and clear events
        if !self.log.events.is_empty() {
            let events = mem::take(&mut self.log.events);
            self.commit_events(events);
        }

        if self.log.written {
            self.log = DownloadLog::new(mem::take(&mut self.log.path), vec![])?;
        }

        self.persist_novel.write_data(&self.data)?;

        Ok(())
    }

    fn commit_events(&mut self, events: Vec<LogEvent>) {
        for event in events {
            match event.kind {
                EventKind::Downloaded { url, path } => {
                    self.data.downloaded.insert(url, path);
                }
            }
        }
    }

    pub fn download(&mut self) -> anyhow::Result<()> {
        let chapter_dir = get_chapters_dir(&self.persist_novel.dir().join("chapters"));
        if !chapter_dir.exists() {
            fs::create_dir_all(&chapter_dir)?;
        }

        let chapters = self
            .data
            .novel
            .volumes
            .iter()
            .flat_map(|v| &v.chapters)
            .collect::<Vec<_>>();

        let chapters = match self.options.range.as_ref() {
            Some(range) => &chapters[range.clone()],
            None => &chapters,
        };

        Self::download_chapters(
            &mut self.runner,
            &self.data,
            &mut self.log,
            &chapter_dir,
            &chapters,
            self.persist_novel.dir(),
            &self.options,
        )?;

        Ok(())
    }

    fn download_chapters(
        runner: &mut Runner,
        data: &SavedNovel,
        log: &mut DownloadLog,
        chapter_dir: &Path,
        chapters: &[&Chapter],
        save_dir: &Path,
        options: &DownloadOptions,
    ) -> anyhow::Result<()> {
        for chapter in chapters {
            if let Some(path) = data.downloaded.get(&chapter.url) {
                if save_dir.join(path).exists() {
                    continue;
                }
            }

            if let Some(delay) = &options.delay {
                thread::sleep(*delay);
            }

            let content = runner.fetch_chapter_content(&chapter.url)?;
            let filename = format!("{}.html", chapter.index);
            let path = chapter_dir.join(&filename);
            fs::write(&path, content)?;

            info!("Downloaded '{}' to '{}'.", &chapter.title, path.display());

            log.push_event(EventKind::Downloaded {
                url: chapter.url.clone(),
                path: Path::new("chapters").join(&filename),
            })?;
        }

        Ok(())
    }

    pub fn is_cover_downloaded(&self) -> bool {
        let cover = &self.data.cover;
        let Some(cover) = cover else { return false };
        return cover.path.exists() && cover.path.is_file();
    }

    pub fn download_cover(&mut self) -> anyhow::Result<()> {
        let data = &mut self.data;
        let Some(url) = data.novel.cover.as_ref() else { return Ok(()) };

        let client = Client::builder()
            .user_agent(
                "Mozilla/5.0 (X11; Fedora; Linux x86_64; rv:107.0) Gecko/20100101 Firefox/107.0",
            )
            .build()?;

        let mut response = client.get(url).send()?;
        if !response.status().is_success() {
            let status = response.status();
            bail!("Cover download failed with {}", status.as_str());
        }

        info!("Downloaded novel cover from '{url}'.");

        let content_type = response
            .headers()
            .get(CONTENT_TYPE)
            .map(|value| value.to_str().ok())
            .flatten()
            .map(|value| value.to_owned())
            .unwrap_or_default();

        info!("Content type from headers: {content_type}");

        let suffix = mime_guess::get_mime_extensions_str(&content_type).map(|exts| exts[0]);
        let file_name = match suffix {
            Some(suffix) => format!("cover.{suffix}"),
            None => String::from("cover"),
        };

        let path = self.persist_novel.dir().join(file_name);

        let mut file = BufWriter::new(File::create(&path)?);
        response.copy_to(&mut file)?;

        info!("Saved novel cover to '{}'.", path.display());
        data.cover = Some(CoverLoc { path, content_type });

        Ok(())
    }
}
