use std::{collections::HashMap, fs::File, io::BufWriter, path::Path};

use epub_builder::{EpubBuilder, EpubContent, ReferenceType, ZipLibrary};
use indoc::formatdoc;
use itertools::Itertools;
use log::{info, warn};
use quelle_core::prelude::*;

use crate::data::Bundle;

pub fn bundle_epub<B: Bundle>(
    bundle: B,
    out: &mut BufWriter<File>,
) -> Result<(), Box<dyn std::error::Error>> {
    let meta = bundle.meta();
    let novel = bundle.novel();

    let mut builder = EpubBuilder::new(ZipLibrary::new()?)?;

    let preface_content = preface_content(meta, novel);
    let preface = EpubContent::new("preface.xhtml", preface_content.as_bytes())
        .title("Preface")
        .reftype(ReferenceType::Preface);

    if let (Some(path), Some(content_type)) = (bundle.cover_path(), bundle.cover_content_type()) {
        set_cover_image(&mut builder, path, content_type)?;
    }

    builder.set_title(&novel.title);
    for author in &novel.authors {
        builder.add_author(author);
    }

    for paragraph in &novel.description {
        builder.add_description(paragraph);
    }

    info!("Written title, authors, and description");

    for metadata in &novel.metadata {
        if ["title", "author", "subject", "language"].contains(&metadata.name.as_str()) {
            builder.metadata(&metadata.name, &metadata.value)?;
        }
    }

    builder.set_generator("quelle");
    builder.set_lang(novel.langs.iter().join(","));

    info!("Written metadata");

    builder.add_content(preface)?;

    info!("Written novel preface");

    for volume in &novel.volumes {
        for chapter in &volume.chapters {
            let file_name = format!("chapters/{}.xhtml", &chapter.index);

            let content = if let Some(content) = bundle.chapter_content(&chapter.url)? {
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

fn set_cover_image(
    builder: &mut EpubBuilder<ZipLibrary>,
    cover_path: &Path,
    content_type: &str,
) -> Result<(), Box<dyn std::error::Error>> {
    if cover_path.exists() {
        let file = File::open(cover_path)?;
        let name = cover_path
            .file_name()
            .map(|name| name.to_string_lossy().to_string())
            .unwrap_or_else(|| String::from("cover.unknwon"));

        builder.add_cover_image(name, file, content_type)?;
        info!("Written cover file '{}'", cover_path.display());
    } else {
        warn!("The cover file could not be found.");
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

pub fn preface_content(_meta: Option<&Meta>, novel: &Novel) -> String {
    let title = &novel.title;
    let url = &novel.url;

    let authors = if novel.authors.is_empty() {
        String::from("<p>Unknown author</p>")
    } else {
        format!("<p>{}</p>", novel.authors.join(", "))
    };

    let description = if novel.description.is_empty() {
        String::from("<p>No description provided</p>")
    } else {
        format!("<p>{}</p>", novel.description.join("</p><p>"))
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
