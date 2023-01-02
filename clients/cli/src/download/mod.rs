mod handler;
mod options;

use std::path::PathBuf;

pub use handler::{DATA_FILENAME, LOG_FILENAME};
pub use options::DownloadOptions;
use url::Url;

use crate::data::{GlobalTracker, NovelTracking};

use self::handler::DownloadHandler;

pub fn download(
    url: Url,
    wasm_path: PathBuf,
    options: DownloadOptions,
) -> anyhow::Result<NovelTracking> {
    let mut global = GlobalTracker::in_dir(&options.dir)?;

    let url_string = url.to_string();
    let mut handler = DownloadHandler::new(url, wasm_path, options)?;
    handler.save()?;

    if !handler.is_cover_downloaded() {
        handler.download_cover()?;
        handler.save()?;
    }

    global
        .data
        .insert_novel(url_string, handler.save_dir.clone());
    global.save()?;

    handler.download()?;
    handler.save()?;

    Ok(handler.tracking)
}
