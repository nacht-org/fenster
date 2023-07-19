use std::{cell::RefCell, error, mem, ptr};

use quelle_engine::MemLoc;

thread_local! {
    static LAST_RESULT: RefCell<Option<Vec<u8>>> = RefCell::new(None);
    static LAST_POINTER: RefCell<Option<*mut u8>> = RefCell::new(None);
    static LAST_OFFSET: RefCell<Option<i32>> = RefCell::new(None);
}

fn set_last_result(mut bytes: Vec<u8>) -> usize {
    bytes.shrink_to_fit();
    let len = bytes.len();
    LAST_RESULT.with(|last_result| {
        *last_result.borrow_mut() = Some(bytes);
    });
    len
}

#[no_mangle]
pub extern "C" fn last_result() -> *mut u8 {
    let last_result = LAST_RESULT.with(|prev| prev.borrow_mut().take());

    let mut bytes = match last_result {
        Some(r) => r,
        None => return ptr::null_mut(),
    };

    let ptr = bytes.as_mut_ptr();
    mem::forget(bytes);
    ptr
}

pub fn capture_result(f: impl Fn() -> Result<Vec<u8>, Box<dyn error::Error>>) -> i32 {
    match f() {
        Ok(b) if b.is_empty() => 0,
        Ok(b) => set_last_result(b) as i32,
        Err(e) => -(set_last_result(e.to_string().into_bytes()) as i32),
    }
}

#[no_mangle]
pub extern "C" fn last_pointer() -> *mut u8 {
    let last_pointer = LAST_POINTER.with(|prev| prev.borrow_mut().take());
    return last_pointer.unwrap_or_else(|| ptr::null_mut());
}

#[no_mangle]
pub extern "C" fn last_offset() -> i32 {
    let last_offset = LAST_OFFSET.with(|prev| prev.borrow_mut().take());
    return last_offset.unwrap_or(-1);
}

pub fn capture_memloc(f: impl Fn() -> Result<MemLoc, Box<dyn error::Error>>) -> i32 {
    match f() {
        Ok(memloc) => {
            LAST_POINTER.with(|inner| {
                *inner.borrow_mut() = Some(memloc.ptr);
            });
            LAST_OFFSET.with(|inner| {
                *inner.borrow_mut() = Some(memloc.offset);
            });
            memloc.len
        }
        Err(e) => {
            println!("{}", e);
            -(set_last_result(e.to_string().into_bytes()) as i32)
        }
    }
}

pub fn capture_error(f: impl Fn() -> Result<(), Box<dyn error::Error>>) -> i32 {
    match f() {
        Ok(()) => 0,
        Err(e) => -(set_last_result(e.to_string().into_bytes()) as i32),
    }
}
