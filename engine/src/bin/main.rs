use fenster_engine::Runner;
use log::LevelFilter;
use std::error;

fn main() -> Result<(), Box<dyn error::Error>> {
    env_logger::builder()
        .filter_level(LevelFilter::Info)
        .filter_module("fenster_engine", LevelFilter::Trace)
        .parse_default_env()
        .init();

    let mut runner = Runner::new("target/wasm32-unknown-unknown/debug/ext_scribblehub.wasm")?;
    // runner.main()?;
    runner.meta()?;
    runner.fetch_novel("https://www.royalroad.com/fiction/21220/mother-of-learning")?;
    Ok(())
}
