use quelle_core::prelude::Meta;
use std::{fs::File, io::BufReader, path::PathBuf};

use crate::{error::PersistResult, global::Global, novel::PersistNovel, PersistOptions};

#[derive(Debug)]
pub struct Persist {
    pub meta: Meta,
    pub options: PersistOptions,
}

impl Persist {
    pub fn new(meta: Meta, options: PersistOptions) -> Self {
        Persist { meta, options }
    }

    pub fn persist_novel<'a>(&'a self, dir: PathBuf) -> PersistNovel<'a> {
        PersistNovel::new(dir, self)
    }

    pub fn novel_path(&self, global: &Global, url: &str, title: &str) -> PathBuf {
        match global.novel_path_by_url(url) {
            Some(path) => path.clone(),
            None => {
                let mut path = self.options.novel.dir.join(&self.meta.id);
                path.push(slug::slugify(title));
                path
            }
        }
    }

    pub fn global(&self) -> PersistResult<Global> {
        let data = if self.options.global_path.exists() {
            let file = File::open(&self.options.global_path)?;
            let reader = BufReader::new(file);
            serde_json::from_reader(reader)?
        } else {
            Default::default()
        };

        Ok(data)
    }
}
