use crate::{error::PersistResult, global::Global, novel::PersistNovel, PersistOptions};
use quelle_core::prelude::Meta;
use std::path::PathBuf;

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

    pub fn read_global(&self) -> PersistResult<Global> {
        Global::open(&self.options.global_path)
    }

    pub fn save_global(&self, global: &Global) -> PersistResult<()> {
        global.save(&self.options.global_path)
    }
}
