use log::LevelFilter;
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Debug)]
pub struct ExtensionConfig {
    pub level_filter: LevelFilter,
}

impl Default for ExtensionConfig {
    fn default() -> Self {
        Self {
            level_filter: LevelFilter::Error,
        }
    }
}
