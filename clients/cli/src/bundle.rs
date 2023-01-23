use std::{fs::File, io::BufWriter, path::Path};

use fenster_bundle::epub::Bundle;
use fenster_core::prelude::*;

use crate::data::TrackingData;

pub fn compile_epub(
    meta: Option<Meta>,
    data: TrackingData,
    base_path: &Path,
    out: &mut BufWriter<File>,
) -> Result<(), Box<dyn std::error::Error>> {
    let bundle = Bundle {
        meta,
        novel: data.novel,
        cover: data.cover,
        chapter_content: data.downloaded,
    };

    fenster_bundle::epub::bundle_epub(bundle, base_path, out)
}
