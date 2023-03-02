mod event;
mod handler;
mod options;

use std::path::PathBuf;

pub use handler::LOG_FILENAME;
use log::warn;
pub use options::DownloadOptions;
use quelle_persist::{Persist, SavedNovel};
use url::Url;

use crate::args::CoverAction;

use self::handler::DownloadHandler;

pub fn download(
    persist: Persist,
    url: Url,
    wasm_path: PathBuf,
    options: DownloadOptions,
) -> anyhow::Result<SavedNovel> {
    let mut global = persist.read_global()?;

    let url_string = url.to_string();
    let mut handler = DownloadHandler::new(&persist, url, wasm_path, options)?;
    handler.save()?;

    match &handler.options.cover {
        CoverAction::Dynamic => {
            if !handler.is_cover_downloaded() {
                download_cover_and_warn(&mut handler)?;
            }
        }
        CoverAction::Force => download_cover_and_warn(&mut handler)?,
        CoverAction::Ignore => (),
    }

    global.insert_novel(url_string, handler.persist_novel.dir().to_path_buf());
    persist.save_global(&global)?;

    handler.download()?;
    handler.save()?;

    Ok(handler.data)
}

fn download_cover_and_warn(handler: &mut DownloadHandler) -> Result<(), anyhow::Error> {
    match handler.download_cover() {
        Ok(_) => handler.save(),
        Err(error) => {
            warn!("{error}");
            Ok(())
        }
    }
}
