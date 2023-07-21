mod result;

use std::{
    error::Error,
    ffi::{c_char, CStr},
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
    result::capture_error(|| open_engine_with_path_private(path, engine_out))
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
pub extern "C" fn source_meta(engine: *mut Runner) -> i32 {
    result::capture_memloc(|| unsafe {
        let engine = engine.as_mut().ok_or(CustomError::WrongEnginePtr)?;
        let memloc = engine.meta_memloc()?;
        Ok(memloc)
    })
}

#[no_mangle]
pub extern "C" fn fetch_novel(engine: *mut Runner, url: *mut c_char) -> i32 {
    result::capture_memloc(|| unsafe {
        let url = CStr::from_ptr(url).to_str()?;
        let engine = engine.as_mut().ok_or(CustomError::WrongEnginePtr)?;
        let memloc = engine.fetch_novel_memloc(url)?;
        Ok(memloc)
    })
}

#[no_mangle]
pub extern "C" fn fetch_chapter_content(engine: *mut Runner, url: *mut c_char) -> i32 {
    result::capture_memloc(|| unsafe {
        let url = CStr::from_ptr(url).to_str()?;
        let engine = engine.as_mut().ok_or(CustomError::WrongEnginePtr)?;
        let content = engine.fetch_chapter_content_memloc(url)?;
        Ok(content)
    })
}

#[no_mangle]
pub extern "C" fn popular_supported(engine: *mut Runner) -> i32 {
    result::capture_error_with_return(|| {
        let engine = unsafe { engine.as_mut().ok_or(CustomError::WrongEnginePtr)? };
        Ok(engine.popular_supported() as i32)
    })
}

#[no_mangle]
pub extern "C" fn popular_url(engine: *mut Runner, page: i32) -> i32 {
    result::capture_memloc(|| unsafe {
        let engine = engine.as_mut().ok_or(CustomError::WrongEnginePtr)?;
        let memloc = engine.popular_url_memloc(page)?;
        Ok(memloc)
    })
}

#[no_mangle]
pub extern "C" fn popular(engine: *mut Runner, page: i32) -> i32 {
    result::capture_memloc(|| unsafe {
        let engine = engine.as_mut().ok_or(CustomError::WrongEnginePtr)?;
        let memloc = engine.popular_memloc(page)?;
        Ok(memloc)
    })
}

#[no_mangle]
pub extern "C" fn text_search_supported(engine: *mut Runner) -> i32 {
    result::capture_error_with_return(|| {
        let engine = unsafe { engine.as_ref().ok_or(CustomError::WrongEnginePtr)? };
        Ok(engine.text_search_supported() as i32)
    })
}

#[no_mangle]
pub extern "C" fn text_search(engine: *mut Runner, query: *mut c_char, page: i32) -> i32 {
    result::capture_memloc(|| unsafe {
        let query = CStr::from_ptr(query).to_str()?;
        let engine = engine.as_mut().ok_or(CustomError::WrongEnginePtr)?;
        let memloc = engine.text_search_memloc(query, page)?;
        Ok(memloc)
    })
}

#[no_mangle]
pub extern "C" fn memloc_dealloc(engine: *mut Runner, ptr: i32, len: i32) -> i32 {
    result::capture_error(|| {
        let engine = unsafe { engine.as_mut().ok_or(CustomError::WrongEnginePtr)? };
        engine.dealloc_memory(ptr, len)?;
        Ok(())
    })
}
