mod error;

use std::{
    error::Error,
    ffi::{c_char, CStr, CString},
    mem,
    path::Path,
};

use quelle_engine::Runner;

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

fn source_meta_private(
    engine: *mut Runner,
    buffer: *mut *mut c_char,
) -> Result<(), Box<dyn Error>> {
    let engine = unsafe { engine.as_mut().unwrap() };
    let meta = engine.meta_raw()?;
    write_buffer(buffer, meta)?;
    Ok(())
}

#[no_mangle]
pub extern "C" fn fetch_novel(
    engine: *mut Runner,
    url: *mut c_char,
    buffer: *mut *mut c_char,
) -> i32 {
    error::capture_error(|| fetch_novel_private(engine, url, buffer))
}

fn fetch_novel_private(
    engine: *mut Runner,
    url: *mut c_char,
    buffer: *mut *mut c_char,
) -> Result<(), Box<dyn Error>> {
    let url = unsafe { CStr::from_ptr(url) }.to_str()?;
    let engine = unsafe { engine.as_mut().unwrap() };

    let novel = engine.fetch_novel(url)?;
    let json = serde_json::to_string(&novel)?;
    write_buffer(buffer, json)?;

    Ok(())
}

#[no_mangle]
pub extern "C" fn fetch_chapter_content(
    engine: *mut Runner,
    url: *mut c_char,
    buffer: *mut *mut c_char,
) -> i32 {
    error::capture_error(|| fetch_chapter_content_private(engine, url, buffer))
}

fn fetch_chapter_content_private(
    engine: *mut Runner,
    url: *mut c_char,
    buffer: *mut *mut c_char,
) -> Result<(), Box<dyn Error>> {
    let url = unsafe { CStr::from_ptr(url) }.to_str()?;
    let engine = unsafe { engine.as_mut().unwrap() };

    let content = engine.fetch_chapter_content(url)?;
    write_buffer(buffer, content)?;

    Ok(())
}

fn write_buffer(buffer: *mut *mut c_char, string: String) -> Result<(), Box<dyn Error>> {
    let cstring = CString::new(string)?;
    // The caller is responsible for handling the output
    unsafe { *buffer = cstring.as_ptr() as *mut c_char };
    mem::forget(cstring);
    Ok(())
}
