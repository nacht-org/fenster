use std::ffi::CStr;

use fenster_core::Request;
use wasmtime::{AsContext, AsContextMut, Caller, Memory};

pub fn ext_print(mut caller: Caller<'_, ()>, ptr: i32) {
    println!("print called");

    let memory = caller.get_export("memory").unwrap().into_memory().unwrap();
    unsafe {
        let ptr = memory.data_ptr(&caller).offset(ptr as isize);
        let bytes = CStr::from_ptr(ptr as *const i8);
        let string = bytes.clone().to_str().unwrap();
        print!("{string}");
    }
}

pub fn ext_eprint(mut caller: Caller<'_, ()>, ptr: i32) {
    let memory = caller.get_export("memory").unwrap().into_memory().unwrap();
    unsafe {
        let ptr = memory.data_ptr(&caller).offset(ptr as isize);
        let bytes = CStr::from_ptr(ptr as *const i8);
        let string = bytes.clone().to_str().unwrap();
        eprint!("{string}");
    }
}

pub fn ext_trace(mut caller: Caller<'_, ()>, ptr: i32) {
    let memory = caller.get_export("memory").unwrap().into_memory().unwrap();
    unsafe {
        let ptr = memory.data_ptr(&caller).offset(ptr as isize);
        let bytes = CStr::from_ptr(ptr as *const i8);
        let string = bytes.clone().to_str().unwrap();
        eprintln!("{string}");
    }
}

pub fn ext_send_request(mut caller: Caller<'_, ()>, ptr: i32) -> i32 {
    let memory = caller.get_export("memory").unwrap().into_memory().unwrap();

    let request = read_string(&mut caller, &memory, ptr);
    let request = serde_json::from_str::<Request>(request).unwrap();
    println!("{request:?}");

    let value = serde_json::json!({
        "status": 200
    });
    let json = serde_json::to_string(&value).unwrap();
    write_string(&mut caller, &memory, json.as_str())
}

fn read_string<'c, 'm>(caller: &'c mut Caller<'_, ()>, memory: &'m Memory, ptr: i32) -> &'m str {
    unsafe {
        let ptr = memory.data_ptr(&caller).offset(ptr as isize);
        let cstr = CStr::from_ptr(ptr as *const i8);
        let str = cstr.clone().to_str().unwrap();
        str
    }
}

fn write_string<'c, 'm>(caller: &'c mut Caller<'_, ()>, memory: &'m Memory, value: &str) -> i32 {
    let alloc_func = caller.get_export("alloc").unwrap().into_func().unwrap();

    let ptr = alloc_func
        .typed::<i32, i32, _>(caller.as_context())
        .expect("Failed to parse func type")
        .call(caller.as_context_mut(), value.len() as i32)
        .expect("Failed while calling alloc");

    memory
        .write(caller.as_context_mut(), ptr as usize, value.as_bytes())
        .unwrap();

    ptr
}
