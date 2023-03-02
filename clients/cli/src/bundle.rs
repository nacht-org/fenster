use std::{fs::File, io::BufWriter, path::Path};

use quelle_bundle::Bundle;
use quelle_core::prelude::*;
use quelle_persist::SavedNovel;

pub fn compile_epub(
    meta: Option<Meta>,
    data: SavedNovel,
    base_path: &Path,
    out: &mut BufWriter<File>,
) -> Result<(), Box<dyn std::error::Error>> {
    let bundle = Bundle {
        meta,
        novel: data.novel,
        cover: data.cover.map(Into::into),
        chapter_content: data.downloaded,
    };

    quelle_bundle::epub::bundle_epub(bundle, base_path, out)
}
