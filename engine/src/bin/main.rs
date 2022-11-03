use fenster_engine::{ext_eprint, ext_print, ext_send_request, ext_trace};
use std::error::Error;
use wasmtime::*;

fn main() -> Result<(), Box<dyn Error>> {
    let engine = Engine::default();
    let mut linker = Linker::new(&engine);

    let module = Module::from_file(
        &engine,
        "target/wasm32-unknown-unknown/debug/test_impl.wasm",
    )?;

    linker.func_wrap("env", "ext_send_request", ext_send_request)?;
    linker.func_wrap("env", "ext_print", ext_print)?;
    linker.func_wrap("env", "ext_eprint", ext_eprint)?;
    linker.func_wrap("env", "ext_trace", ext_trace)?;

    let mut store = Store::new(&engine, ());
    let instance = linker.instantiate(&mut store, &module)?;

    let run = instance
        .get_func(&mut store, "main")
        .expect("'main' was not an exported function");

    run.typed::<(), (), _>(&store)?.call(&mut store, ())?;
    Ok(())
}
