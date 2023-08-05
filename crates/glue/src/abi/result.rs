use quelle_core::prelude::{BasicNovel, Content, Novel, QuelleError};
use serde::Serialize;
use std::{cell::RefCell, mem, ptr};

use super::ToWasmAbi;

thread_local! {
    static LAST_RESULT: RefCell<Option<Vec<u8>>> = RefCell::new(None);
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

fn store_serde<S>(value: S, force_neg: bool) -> i32
where
    S: Serialize,
{
    let serial_result = serde_json::to_vec(&value);
    let serial_is_err = serial_result.is_err();

    let bytes = match serial_result {
        Ok(v) => v,
        Err(e) => {
            let wrapped_err = QuelleError::WasmAbiError(e.to_string());
            serde_json::to_vec(&wrapped_err).unwrap()
        }
    };

    let len = store_bytes(bytes);
    if force_neg || serial_is_err {
        -len
    } else {
        len
    }
}

fn store_bytes(bytes: Vec<u8>) -> i32 {
    let len = bytes.len() as i32;
    set_last_result(bytes);
    len
}

#[inline]
fn store_error<E>(e: E) -> i32
where
    E: Serialize,
{
    store_serde(e, true)
}

impl ToWasmAbi for Result<String, QuelleError> {
    type Type = i32;

    #[inline]
    fn to_wasm_abi(self) -> Self::Type {
        match self {
            Ok(v) if v.is_empty() => 0,
            Ok(v) => store_bytes(v.into_bytes()),
            Err(e) => store_error(e),
        }
    }
}

impl ToWasmAbi for Result<Novel, QuelleError> {
    type Type = i32;

    #[inline]
    fn to_wasm_abi(self) -> Self::Type {
        match self {
            Ok(v) => store_serde(v, false),
            Err(e) => store_error(e),
        }
    }
}

impl ToWasmAbi for Result<Content, QuelleError> {
    type Type = i32;

    #[inline]
    fn to_wasm_abi(self) -> Self::Type {
        match self {
            Ok(v) => store_serde(v, false),
            Err(e) => store_error(e),
        }
    }
}

impl ToWasmAbi for Result<Vec<BasicNovel>, QuelleError> {
    type Type = i32;

    #[inline]
    fn to_wasm_abi(self) -> Self::Type {
        match self {
            Ok(v) => store_serde(v, false),
            Err(e) => store_error(e),
        }
    }
}
