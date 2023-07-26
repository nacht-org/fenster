use log::trace;
use wasmtime::Caller;

use crate::module::utils::read_str_with_len;

pub fn print<D>(mut caller: Caller<'_, D>, ptr: i32, len: u32) {
    trace!("executing exposed function 'print'");

    let memory = caller.get_export("memory").unwrap().into_memory().unwrap();
    let string = read_str_with_len(&mut caller, &memory, ptr, len as usize);
    print!("{string}");
}

pub fn eprint<D>(mut caller: Caller<'_, D>, ptr: i32, len: u32) {
    trace!("executing exposed function 'eprint'");

    let memory = caller.get_export("memory").unwrap().into_memory().unwrap();
    let string = read_str_with_len(&mut caller, &memory, ptr, len as usize);
    eprint!("{string}");
}

pub fn trace<D>(mut caller: Caller<'_, D>, ptr: i32, len: u32) {
    trace!("executing exposed function 'trace'");

    let memory = caller.get_export("memory").unwrap().into_memory().unwrap();
    let string = read_str_with_len(&mut caller, &memory, ptr, len as usize);
    eprintln!("{string}");
}
