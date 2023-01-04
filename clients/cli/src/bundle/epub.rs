use std::{
    collections::HashMap,
    fs::{self, File},
    io::BufWriter,
    path::Path,
};

use epub_builder::{EpubBuilder, EpubContent, ReferenceType, ZipLibrary};
use fenster_core::prelude::{Chapter, Meta, Metadata, Novel};
use indoc::formatdoc;
use itertools::Itertools;
use log::{info, warn};

use crate::data::TrackingData;

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
        .reftype(ReferenceType::Preface);

    if let Some(path) = &data.cover_path {
        add_cover_image(&mut builder, path)?;
    }

    builder.metadata("title", &novel.title)?;
    for author in novel.authors {
        builder.metadata("author", author)?;
    }

    for paragraph in novel.desc {
        builder.metadata("description", paragraph)?;
    }

    info!("Written title, authors, and description");

    for metadata in novel.metadata {
        if metadata.name != "subject" {
            continue;
        }

        builder.metadata(metadata.name, metadata.value)?;
    }

    builder.metadata("generator", "fenster")?;
    builder.metadata("lang", novel.lang)?;

    info!("Written metadata");

    builder.add_content(preface)?;

    info!("Written novel preface");

    for volume in novel.volumes {
        for chapter in volume.chapters {
            let file_name = format!("chapters/{}.xhtml", &chapter.index,);

            let content = if let Some(file_path) = data.downloaded.get(&chapter.url) {
                let file_path = base_path.join(file_path);
                let content = fs::read_to_string(&file_path)?;

                info!("Read chapter content from '{}'.", file_path.display());
                prepare_content(&chapter, content)
            } else {
                warn!("Using placeholder content for '{}'.", file_name);
                empty_content(&chapter)
            };

            let content = EpubContent::new(&file_name, content.as_bytes()).title(&chapter.title);
            builder.add_content(content)?;

            info!("Written '{}' as '{}'.", chapter.title, file_name);
        }
    }

    builder.generate(out)?;

    info!("Epub writing complete.");

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

fn add_cover_image(
    builder: &mut EpubBuilder<ZipLibrary>,
    cover_path: &Path,
) -> Result<(), Box<dyn std::error::Error>> {
    let guess = mime_guess::from_path(&cover_path);
    if let Some(mime) = guess.first() {
        if cover_path.exists() {
            let file = File::open(&cover_path)?;
            let name = cover_path
                .file_name()
                .map(|name| name.to_string_lossy().to_string())
                .unwrap_or_else(|| {
                    let suffix = mime.suffix().map(|s| s.as_str()).unwrap_or_default();
                    format!("cover.{suffix}")
                });

            builder.add_cover_image(name, file, mime.essence_str())?;
            info!("Written cover file '{}'", cover_path.display());
        } else {
            warn!("The cover file could not be found.");
        }
    }

    Ok(())
}

fn capitalize(s: &str) -> String {
    let mut c = s.chars();
    match c.next() {
        None => String::new(),
        Some(f) => f.to_uppercase().collect::<String>() + c.as_str(),
    }
}

pub fn preface_content(meta: &Option<Meta>, novel: &Novel) -> String {
    let title = &novel.title;
    let url = &novel.url;

    let authors = if novel.authors.is_empty() {
        String::from("<p>Unknown Author</p>")
    } else {
        format!("<p>{}</p>", novel.authors.join(", "))
    };

    let description = if novel.desc.is_empty() {
        String::from("<p>No description provided</p>")
    } else {
        format!("<p>{}</p>", novel.desc.join("</p><p>"))
    };

    let metadata = {
        let mut metadata_by_tag = HashMap::<String, Vec<&Metadata>>::new();
        for metadata in &novel.metadata {
            metadata_by_tag
                .entry(metadata.name.clone())
                .or_insert(vec![])
                .push(metadata);
        }

        let metadata = metadata_by_tag
            .into_iter()
            .map(|(name, values)| {
                format!(
                    "<div><h2>{}</h2><p>{}</p></div>",
                    capitalize(&name),
                    values.into_iter().map(|v| &v.value).join(", ")
                )
            })
            .join("");

        metadata
    };

    formatdoc! {r#"
        <h1>
            <a href="{url}">{title}</a>
        </h1>
        <div>
            <h2>Authors</h2>
            {authors}
        </div>
        <div>
            <h2>Description</h2>
            {description}
        </div>
        {metadata}
    "#}
}
