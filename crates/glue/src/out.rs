use std::{
    fmt::{self, Write},
    panic,
};

use crate::prelude::ToWasmAbi;

#[link(name = "io")]
extern "C" {
    fn ext_print(ptr: *const u8);
    fn ext_eprint(ptr: *const u8);
    fn ext_trace(ptr: *const u8);
}

#[inline]
fn _print(buf: &str) {
    unsafe {
        ext_print(buf.to_wasm_abi());
    }
}

#[inline]
fn _eprint(buf: &str) {
    unsafe {
        ext_eprint(buf.to_wasm_abi());
    }
}

/// Used by the `print` macro
#[doc(hidden)]
pub fn _print_args(args: fmt::Arguments) {
    let mut buf = String::new();
    let _ = buf.write_fmt(args);
    let _ = _print(&buf);
}

/// Used by the `println` macro
#[doc(hidden)]
pub fn _print_args_nl(args: fmt::Arguments) {
    let mut buf = String::new();
    let _ = buf.write_fmt(args);
    buf.push('\n');
    let _ = _eprint(&buf);
}

/// Used by the `eprint` macro
#[doc(hidden)]
pub fn _eprint_args(args: fmt::Arguments) {
    let mut buf = String::new();
    let _ = buf.write_fmt(args);
    let _ = _eprint(&buf);
}

/// Overrides the default `print!` macro.
#[macro_export]
macro_rules! print {
    ($($arg:tt)*) => ($crate::out::_print_args(format_args!($($arg)*)));
}

#[macro_export]
macro_rules! println {
    ($($arg:tt)*) => ($crate::out::_print_args_nl(format_args!($($arg)*)));
}

/// Overrides the default `eprint!` macro.
#[macro_export]
macro_rules! eprint {
    ($($arg:tt)*) => ($crate::out::_eprint_args(format_args!($($arg)*)));
}

pub fn set_panic_hook() {
    panic::set_hook(Box::new(|info| {
        let file = info.location().unwrap().file();
        let line = info.location().unwrap().line();
        let col = info.location().unwrap().column();

        let msg = match info.payload().downcast_ref::<&'static str>() {
            Some(s) => *s,
            None => match info.payload().downcast_ref::<String>() {
                Some(s) => &s[..],
                None => "Box<Any>",
            },
        };

        let err_info = format!("Panicked at '{}', {}:{}:{}", msg, file, line, col);

        unsafe {
            ext_trace(err_info.as_str().to_wasm_abi());
        }
    }));
}
