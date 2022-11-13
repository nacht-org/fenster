use fenster_engine::Runner;
use log::{info, trace, LevelFilter};
use std::error;

fn main() -> Result<(), Box<dyn error::Error>> {
    env_logger::builder()
        .filter_level(LevelFilter::Info)
        .filter_module("fenster_engine", LevelFilter::Trace)
        .parse_default_env()
        .init();

    trace!("initializing the wasm engine...");
    let mut runner = Runner::new("target/wasm32-unknown-unknown/debug/extension_royalroad.wasm")?;

    // runner.main()?;

    info!("Calling exposed wasm 'meta' function");
    let meta = runner.meta()?;
    println!("{meta:#?}");

    // info!("Calling exposed wasm 'fetch_novel' function");
    // runner.fetch_novel("https://www.royalroad.com/fiction/21220/mother-of-learning")?;
    Ok(())
}
