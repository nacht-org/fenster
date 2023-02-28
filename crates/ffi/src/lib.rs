mod error;

use std::{
    error::Error,
    ffi::{c_char, CStr, CString},
    mem,
    path::Path,
};

use quelle_engine::Runner;

#[derive(thiserror::Error, Debug)]
enum CustomError {
    #[error("pointer does not reference a valid engine")]
    WrongEnginePtr,
}

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
    let engine = unsafe { engine.as_mut().ok_or(CustomError::WrongEnginePtr)? };
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
    let engine = unsafe { engine.as_mut().ok_or(CustomError::WrongEnginePtr)? };

    let content = engine.fetch_novel_raw(url)?;
    write_buffer(buffer, content)?;

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
    let engine = unsafe { engine.as_mut().ok_or(CustomError::WrongEnginePtr)? };

    let content = engine.fetch_chapter_content(url)?;
    write_buffer(buffer, content)?;

    Ok(())
}

#[no_mangle]
pub extern "C" fn popular_supported(engine: *mut Runner) -> i32 {
    error::capture_error_with_return(|| {
        let engine = unsafe { engine.as_mut().ok_or(CustomError::WrongEnginePtr)? };
        Ok(if engine.popular_supported() { 1 } else { 0 })
    })
}

#[no_mangle]
pub extern "C" fn popular(engine: *mut Runner, page: i32, buffer: *mut *mut c_char) -> i32 {
    error::capture_error(|| {
        let engine = unsafe { engine.as_mut().ok_or(CustomError::WrongEnginePtr)? };
        let novels = engine.popular(page)?;
        let content = serde_json::to_string(&novels)?;

        write_buffer(buffer, content)
    })
}

#[no_mangle]
pub extern "C" fn text_search(
    engine: *mut Runner,
    query: *mut c_char,
    page: i32,
    buffer: *mut *mut c_char,
) -> i32 {
    error::capture_error(|| text_search_private(engine, query, page, buffer))
}

fn text_search_private(
    engine: *mut Runner,
    query: *mut c_char,
    page: i32,
    buffer: *mut *mut c_char,
) -> Result<(), Box<dyn Error>> {
    let query = unsafe { CStr::from_ptr(query) }.to_str()?;
    let engine = unsafe { engine.as_mut().ok_or(CustomError::WrongEnginePtr)? };
    let content = engine.text_search_raw(query, page)?;
    write_buffer(buffer, content)
}

fn write_buffer(buffer: *mut *mut c_char, content: String) -> Result<(), Box<dyn Error>> {
    let cstring = CString::new(content)?;
    // The caller is responsible for handling the output
    unsafe { *buffer = cstring.as_ptr() as *mut c_char };
    mem::forget(cstring);
    Ok(())
}
