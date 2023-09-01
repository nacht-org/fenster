use std::{
    collections::HashMap,
    fs::{File, OpenOptions},
    io::{BufReader, BufWriter},
    path::{Path, PathBuf},
};

use serde::{Deserialize, Serialize};

use crate::{create_parent_all, error::PersistResult};

#[derive(Serialize, Deserialize, Debug, Default)]
pub struct Global {
    novels: HashMap<String, PathBuf>,
}

impl Global {
    pub fn open(path: &Path) -> PersistResult<Self> {
        let data = if path.exists() {
            let file = File::open(path)?;
            let reader = BufReader::new(file);
            serde_json::from_reader(reader)?
        } else {
            Default::default()
        };

        Ok(data)
    }

    pub fn save(&self, path: &Path) -> PersistResult<()> {
        create_parent_all(path)?;

        let file = OpenOptions::new()
            .create(true)
            .truncate(true)
            .write(true)
            .open(path)?;

        let writer = BufWriter::new(file);
        serde_json::to_writer(writer, self)?;

        Ok(())
    }

    pub fn novel_path_from_url(&self, url: &str) -> Option<&Path> {
        if let Some(value) = self.novels.get(url).map(AsRef::as_ref) {
            return Some(value);
        }

        if let Some(url) = url.strip_suffix("/") {
            if let Some(value) = self.novels.get(url).map(AsRef::as_ref) {
                return Some(value);
            }
        }

        None
    }

    pub fn insert_novel(&mut self, url: String, path: PathBuf) {
        self.novels.insert(url, path);
    }
}

#[cfg(test)]
mod tests {
    use std::path::{Path, PathBuf};

    use super::Global;

    #[test]
    fn should_return_path_with_or_without_trailing_slash() {
        let mut global = Global::default();
        global.insert_novel(
            String::from("https://example.com/novel/123"),
            PathBuf::from("/novels/123"),
        );

        assert_eq!(
            global.novel_path_from_url("https://example.com/novel/123"),
            Some(Path::new("/novels/123"))
        );
        assert_eq!(
            global.novel_path_from_url("https://example.com/novel/123/"),
            Some(Path::new("/novels/123"))
        );
    }
}
