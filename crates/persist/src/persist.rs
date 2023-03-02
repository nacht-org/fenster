use quelle_core::prelude::Meta;
use std::{
    fs::{self, File, OpenOptions},
    io::{BufReader, BufWriter},
    path::PathBuf,
};

use crate::{error::PersistResult, global::Global, novel::PersistNovel, PersistOptions};

#[derive(Debug)]
pub struct Persist {
    pub options: PersistOptions,
}

impl Persist {
    pub fn new(options: PersistOptions) -> Self {
        Persist { options }
    }

    pub fn persist_novel<'a>(&'a self, dir: PathBuf) -> PersistNovel<'a> {
        PersistNovel::new(dir, self)
    }

    pub fn novel_path(&self, meta: &Meta, title: &str) -> PathBuf {
        let mut path = self.options.novel.dir.join(&meta.id);
        path.push(slug::slugify(title));
        path
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

    pub fn save_global(&self, global: &Global) -> PersistResult<()> {
        let path = self.options.global_path.as_path();

        if let Some(parent) = path.parent() {
            if !parent.exists() {
                fs::create_dir_all(parent)?;
            }
        }

        let file = OpenOptions::new()
            .create(true)
            .truncate(true)
            .write(true)
            .open(path)?;

        let writer = BufWriter::new(file);
        serde_json::to_writer(writer, &global)?;

        Ok(())
    }
}
