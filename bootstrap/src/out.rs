use std::{
    ffi::CString,
    fmt::{self, Write},
    io,
    os::raw::c_char,
    panic,
};

extern "C" {
    fn ext_print(ptr: *const c_char);
    fn ext_eprint(ptr: *const c_char);
    fn ext_trace(ptr: *const c_char);
}

fn _print(buf: &str) -> io::Result<()> {
    let cstring = CString::new(buf)?;

    unsafe {
        ext_print(cstring.as_ptr());
    }

    Ok(())
}

fn _eprint(buf: &str) -> io::Result<()> {
    let cstring = CString::new(buf)?;

    unsafe {
        ext_eprint(cstring.as_ptr());
    }

    Ok(())
}

/// Used by the `print` macro
#[doc(hidden)]
pub fn _print_args(args: fmt::Arguments) {
    let mut buf = String::new();
    let _ = buf.write_fmt(args);
    let _ = _print(&buf);
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
    ($($arg:tt)*) => ($crate::_print_args(format_args!($($arg)*)));
}

/// Overrides the default `eprint!` macro.
#[macro_export]
macro_rules! eprint {
    ($($arg:tt)*) => ($crate::_eprint_args(format_args!($($arg)*)));
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
        let cstring = CString::new(err_info).unwrap();

        unsafe {
            ext_trace(cstring.as_ptr());
        }
    }));
}
