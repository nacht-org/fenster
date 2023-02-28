use std::{cell::RefCell, error, mem, ptr};

thread_local! {
    static LAST_RESULT: RefCell<Option<Vec<u8>>> = RefCell::new(None);
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
