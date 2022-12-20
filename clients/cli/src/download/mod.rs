mod handler;
mod options;

use std::path::PathBuf;

pub use options::DownloadOptions;
use url::Url;

use crate::data::NovelTracking;

use self::handler::DownloadHandler;

pub fn download(
    url: Url,
    wasm_path: PathBuf,
    options: DownloadOptions,
) -> anyhow::Result<NovelTracking> {
    let mut handler = DownloadHandler::new(url, wasm_path, options)?;

    handler.save()?;
    handler.download()?;
    handler.save()?;

    Ok(handler.tracking)
}
