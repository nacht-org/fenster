use std::{
    cell::RefCell,
    error::Error,
    ffi::{c_char, c_int, CString},
    mem,
};

thread_local! {
    static LAST_ERROR: RefCell<Option<Box<dyn Error>>> = RefCell::new(None);
}

/// Update the most recent error, clearing whatever may have been there before.
pub fn update_last_error(err: Box<dyn Error>) {
    // error!("Setting LAST_ERROR: {}", err);

    // {
    //     // Print a pseudo-backtrace for this error, following back each error's
    //     // cause until we reach the root error.
    //     let mut cause = err.cause();
    //     while let Some(parent_err) = cause {
    //         // warn!("Caused by: {}", parent_err);
    //         cause = parent_err.cause();
    //     }
    // }

    LAST_ERROR.with(|prev| {
        *prev.borrow_mut() = Some(err);
    });
}

/// Retrieve the most recent error, clearing it in the process.
pub fn take_last_error() -> Option<Box<dyn Error>> {
    LAST_ERROR.with(|prev| prev.borrow_mut().take())
}

#[no_mangle]
pub unsafe extern "C" fn last_error_message(buffer: *mut *mut c_char) -> c_int {
    let last_error = match take_last_error() {
        Some(err) => err,
        None => return 0,
    };

    let error_message = last_error.to_string();
    if error_message.is_empty() {
        return 0;
    }

    let error_len = error_message.len();

    let cstring = match CString::new(error_message) {
        Ok(v) => v,
        Err(_) => return -1,
    };

    // The caller is responsible for handling the output
    unsafe { *buffer = cstring.as_ptr() as *mut c_char };
    mem::forget(cstring);

    error_len as c_int
}

pub fn capture_error(f: impl Fn() -> Result<(), Box<dyn Error>>) -> i32 {
    match f() {
        Ok(_) => 0,
        Err(e) => {
            update_last_error(e);
            -1
        }
    }
}

pub fn capture_error_with_return(f: impl Fn() -> Result<i32, Box<dyn Error>>) -> i32 {
    match f() {
        Ok(v) => v,
        Err(e) => {
            update_last_error(e);
            -1
        }
    }
}
