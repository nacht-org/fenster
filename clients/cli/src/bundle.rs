use std::{fs::File, io::BufWriter, path::PathBuf};

use quelle_bundle::PersistBundle;
use quelle_core::prelude::*;
use quelle_persist::SavedNovel;

pub fn compile_epub(
    meta: Option<Meta>,
    data: SavedNovel,
    base_path: PathBuf,
    out: &mut BufWriter<File>,
) -> Result<(), Box<dyn std::error::Error>> {
    let bundle = PersistBundle {
        meta,
        novel: data.novel,
        cover: data.cover.map(Into::into),
        base_path,
        chapter_content: data.downloaded,
    };

    quelle_bundle::epub::bundle_epub(bundle, out)
}
