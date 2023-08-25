mod build;
mod cache;

use std::path::PathBuf;

use cache::{Cache, CachingImpl};
use clap::{Parser, Subcommand};
use quelle_core::prelude::{ExtensionConfig, Request};
use quelle_engine::Runtime;
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
    /// Run a given wasm extension
    Run {
        /// The path to the wasm file to be ran
        path: PathBuf,

        /// Print the meta information of the source
        #[arg(short, long)]
        meta: bool,

        /// Fetch and print the novel information
        #[arg(short, long)]
        novel: Option<Url>,

        /// Fetch and print the chapter content
        #[arg(short, long)]
        content: Option<Url>,

        /// Fetch and print popular novels
        #[arg(short, long)]
        popular: bool,

        /// A text query to use to search
        #[arg(short, long)]
        search: Option<String>,

        #[arg(short, long)]
        options: bool,

        /// Page used in search and popular
        #[arg(short, long, default_value = "1")]
        page: i32,
    },

    /// Build the extensions into wasm
    Build {
        /// Build this extension only.
        #[arg(short, long)]
        extension: Option<PathBuf>,

        /// The output directory for the built extensions
        #[arg(short, long, default_value = "extensions")]
        out: PathBuf,

        /// Build the extension(s) with release profile
        #[arg(short, long)]
        release: bool,
    },

    /// Read the compiled wasm files and create a record
    Lock {
        /// The directory to find wasm extensions
        #[arg(short, long, default_value = ".")]
        dir: PathBuf,
    },

    /// Check if a given url belongs to a source
    Detect {
        /// The url of a source to check
        url: Url,

        /// The path to the lock file
        #[arg(short, long, default_value = "extension-lock.json")]
        lock: PathBuf,
    },

    /// Functionality related to cache
    Cache {
        /// Download and cache the response
        #[arg(short, long)]
        url: Option<String>,

        /// Remove all items from the cache
        #[arg(short, long)]
        clear: bool,
    },
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
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
        Commands::Run {
            path,
            meta,
            novel,
            content,
            popular,
            search,
            options,
            page,
        } => {
            let config = ExtensionConfig {
                level_filter: level,
            };

            let mut runner = Runtime::builder()
                .send_request(cache::send_request)
                .build(&path, CachingImpl::new())
                .await?;

            runner.setup(&config).await?;

            if meta {
                let meta = runner.meta().await?;
                println!("{meta:#?}");
            }

            if let Some(url) = novel {
                let novel = runner.fetch_novel(url.as_str()).await?;
                println!("{novel:#?}");
            }

            if let Some(url) = content {
                let content = runner.fetch_chapter_content(url.as_str()).await?;
                println!("{content:#?}");
            }

            if let Some(query) = search {
                if runner.text_search_supported() {
                    let result = runner.text_search(&query, page).await?;
                    for item in result {
                        println!("{item:?}");
                    }
                } else {
                    println!("query search not supported");
                }
            }

            if popular {
                if runner.popular_supported() {
                    let url = runner.popular_url(page).await?;
                    println!("{url}");

                    let result = runner.popular(page).await?;
                    for item in result {
                        println!("{item:?}");
                    }
                } else {
                    println!("popular not supported");
                }
            }

            if options {
                if runner.filter_search_supported() {
                    let result = runner.filter_options().await?;
                    println!("{result:#?}");
                } else {
                    println!("Filter search not supported");
                }
            }
        }
        Commands::Detect { url, lock } => {
            let lock = quelle_lock::Lock::open(&lock)?;

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
        Commands::Build {
            extension,
            out,
            release,
        } => {
            build::build(extension, out, release)?;
        }
        Commands::Lock { dir } => {
            quelle_lock::Lock::generate(&dir).await?;
        }
        Commands::Cache { url, clear } => {
            if let Some(url) = url {
                let data = CachingImpl::new();

                let key = url.clone();
                let client = &data.client;

                let request = Request::new(quelle_core::prelude::Method::Get, url);

                use quelle_engine::module::http::{parse_response, send_request_reqwest};
                let response = send_request_reqwest::<CachingImpl>(client, request).await;
                let response = parse_response(response).await;

                let json = serde_json::to_string(&response).unwrap();
                data.cache.put(&key, &json.as_bytes())?;
                println!("{json}");
                println!("{:?}", response.unwrap().text()?);
            }

            if clear {
                Cache::default().clear()?;
            }
        }
    }

    Ok(())
}
