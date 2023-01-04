use std::{ops::RangeInclusive, path::PathBuf, time::Duration};

#[derive(Debug)]
pub struct DownloadOptions {
    pub dir: PathBuf,
    pub range: Option<RangeInclusive<usize>>,
    pub delay: Option<Duration>,
}

impl Default for DownloadOptions {
    fn default() -> Self {
        Self {
            dir: PathBuf::from("data"),
            range: None,
            delay: None,
        }
    }
}
