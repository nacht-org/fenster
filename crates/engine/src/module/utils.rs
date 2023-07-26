use std::slice;

use log::{debug, info};
use wasmtime::{AsContext, AsContextMut, Caller, Memory};

pub async fn read_str<'c, 'm, D: Send>(
    caller: &'c mut Caller<'_, D>,
    memory: &'m Memory,
    ptr: i32,
) -> &'m str {
    let len = stack_pop(caller).await as usize;
    debug!("retrieved byte length from stack: {len}");
    read_str_with_len(caller, memory, ptr, len)
}

pub fn read_str_with_len<'c, 'm, D>(
    caller: &'c mut Caller<'_, D>,
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

pub fn read_bytes_with_len<'c, 'm, D>(
    caller: &'c mut Caller<'_, D>,
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

pub async fn write_str<'c, 'm, D: Send>(
    caller: &'c mut Caller<'_, D>,
    memory: &'m Memory,
    value: &str,
) -> i32 {
    let alloc_func = caller.get_export("alloc").unwrap().into_func().unwrap();

    let ptr = alloc_func
        .typed::<i32, i32>(caller.as_context())
        .unwrap()
        .call_async(caller.as_context_mut(), value.len() as i32)
        .await
        .unwrap();

    stack_push(caller, value.len() as i32).await;

    memory
        .write(caller.as_context_mut(), ptr as usize, value.as_bytes())
        .unwrap();

    ptr
}

pub async fn stack_push<'c, 'm, D: Send>(caller: &'c mut Caller<'_, D>, value: i32) {
    let push_fn = caller
        .get_export("stack_push")
        .unwrap()
        .into_func()
        .unwrap();

    push_fn
        .typed::<i32, ()>(&caller)
        .unwrap()
        .call_async(caller, value)
        .await
        .unwrap();
}

pub async fn stack_pop<'c, 'm, D: Send>(caller: &'c mut Caller<'_, D>) -> i32 {
    let pop_fn = caller.get_export("stack_pop").unwrap().into_func().unwrap();

    let value = pop_fn
        .typed::<(), i32>(&caller)
        .unwrap()
        .call_async(caller, ())
        .await
        .unwrap();

    value
}
