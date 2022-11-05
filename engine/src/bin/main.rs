use fenster_engine::Runner;
use std::error;

fn main() -> Result<(), Box<dyn error::Error>> {
    let mut runner = Runner::new("target/wasm32-unknown-unknown/debug/ext_scribblehub.wasm")?;
    runner.meta()?;
    runner.fetch_novel("https://www.royalroad.com/fiction/21220/mother-of-learning")?;
    Ok(())
}
