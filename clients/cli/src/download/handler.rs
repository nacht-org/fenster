use std::{
    fs::{self, File},
    io::BufWriter,
    path::{Path, PathBuf},
    thread,
};

use anyhow::bail;
use log::info;
use quelle_core::prelude::{Chapter, Meta};
use quelle_engine::Runner;
use quelle_persist::{CoverLoc, EventKind, EventLog, Persist, PersistNovel, SavedNovel};
use reqwest::{blocking::Client, header::CONTENT_TYPE};
use url::Url;

use super::DownloadOptions;

pub struct DownloadHandler<'a> {
    pub runner: Runner,
    pub meta: Meta,
    pub persist_novel: PersistNovel<'a>,
    pub data: SavedNovel,
    pub options: DownloadOptions,
    pub log: EventLog,
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
        if novel.title.is_empty() {
            bail!("The novel title cannot be empty");
        }

        let meta = runner.meta()?;

        let persist_novel = persist.persist_novel(persist.novel_path(&meta, &novel.title));
        let data = persist_novel
            .read_data()?
            .unwrap_or_else(|| SavedNovel::new(novel));

        let log = persist_novel.event_log()?;

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
        if let Some(events) = self.log.take_events() {
            self.data.commit_events(events);
        }

        self.persist_novel.write_data(&self.data)?;
        self.log.truncate()?;

        Ok(())
    }

    pub fn download(&mut self) -> anyhow::Result<()> {
        let chapter_dir = self.persist_novel.chapters_dir();
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
            &self.persist_novel,
            &self.data,
            &mut self.log,
            &chapters,
            self.persist_novel.dir(),
            &self.options,
        )?;

        Ok(())
    }

    fn download_chapters(
        runner: &mut Runner,
        persist_novel: &PersistNovel<'a>,
        data: &SavedNovel,
        log: &mut EventLog,
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
            let path = persist_novel.save_chapter(chapter, content)?;

            info!("Downloaded '{}' to '{}'.", &chapter.title, path.display());

            let path = persist_novel.relative_path(path);
            log.push_event(EventKind::Downloaded {
                url: chapter.url.clone(),
                path,
            })?;
        }

        Ok(())
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
        let path = self.persist_novel.cover_path(suffix);

        let mut file = BufWriter::new(File::create(&path)?);
        response.copy_to(&mut file)?;

        info!("Saved novel cover to '{}'.", path.display());
        data.cover = Some(CoverLoc { path, content_type });

        Ok(())
    }
}
