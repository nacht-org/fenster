use log::warn;
use quelle_core::prelude::LogEvent;
use wasmtime::Caller;

use super::utils::read_bytes_with_len;

pub fn event<D>(mut caller: Caller<'_, D>, ptr: i32, len: i32) {
    let memory = caller.get_export("memory").unwrap().into_memory().unwrap();
    let bytes = read_bytes_with_len(&mut caller, &memory, ptr, len as usize);

    let event = match serde_json::from_slice::<LogEvent>(bytes) {
        Ok(v) => v,
        Err(e) => {
            warn!("{e}");
            return;
        }
    };

    println!("{} - {}", event.level, event.args);
}
