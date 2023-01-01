mod args;
mod bundle;
mod data;
mod download;
mod lock;

use std::{fs::File, io::BufReader, path::PathBuf, process::exit};

use anyhow::Context;
use args::download_range::DownloadRange;
use clap::{Parser, Subcommand};
use data::{GlobalTracker, NovelTracking};
use download::DownloadOptions;
use lock::Lock;
use simplelog::{Config, LevelFilter, TermLogger};
use url::Url;

#[derive(Parser)]
#[command(author, version, about, long_about = None)]
struct Cli {
    /// Provide additional information (default only shows errors).
    #[clap(short, long, action = clap::ArgAction::Count)]
    verbose: u8,

    #[clap(short, long, default_value = "dist/extension-lock.json")]
    lock_file: PathBuf,

    #[clap(short, long, default_value = "dist/data")]
    data_dir: PathBuf,

    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    Lock {
        /// The directory to find wasm extensions
        #[arg(short, long, default_value = "dist")]
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
    },

    Bundle {
        url: Url,
    },
}

fn main() -> anyhow::Result<()> {
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

    run(cli)
}

fn run(cli: Cli) -> anyhow::Result<()> {
    match cli.command {
        Commands::Detect { url } => {
            let file = File::open(cli.lock_file)?;
            let lock: Lock = serde_json::from_reader(BufReader::new(file))?;

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
            let lock = Lock::generate(&dir)?;
            lock.save(&cli.lock_file)?;
        }
        Commands::Download { url, range } => {
            let lock = Lock::open(&cli.lock_file)?;
            let Some(extension) = lock.detect(url.to_string())? else {
                println!("supported source not found.");
                exit(1);
            };

            let options = DownloadOptions {
                dir: cli.data_dir,
                range: range.map(|r| r.0),
            };

            download::download(url, PathBuf::from(&extension.path), options)?;
        }
        Commands::Bundle { url } => {
            let global = GlobalTracker::in_dir(&cli.data_dir)?;
            let path = global
                .data
                .get_path_for_url(&url.to_string())
                .with_context(|| "The novel does not exist")?;

            let tracking = NovelTracking::open(path.join(download::DATA_FILENAME))?;
            println!("{tracking:#?}");
        }
    }

    Ok(())
}
