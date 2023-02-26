use quelle_core::prelude::{BasicNovel, Novel, QuelleError};
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

fn result_to_wasm_abi<T, E>(result: Result<T, E>) -> i32
where
    T: Serialize,
    E: Serialize,
{
    let self_is_err = result.is_err();

    let json_result = match result {
        Ok(v) => serde_json::to_vec(&v),
        Err(e) => serde_json::to_vec(&e),
    };

    result_bytes_to_wasm_abi(json_result, self_is_err)
}

fn result_bytes_to_wasm_abi<E>(result: Result<Vec<u8>, E>, from_error: bool) -> i32
where
    E: ToString,
{
    let json_is_error = result.is_err();

    let response = match result {
        Ok(v) => v,
        Err(e) => e.to_string().into_bytes(),
    };

    let len = response.len() as i32;
    set_last_result(response);

    if from_error || json_is_error {
        -len
    } else {
        len
    }
}

macro_rules! impl_to_abi_for_result {
    (Option<$ok:ty>, $err:ty) => {
        impl ToWasmAbi for Result<Option<$ok>, $err> {
            type Type = i32;

            #[inline]
            fn to_wasm_abi(self) -> Self::Type {
                match self {
                    Ok(None) => 0,
                    _ => result_to_wasm_abi(self),
                }
            }
        }
    };
    (String, $err:ty) => {
        impl ToWasmAbi for Result<String, $err> {
            type Type = i32;

            #[inline]
            fn to_wasm_abi(self) -> Self::Type {
                let self_is_err = self.is_err();

                let json_result = match self {
                    Ok(v) => Ok(v.into_bytes()),
                    Err(e) => serde_json::to_vec(&e),
                };

                result_bytes_to_wasm_abi(json_result, self_is_err)
            }
        }
    };
    ($ok:ty, $err:ty) => {
        impl ToWasmAbi for Result<$ok, $err> {
            type Type = i32;

            fn to_wasm_abi(self) -> Self::Type {
                result_to_wasm_abi(self)
            }
        }
    };
}

impl_to_abi_for_result!(Novel, QuelleError);
impl_to_abi_for_result!(Vec<BasicNovel>, QuelleError);
impl_to_abi_for_result!(Option<String>, QuelleError);
impl_to_abi_for_result!(String, QuelleError);
