use std::{ops::RangeInclusive, path::PathBuf};

#[derive(Debug)]
pub struct DownloadOptions {
    pub dir: PathBuf,
    pub range: Option<RangeInclusive<usize>>,
}

impl Default for DownloadOptions {
    fn default() -> Self {
        Self {
            dir: PathBuf::from("data"),
            range: None,
        }
    }
}
