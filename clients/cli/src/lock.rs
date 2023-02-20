use std::{
    collections::HashMap,
    ffi::OsStr,
    fs::{self, File},
    io::BufReader,
    path::Path,
};

use anyhow::{anyhow, bail, Context};
use quelle_engine::Runner;
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

impl Lock {
    pub fn open(path: &Path) -> anyhow::Result<Self> {
        let file = File::open(path).with_context(|| "failed to open lock file")?;
        let reader = BufReader::new(file);
        serde_json::from_reader(reader).with_context(|| "failed to parse lock file")
    }

    pub fn detect(&self, url: &str) -> anyhow::Result<Option<&Extension>> {
        for (_, extension) in &self.extensions {
            for base_url in &extension.base_urls {
                if url.starts_with(base_url) {
                    return Ok(Some(extension));
                }
            }
        }

        Ok(None)
    }

    pub fn generate(extensions_dir: &Path) -> anyhow::Result<Self> {
        let mut extensions = HashMap::new();

        for entry in fs::read_dir(extensions_dir)? {
            let entry = entry?;
            let path = entry.path();

            if path.extension() != Some(OsStr::new("wasm")) {
                debug!("skipped non-wasm file '{}'", path.display());
                continue;
            }

            info!("Reading meta info from '{}'...", path.display());
            let mut runner = Runner::new(&path).map_err(|e| anyhow!(e.to_string()))?;
            let meta = runner.meta().map_err(|e| anyhow!(e.to_string()))?;

            if let Some(Extension { name, .. }) = extensions.get(&meta.id) {
                bail!("Both '{}' and '{}' have the same id", name, &meta.name);
            }

            info!("Found {}=={}", meta.id, meta.version);

            let extension = Extension {
                name: meta.name,
                version: meta.version,
                base_urls: meta.base_urls,
                langs: meta.langs,
                path: entry.path().as_os_str().to_string_lossy().to_string(),
            };

            extensions.insert(meta.id, extension);
        }

        let lock = Lock {
            version: 1,
            extensions,
        };

        info!("generated lock file at 'dist/lock.json'");
        Ok(lock)
    }

    pub fn save(&self, path: &Path) -> anyhow::Result<()> {
        let mut file = File::options()
            .create(true)
            .write(true)
            .truncate(true)
            .open(path)?;

        serde_json::to_writer_pretty(&mut file, self)?;
        Ok(())
    }
}
