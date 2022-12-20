use std::{
    collections::HashMap,
    fs::File,
    io::{BufReader, BufWriter},
    path::PathBuf,
};

use serde::{Deserialize, Serialize};

#[derive(Debug)]
pub struct GlobalTracker {
    pub data: GlobalData,
    pub path: PathBuf,
}

#[derive(Serialize, Deserialize, Debug, Default)]
pub struct GlobalData {
    novels: HashMap<String, String>,
}

impl GlobalTracker {
    pub fn new(path: PathBuf) -> anyhow::Result<Self> {
        let data = if path.exists() {
            let file = BufReader::new(File::open(&path)?);
            serde_json::from_reader(file)?
        } else {
            Default::default()
        };

        Ok(GlobalTracker { data, path })
    }

    pub fn save(&self) -> anyhow::Result<()> {
        let mut file = BufWriter::new(File::create(&self.path)?);
        serde_json::to_writer(&mut file, &self.data)?;
        Ok(())
    }
}
