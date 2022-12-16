use std::path::PathBuf;

use url::Url;

use super::download::DownloadHandler;

pub fn compile_epub(url: Url, wasm_path: PathBuf) -> anyhow::Result<()> {
    let mut handler = DownloadHandler::new(url, wasm_path)?;

    handler.save()?;
    handler.download()?;
    handler.save()?;

    Ok(())
}
