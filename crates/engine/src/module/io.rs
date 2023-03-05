use log::trace;
use wasmtime::Caller;

use crate::{module::utils::read_str_with_len, Data};

pub fn ext_print(mut caller: Caller<'_, Data>, ptr: i32, len: u32) {
    trace!("executing exposed function 'ext_print'");

    let memory = caller.get_export("memory").unwrap().into_memory().unwrap();
    let string = read_str_with_len(&mut caller, &memory, ptr, len as usize);
    print!("{string}");
}

pub fn ext_eprint(mut caller: Caller<'_, Data>, ptr: i32, len: u32) {
    trace!("executing exposed function 'ext_eprint'");

    let memory = caller.get_export("memory").unwrap().into_memory().unwrap();
    let string = read_str_with_len(&mut caller, &memory, ptr, len as usize);
    eprint!("{string}");
}

pub fn ext_trace(mut caller: Caller<'_, Data>, ptr: i32, len: u32) {
    trace!("executing exposed function 'ext_trace'");

    let memory = caller.get_export("memory").unwrap().into_memory().unwrap();
    let string = read_str_with_len(&mut caller, &memory, ptr, len as usize);
    eprintln!("{string}");
}
