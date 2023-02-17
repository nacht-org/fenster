use std::{
    ffi::{c_char, CStr, CString},
    mem,
    path::Path,
};

use fenster_engine::Runner;
use std::cell::RefCell;

thread_local! {
    static LAST_ERROR: RefCell<Option<Box<dyn std::error::Error>>> = RefCell::new(None);
}

#[no_mangle]
#[allow(unused_variables, unused_assignments)]
pub extern "C" fn open_engine_with_path(path: *const c_char, engine_out: *mut *mut Runner) -> i32 {
    env_logger::init();

    let path = unsafe { CStr::from_ptr(path) };
    let path = path.to_str().unwrap();
    println!("{path}");

    let mut engine = Runner::new(Path::new(path)).unwrap();
    println!("{:?}", engine.meta());

    let engine = Box::into_raw(Box::new(engine));
    unsafe { *engine_out = engine }

    0
}

#[no_mangle]
pub extern "C" fn source_meta(engine: *mut Runner, out: *mut *mut c_char) -> i32 {
    let engine = unsafe { engine.as_mut().unwrap() };
    let meta = engine.meta_raw().unwrap();
    let meta = CString::new(meta).unwrap();

    // The caller is responsible for handling the output
    unsafe { *out = meta.as_ptr() as *mut c_char };
    mem::forget(meta);

    0
}
