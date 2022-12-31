mod handler;
mod options;

use std::path::PathBuf;

pub use options::DownloadOptions;
use url::Url;

use crate::data::{GlobalTracker, NovelTracking};

use self::handler::DownloadHandler;

pub fn download(
    url: Url,
    wasm_path: PathBuf,
    global_file: PathBuf,
    options: DownloadOptions,
) -> anyhow::Result<NovelTracking> {
    let url_string = url.to_string();
    let mut handler = DownloadHandler::new(url, wasm_path, options)?;
    handler.save()?;

    {
        let mut global = GlobalTracker::open(global_file)?;
        global
            .data
            .insert_novel(url_string, handler.save_dir.clone());
        global.save()?;
    }

    handler.download()?;
    handler.save()?;

    Ok(handler.tracking)
}
