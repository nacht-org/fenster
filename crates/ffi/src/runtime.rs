use std::slice;

use quelle_engine::{
    module::utils::{read_bytes_with_len, write_str},
    Runtime,
};
use wasmtime::Caller;

use crate::result::{get_last_offset, get_last_pointer};

pub type RuntimeFfi = Runtime<FfiData>;

pub type SendRequestFn = unsafe extern "C" fn(ptr: *const u8, len: i32) -> i32;

pub type LogEventFn = unsafe extern "C" fn(ptr: *const u8, len: i32);

pub struct FfiData {
    pub send_request: SendRequestFn,
    pub log_event: LogEventFn,
}

pub fn send_request(mut caller: Caller<'_, FfiData>, ptr: i32, len: i32) -> i32 {
    let mut memory = caller.get_export("memory").unwrap().into_memory().unwrap();
    let str = read_bytes_with_len(&mut caller, &mut memory, ptr, len as usize);
    let data = caller.data();
    unsafe { (data.send_request)(str.as_ptr(), str.len() as i32) };
    let (ptr, len) = loop {
        match (get_last_pointer(), get_last_offset()) {
            (Some(ptr), Some(len)) => break (ptr, len),
            n @ _ => println!("{:?}", n),
        }
    };
    let response =
        String::from_utf8_lossy(unsafe { slice::from_raw_parts(ptr as *const u8, len as usize) });
    println!("{response}");
    write_str(&mut caller, &memory, &response)
}

pub fn log_event(mut caller: Caller<'_, FfiData>, ptr: i32, len: i32) {
    let mut memory = caller.get_export("memory").unwrap().into_memory().unwrap();
    let str = read_bytes_with_len(&mut caller, &mut memory, ptr, len as usize);
    let data = caller.data();
    unsafe { (data.log_event)(str.as_ptr(), str.len() as i32) }
}
