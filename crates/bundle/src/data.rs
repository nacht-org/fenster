use std::{
    collections::HashMap,
    fs,
    path::{Path, PathBuf},
};

use log::info;
use quelle_core::prelude::*;
use quelle_persist::CoverLoc;

/// A trait that provides necessary information for bundlers
pub trait Bundle {
    /// The source meta information
    fn meta(&self) -> Option<&Meta>;

    /// The novel being bundled
    fn novel(&self) -> &Novel;

    /// The path to the cover or thumbnail
    fn cover_path(&self) -> Option<&Path>;

    /// The content type of the cover or thumbnail
    fn cover_content_type(&self) -> Option<&str>;

    /// Return chapter content when the url of the chapter is provided
    fn chapter_content(&self, url: &str) -> Result<Option<String>, Box<dyn std::error::Error>>;
}

///
#[cfg(feature = "persist")]
pub struct PersistBundle {
    pub meta: Option<Meta>,
    pub novel: Novel,
    pub cover: Option<CoverLoc>,
    pub base_path: PathBuf,
    pub chapter_content: HashMap<String, PathBuf>,
}

#[cfg(feature = "persist")]
impl Bundle for PersistBundle {
    fn meta(&self) -> Option<&Meta> {
        self.meta.as_ref()
    }

    fn novel(&self) -> &Novel {
        &self.novel
    }

    fn cover_path(&self) -> Option<&Path> {
        self.cover.as_ref().map(|cover| cover.path.as_path())
    }

    fn cover_content_type(&self) -> Option<&str> {
        self.cover.as_ref().map(|cover| cover.content_type.as_str())
    }

    fn chapter_content(&self, url: &str) -> Result<Option<String>, Box<dyn std::error::Error>> {
        let Some(file_path) = self.chapter_content.get(url) else { return Ok(None) };
        let file_path = self.base_path.join(file_path);
        let content = fs::read_to_string(&file_path)?;
        info!("Read chapter content from '{}'.", file_path.display());
        Ok(Some(content))
    }
}
