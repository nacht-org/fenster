use std::path::PathBuf;

#[derive(Debug)]
pub struct PersistOptions {
    pub base_dir: PathBuf,
    pub global_path: PathBuf,
    pub novel: NovelOptions,
}

#[derive(Debug)]
pub struct NovelOptions {
    pub dir: PathBuf,
    pub filename: PathBuf,
}

impl NovelOptions {
    pub fn file_path(&self) -> PathBuf {
        self.dir.join(&self.filename)
    }
}
