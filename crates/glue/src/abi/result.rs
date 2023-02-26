use serde::Serialize;
use std::{cell::RefCell, mem, ptr};

use super::ToWasmAbi;

thread_local! {
    static LAST_RESULT: RefCell<Option<Vec<u8>>>  = RefCell::new(None);
}

fn set_last_result(mut value: Vec<u8>) {
    value.shrink_to_fit();
    LAST_RESULT.with(|last_result| {
        *last_result.borrow_mut() = Some(value);
    })
}

fn take_last_result() -> Option<Vec<u8>> {
    LAST_RESULT.with(|prev| prev.borrow_mut().take())
}

#[no_mangle]
pub extern "C" fn last_result() -> *mut u8 {
    let mut last_result = match take_last_result() {
        Some(r) => r,
        None => return ptr::null_mut(),
    };

    let ptr = last_result.as_mut_ptr();
    mem::forget(last_result);
    ptr
}

impl<T, E> ToWasmAbi for Result<T, E>
where
    T: Serialize,
    E: Serialize,
{
    type Type = i32;

    fn to_wasm_abi(self) -> Self::Type {
        let self_is_err = self.is_err();

        let json_result = match self {
            Ok(v) => serde_json::to_vec(&v),
            Err(e) => serde_json::to_vec(&e),
        };

        let json_is_error = json_result.is_err();

        let response = match json_result {
            Ok(v) => v,
            Err(e) => e.to_string().into_bytes(),
        };

        let len = response.len() as i32;

        set_last_result(response);

        if self_is_err || json_is_error {
            -len
        } else {
            len
        }
    }
}
