mod error;

use std::{
    error::Error,
    ffi::{c_char, CStr, CString},
    mem,
    path::Path,
};

use fenster_engine::Runner;

#[no_mangle]
pub extern "C" fn open_engine_with_path(path: *const c_char, engine_out: *mut *mut Runner) -> i32 {
    env_logger::init();
    error::capture_error(|| open_engine_with_path_private(path, engine_out))
}

fn open_engine_with_path_private(
    path: *const c_char,
    engine_out: *mut *mut Runner,
) -> Result<(), Box<dyn Error>> {
    let path = unsafe { CStr::from_ptr(path) };
    let path = path.to_str()?;

    let engine = Runner::new(Path::new(path))?;
    let engine = Box::into_raw(Box::new(engine));
    unsafe { *engine_out = engine }
    Ok(())
}

#[no_mangle]
pub extern "C" fn source_meta(engine: *mut Runner, out: *mut *mut c_char) -> i32 {
    error::capture_error(|| source_meta_private(engine, out))
}

fn source_meta_private(engine: *mut Runner, out: *mut *mut c_char) -> Result<(), Box<dyn Error>> {
    let engine = unsafe { engine.as_mut().unwrap() };
    let meta = engine.meta_raw().unwrap();
    let meta = CString::new(meta).unwrap();

    // The caller is responsible for handling the output
    unsafe { *out = meta.as_ptr() as *mut c_char };
    mem::forget(meta);

    Ok(())
}
