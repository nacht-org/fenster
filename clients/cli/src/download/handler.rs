use std::{
    fs::{self, File},
    io::BufWriter,
    mem,
    path::{Path, PathBuf},
};

use fenster_core::prelude::{Chapter, Meta, Novel};
use fenster_engine::Runner;
use log::info;
use url::Url;

use crate::data::{DownloadLog, EventKind, NovelTracking};

use super::DownloadOptions;

pub struct DownloadHandler {
    pub runner: Runner,
    pub meta: Meta,
    pub save_dir: PathBuf,
    pub log: DownloadLog,
    pub tracking: NovelTracking,
    pub options: DownloadOptions,
}

pub const DATA_FILENAME: &'static str = "data.json";
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

impl DownloadHandler {
    pub fn new(url: Url, wasm_path: PathBuf, options: DownloadOptions) -> anyhow::Result<Self> {
        let mut runner = Runner::new(&wasm_path)?;

        let novel = runner.fetch_novel(url.as_str())?;
        let meta = runner.meta()?;

        let save_dir = get_novel_dir(&options.dir, &meta, &novel);
        if !save_dir.exists() {
            fs::create_dir_all(&save_dir)?;
        }

        let tracking_path = save_dir.join(DATA_FILENAME);
        let tracking = NovelTracking::new(novel, tracking_path)?;

        let log_path = save_dir.join(LOG_FILENAME);
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
        let chapter_dir = get_chapters_dir(&self.save_dir);
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

        let chapters = match self.options.range.as_ref() {
            Some(range) => &chapters[range.clone()],
            None => &chapters,
        };

        Self::download_chapters(
            &mut self.runner,
            &self.tracking,
            &mut self.log,
            &chapter_dir,
            &chapters,
            &self.save_dir,
        )?;

        Ok(())
    }

    fn download_chapters(
        runner: &mut Runner,
        tracking: &NovelTracking,
        log: &mut DownloadLog,
        chapter_dir: &Path,
        chapters: &[&Chapter],
        save_dir: &Path,
    ) -> anyhow::Result<()> {
        for chapter in chapters {
            if let Some(path) = tracking.data.downloaded.get(&chapter.url) {
                if save_dir.join(path).exists() {
                    continue;
                }
            }

            let content = runner.fetch_chapter_content(&chapter.url)?;
            let Some(content) = content else { continue };

            let filename = format!("{}.html", chapter.index);
            let path = chapter_dir.join(&filename);
            fs::write(&path, content)?;

            info!(
                "Chapter '{}' saved to '{}'.",
                &chapter.title,
                path.display()
            );

            log.push_event(EventKind::Downloaded {
                url: chapter.url.clone(),
                path: Path::new("chapters").join(&filename),
            })?;
        }

        Ok(())
    }

    pub fn is_cover_downloaded(&self) -> bool {
        let cover_path = &self.tracking.data.cover_path;
        let Some(cover_path) = cover_path else { return false };
        return cover_path.exists() && cover_path.is_file();
    }

    pub fn download_cover(&mut self) -> anyhow::Result<()> {
        let data = &mut self.tracking.data;
        println!("{:?}", data.novel.thumb);
        let Some(url) = data.novel.thumb.as_ref() else { return Ok(()) };

        let suffix = get_file_suffix_from_url(url)?;
        println!("{suffix}");

        let mut resp = reqwest::blocking::get(url)?;
        let cover_path = data
            .cover_path
            .clone()
            .unwrap_or_else(|| self.save_dir.join(format!("cover{suffix}")));

        let mut file = BufWriter::new(File::create(&cover_path)?);
        resp.copy_to(&mut file)?;

        info!("Downloaded novel cover to '{}'.", cover_path.display());
        data.cover_path = Some(cover_path);
        Ok(())
    }
}

/// Extract the suffix from a url that point to a file
fn get_file_suffix_from_url(url: &str) -> Result<String, url::ParseError> {
    let parsed_url = Url::parse(url)?;
    let suffix = Path::new(parsed_url.path())
        .extension()
        .unwrap_or_default()
        .to_string_lossy();

    let suffix = if suffix.is_empty() {
        String::new()
    } else {
        format!(".{suffix}")
    };

    Ok(suffix)
}

#[cfg(test)]
mod tests {
    use super::get_file_suffix_from_url;

    #[test]
    fn test_get_file_suffix_from_url() {
        assert_eq!(
            get_file_suffix_from_url("https://website.com/image.jpg"),
            Ok(String::from(".jpg"))
        );
        assert_eq!(
            get_file_suffix_from_url("https://website.com/image.jpg?w=400&h=300"),
            Ok(String::from(".jpg"))
        );
        assert_eq!(
            get_file_suffix_from_url("https://website.com/non/image"),
            Ok(String::from(""))
        );
    }
}
