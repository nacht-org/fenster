use std::mem;

use quelle_core::prelude::Meta;

use super::{stack_pop, stack_push};

pub trait ToWasmAbi {
    type Type;
    fn to_wasm_abi(self) -> Self::Type;
}

pub trait FromWasmAbi {
    type Type;
    fn from_wasm_abi(value: Self::Type) -> Self;
}

impl ToWasmAbi for String {
    type Type = *mut u8;
    fn to_wasm_abi(self) -> Self::Type {
        let mut bytes = self.into_bytes();
        bytes.shrink_to_fit();

        stack_push(bytes.len() as i32);

        let ptr = bytes.as_mut_ptr();
        mem::forget(bytes);

        ptr
    }
}

impl FromWasmAbi for String {
    type Type = *mut u8;
    fn from_wasm_abi(value: Self::Type) -> Self {
        let len = stack_pop() as usize;

        let bytes = unsafe { Vec::from_raw_parts(value, len, len) };
        String::from_utf8(bytes).unwrap()
    }
}

impl ToWasmAbi for &str {
    type Type = *const u8;
    fn to_wasm_abi(self) -> Self::Type {
        stack_push(self.len() as i32);
        self.as_ptr()
    }
}

#[macro_export]
macro_rules! impl_wasm_abi_for_serde {
    ($name:ty) => {
        impl_from_abi_for_serde!($name)
        impl_to_abi_for_serde!($name)
    };
}

#[macro_export]
macro_rules! impl_from_abi_for_serde {
    ($name:ty) => {
        impl crate::abi::ToWasmAbi for $name {
            type Type = *mut u8;

            fn from_wasm_abi(value: Self::Type) -> Self {
                let len = crate::mem::stack_pop() as usize;
                let bytes = unsafe { Vec::from_raw_parts(value, len, len) };
                serde_json::from_bytes(bytes).unwrap()
            }
        }
    };
}

#[macro_export]
macro_rules! impl_to_abi_for_serde {
    ($name:ty) => {
        impl crate::abi::ToWasmAbi for $name {
            type Type = *mut u8;

            fn to_wasm_abi(self) -> Self::Type {
                let mut string = serde_json::to_string(&self).unwrap();
                crate::abi::stack_push(string.len() as i32);

                let ptr = string.as_mut_ptr();
                mem::forget(string);
                ptr
            }
        }
    };
}

impl_to_abi_for_serde!(&Meta);
