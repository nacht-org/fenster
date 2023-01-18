use std::{
    collections::HashMap,
    ffi::OsStr,
    fs::{self, File},
    path::PathBuf,
};

use anyhow::{anyhow, bail};
use fenster_engine::Runner;
use log::{debug, info};
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Debug)]
pub struct Lock {
    pub version: usize,
    pub extensions: HashMap<String, Extension>,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct Extension {
    pub name: String,
    pub version: String,
    pub base_urls: Vec<String>,
    pub langs: Vec<String>,
    pub path: String,
}

pub fn lock(dir: PathBuf) -> anyhow::Result<()> {
    let mut extensions = HashMap::new();

    for entry in fs::read_dir(&dir)? {
        let entry = entry?;
        let path = entry.path();

        if path.extension() != Some(OsStr::new("wasm")) {
            debug!("skipped non-wasm file '{}'", path.display());
            continue;
        }

        info!("collecting meta info from '{}'...", path.display());
        let mut runner = Runner::new(&path).map_err(|e| anyhow!(e.to_string()))?;
        let meta = runner.meta().map_err(|e| anyhow!(e.to_string()))?;

        if let Some(Extension { name, .. }) = extensions.get(&meta.id) {
            bail!("both '{}' and '{}' have the same id", name, &meta.name);
        }

        let extension = Extension {
            name: meta.name,
            version: meta.version,
            base_urls: meta.base_urls,
            langs: meta.langs.clone(),
            path: entry.path().as_os_str().to_string_lossy().to_string(),
        };

        extensions.insert(meta.id, extension);
    }

    let lock = Lock {
        version: 1,
        extensions,
    };

    {
        let mut file = File::options()
            .create(true)
            .write(true)
            .truncate(true)
            .open("extension-lock.json")?;

        serde_json::to_writer_pretty(&mut file, &lock)?;
    }

    info!("generated lock file at 'extension-lock.json'");

    Ok(())
}
