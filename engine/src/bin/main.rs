use fenster_engine::{ext_eprint, ext_print, ext_send_request, ext_trace, Runner};
use std::error;
use wasmtime::*;

fn main() -> Result<(), Box<dyn error::Error>> {
    let mut runner = Runner::new("target/wasm32-unknown-unknown/release/ext_scribblehub.wasm")?;
    runner.meta()?;
    Ok(())
}
