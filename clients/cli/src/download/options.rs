use std::{ops::RangeInclusive, path::PathBuf, time::Duration};

use crate::args::CoverAction;

#[derive(Debug)]
pub struct DownloadOptions {
    pub dir: PathBuf,
    pub range: Option<RangeInclusive<usize>>,
    pub delay: Option<Duration>,
    pub cover: CoverAction,
}

impl Default for DownloadOptions {
    fn default() -> Self {
        Self {
            dir: PathBuf::from("data"),
            range: Default::default(),
            delay: Default::default(),
            cover: Default::default(),
        }
    }
}
