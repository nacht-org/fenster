use std::slice;

use log::{debug, info};
use wasmtime::{AsContext, AsContextMut, Caller, Memory};

use crate::Data;

pub fn read_str<'c, 'm>(caller: &'c mut Caller<'_, Data>, memory: &'m Memory, ptr: i32) -> &'m str {
    let len = stack_pop(caller) as usize;
    debug!("retrieved byte length from stack: {len}");
    read_str_with_len(caller, memory, ptr, len)
}

pub fn read_str_with_len<'c, 'm>(
    caller: &'c mut Caller<'_, Data>,
    memory: &'m Memory,
    ptr: i32,
    len: usize,
) -> &'m str {
    info!("reading string from wasm memory of len: {len}");

    unsafe {
        let ptr = memory.data_ptr(&caller).offset(ptr as isize);
        let bytes = slice::from_raw_parts(ptr, len);
        std::str::from_utf8(bytes).unwrap()
    }
}

pub fn read_bytes_with_len<'c, 'm>(
    caller: &'c mut Caller<'_, Data>,
    memory: &'m Memory,
    ptr: i32,
    len: usize,
) -> &'m [u8] {
    info!("reading bytes from wasm memory of len: {len}");

    unsafe {
        let ptr = memory.data_ptr(&caller).offset(ptr as isize);
        let bytes = slice::from_raw_parts(ptr, len);
        bytes
    }
}

pub fn write_str<'c, 'm>(caller: &'c mut Caller<'_, Data>, memory: &'m Memory, value: &str) -> i32 {
    let alloc_func = caller.get_export("alloc").unwrap().into_func().unwrap();

    let ptr = alloc_func
        .typed::<i32, i32>(caller.as_context())
        .unwrap()
        .call(caller.as_context_mut(), value.len() as i32)
        .unwrap();

    stack_push(caller, value.len() as i32);

    memory
        .write(caller.as_context_mut(), ptr as usize, value.as_bytes())
        .unwrap();

    ptr
}

pub fn stack_push<'c, 'm>(caller: &'c mut Caller<'_, Data>, value: i32) {
    let push_fn = caller
        .get_export("stack_push")
        .unwrap()
        .into_func()
        .unwrap();

    push_fn
        .typed::<i32, ()>(&caller)
        .unwrap()
        .call(caller, value)
        .unwrap();
}

pub fn stack_pop<'c, 'm>(caller: &'c mut Caller<'_, Data>) -> i32 {
    let pop_fn = caller.get_export("stack_pop").unwrap().into_func().unwrap();

    let value = pop_fn
        .typed::<(), i32>(&caller)
        .unwrap()
        .call(caller, ())
        .unwrap();

    value
}
