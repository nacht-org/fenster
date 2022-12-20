mod args;
mod bundle;
mod data;
mod download;
mod lock;

use std::{fs::File, io::BufReader, path::PathBuf};

use args::download_range::DownloadRange;
use clap::{Parser, Subcommand};
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

        /// The path to the lock file
        #[arg(short, long, default_value = "dist/lock.json")]
        lock: PathBuf,
    },

    Download {
        /// The url to the novel
        url: Url,

        /// The path to the source wasm
        #[arg(short, long)]
        wasm: PathBuf,

        /// The range of chapters to download
        #[arg(short, long)]
        range: Option<DownloadRange>,
    },
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
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

    match cli.command {
        Commands::Detect { url, lock } => {
            let file = File::open(lock)?;
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
            lock::lock(dir)?;
        }
        Commands::Download { url, wasm, range } => {
            let options = DownloadOptions {
                range: range.map(|r| r.0),
                ..Default::default()
            };

            download::download(url, wasm, options)?;
        }
    }

    Ok(())
}
