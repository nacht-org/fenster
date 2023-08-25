mod args;
mod bundle;
mod download;
mod lock;

use std::{
    fs::File,
    io::BufWriter,
    path::{Path, PathBuf},
    process::exit,
    time::Duration,
};

use anyhow::{anyhow, bail};
use args::{CoverAction, DownloadRange};
use clap::{Parser, Subcommand};
use download::DownloadOptions;
use lock::Lock;
use log::{info, warn};
use quelle_engine::Runtime;
use quelle_persist::{create_parent_all, Persist, PersistOptions};
use simplelog::{Config, LevelFilter, TermLogger};
use url::Url;

#[derive(Parser)]
#[command(author, version, about, long_about = None)]
struct Cli {
    /// Provide additional information (default only shows errors).
    #[clap(short, long, action = clap::ArgAction::Count)]
    verbose: u8,

    #[clap(short, long, default_value = "extension-lock.json")]
    lock_file: PathBuf,

    #[clap(short, long, default_value = "data")]
    data_dir: PathBuf,

    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    Lock {
        /// The directory to find wasm extensions
        #[arg(short, long, default_value = "extensions")]
        dir: PathBuf,
    },

    Detect {
        url: Url,
    },

    Download {
        /// The url to the novel
        url: Url,

        /// The range of chapters to download
        #[arg(short, long)]
        range: Option<DownloadRange>,

        /// Delay between each chapter download in milliseconds
        #[arg(short, long)]
        delay: Option<u32>,

        /// How the novel cover download should be handled
        #[arg(short, long, default_value = "dynamic")]
        cover: CoverAction,
    },

    Popular {
        /// The url of the source website
        url: Url,

        /// The page to browse
        #[arg(short, long, default_value = "1")]
        page: i32,
    },

    Bundle {
        url: Url,
    },
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let cli = Cli::parse();

    let level = match cli.verbose {
        0 => LevelFilter::Error,
        1 => LevelFilter::Warn,
        2 => LevelFilter::Info,
        3 => LevelFilter::Debug,
        _ => LevelFilter::Trace,
    };

    TermLogger::init(
        level,
        Config::default(),
        simplelog::TerminalMode::Mixed,
        simplelog::ColorChoice::Auto,
    )
    .unwrap();

    run(cli).await
}

async fn run(cli: Cli) -> anyhow::Result<()> {
    match cli.command {
        Commands::Detect { url } => {
            let lock = Lock::open(&cli.lock_file)?;

            let extension = lock
                .extensions
                .into_iter()
                .map(|(_, e)| e)
                .find(|e| e.base_urls.iter().any(|bu| url.as_str().starts_with(bu)));

            match extension {
                Some(extension) => println!("{extension:#?}"),
                None => println!("No source matching '{url}' found"),
            }
        }
        Commands::Lock { dir } => {
            let lock = Lock::generate(&dir).await?;
            lock.save(&cli.lock_file)?;
            info!("Saved lock file to '{}'", cli.lock_file.display());
        }
        Commands::Download {
            url,
            range,
            delay,
            cover,
        } => {
            let persist = Persist::new(PersistOptions::default());

            let lock = Lock::open(&cli.lock_file)?;
            let Some(extension) = lock.detect(url.as_str())? else {
                println!("supported source not found.");
                exit(1);
            };

            let options = DownloadOptions {
                dir: cli.data_dir,
                range: range.map(|r| r.0),
                delay: delay.map(|v| Duration::from_millis(v as u64)),
                cover,
            };

            download::download(persist, url, PathBuf::from(&extension.path), options).await?;
        }
        Commands::Popular { url, page } => {
            let lock = Lock::open(&cli.lock_file)?;
            let Some(extension) = lock.detect(url.as_str())? else {
                println!("supported source not found.");
                exit(1);
            };

            let mut runner = Runtime::new(Path::new(&extension.path)).await?;
            let meta = runner.meta().await?;

            if !runner.popular_supported() {
                log::error!("'{}' does not support popular browse", meta.name);
                exit(1);
            }

            log::info!("fetching popular from '{}'", meta.name);
            let novels = runner.popular(page).await?;
            if novels.is_empty() {
                log::error!("No novels found");
            }

            for novel in novels {
                println!("{} <{}>", novel.title, novel.url);
            }
        }
        Commands::Bundle { url } => {
            let persist = Persist::new(PersistOptions::default());
            let global = persist.read_global()?;
            info!("Loaded global data");

            let path = global
                .novel_path_from_url(&url.to_string())
                .ok_or(anyhow!("The novel does not exist"))?;

            info!("Found novel data at '{}'.", path.display());

            let lock = Lock::open(&cli.lock_file)?;
            let meta = if let Some(ext) = lock.detect(url.as_str())? {
                let path = Path::new(&ext.path);
                if !path.exists() {
                    bail!("The wasm extension file could not be found");
                }

                let mut runner = Runtime::new(path).await?;
                let meta = runner.meta().await?;
                info!("Acquired source meta information from wasm file.");

                Some(meta)
            } else {
                warn!("failed to retrieve meta information for the url");
                None
            };

            let novel = persist.persist_novel(path.into());
            let data = novel.read_data()?.ok_or(anyhow!("novel data not found"))?;

            info!("Loaded novel information from disk");

            let output_path =
                path.join(format!("output/{}.epub", slug::slugify(&data.novel.title)));
            create_parent_all(&output_path)?;

            let mut file = BufWriter::new(File::create(&output_path)?);

            info!("Writing to '{}'", &output_path.display());

            bundle::compile_epub(meta, data, path.to_path_buf(), &mut file)
                .map_err(|e| anyhow!("failed to bundle epub: {}", e.to_string()))?;
        }
    }

    Ok(())
}
