use std::path::PathBuf;

use url::Url;

use super::download::{DownloadHandler, DownloadOptions};

pub fn compile_epub(url: Url, wasm_path: PathBuf) -> anyhow::Result<()> {
    let mut handler = DownloadHandler::new(url, wasm_path, DownloadOptions::default())?;

    handler.save()?;
    handler.download()?;
    handler.save()?;

    Ok(())
}
