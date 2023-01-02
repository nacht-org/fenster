use std::{
    fs::{self, File},
    io::BufWriter,
    path::Path,
};

use epub_builder::{EpubBuilder, EpubContent, ReferenceType, ZipLibrary};
use fenster_core::prelude::{Chapter, Meta, Novel};
use indoc::formatdoc;

use crate::data::{NovelTracking, TrackingData};

pub fn compile_epub(
    meta: Option<Meta>,
    data: TrackingData,
    base_path: &Path,
    out: &mut BufWriter<File>,
) -> Result<(), Box<dyn std::error::Error>> {
    let novel = data.novel;
    let mut builder = EpubBuilder::new(ZipLibrary::new()?)?;

    let preface_content = preface_content(&meta, &novel);
    let preface = EpubContent::new("preface.xhtml", preface_content.as_bytes())
        .title("Preface")
        .reftype(ReferenceType::Text);

    builder.metadata("title", &novel.title)?;
    for author in novel.authors {
        builder.metadata("author", author)?;
    }

    if !novel.desc.is_empty() {
        builder.metadata("description", novel.desc.join("\n"))?;
    }

    for metadata in novel.metadata {
        builder.metadata(metadata.name, metadata.value)?;
    }

    builder.add_content(preface)?;

    for volume in novel.volumes {
        for chapter in volume.chapters {
            let file_name = format!("chapters/{}.xhtml", slug::slugify(&chapter.title));

            let content = if let Some(file_path) = data.downloaded.get(&chapter.url) {
                let file_path = base_path.join(file_path);
                let content = fs::read_to_string(file_path)?;
                prepare_content(&chapter, content)
            } else {
                empty_content(&chapter)
            };

            let content = EpubContent::new(&file_name, content.as_bytes()).title(chapter.title);
            builder.add_content(content)?;
        }
    }

    builder.generate(out)?;

    Ok(())
}

pub fn prepare_content(chapter: &Chapter, content: String) -> String {
    let title = &chapter.title;
    format!("<h1>{title}</h1>{content}")
}

pub fn empty_content(chapter: &Chapter) -> String {
    let title = &chapter.title;

    formatdoc! {r#"
        <h1>{title}</h1>
        <p>No downloaded content</p>
    "#}
}

pub fn preface_content(meta: &Option<Meta>, novel: &Novel) -> String {
    let title = &novel.title;

    let description = if novel.desc.is_empty() {
        String::from("<p>No description provided</p>")
    } else {
        format!("<p>{}</p>", novel.desc.join("</p><p>"))
    };

    formatdoc! {r#"
        <h1>{title}</h1>
        
        <div>
            <h2>Description</h2>
            <div>{description}</div>
        </div>
    "#}
}
