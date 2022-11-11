use std::{ffi::CString, mem, sync::Mutex};

#[no_mangle]
pub extern "C" fn alloc(len: usize) -> *mut u8 {
    let layout = std::alloc::Layout::from_size_align(len, 1).unwrap();
    unsafe { std::alloc::alloc(layout) }
}

#[no_mangle]
pub extern "C" fn dealloc(ptr: *mut u8, len: usize) {
    let layout = std::alloc::Layout::from_size_align(len, 1).unwrap();
    unsafe { std::alloc::dealloc(ptr, layout) }
}

#[no_mangle]
pub extern "C" fn dealloc_string(ptr: *mut i8) {
    unsafe {
        let _ = CString::from_raw(ptr);
    }
}

#[derive(Debug)]
struct Stack {
    array: [i32; 256],
    top: usize,
}

static STACK: Mutex<Stack> = Mutex::new(Stack {
    array: [0; 256],
    top: 0,
});

#[no_mangle]
pub extern "C" fn stack_push(value: i32) {
    let mut stack = STACK.lock().unwrap();
    let top = stack.top;
    stack.array[top] = value;
    stack.top += 1;
}

#[no_mangle]
pub extern "C" fn stack_pop() -> i32 {
    let mut stack = STACK.lock().unwrap();
    stack.top -= 1;
    let value = stack.array[stack.top];
    value
}

pub trait ToMem {
    type Type;
    fn to_mem(self) -> Self::Type;
}

pub trait FromMem {
    type Type;
    fn from_mem(value: Self::Type) -> Self;
}

impl ToMem for String {
    type Type = *mut u8;
    fn to_mem(self) -> Self::Type {
        let mut bytes = self.into_bytes();
        bytes.shrink_to_fit();

        stack_push(bytes.len() as i32);

        let ptr = bytes.as_mut_ptr();
        mem::forget(bytes);

        ptr
    }
}

impl FromMem for String {
    type Type = *mut u8;
    fn from_mem(value: Self::Type) -> Self {
        let len = stack_pop() as usize;

        let bytes = unsafe { Vec::from_raw_parts(value, len, len) };
        String::from_utf8(bytes).unwrap()
    }
}

impl ToMem for &str {
    type Type = *const u8;
    fn to_mem(self) -> Self::Type {
        stack_push(self.len() as i32);
        self.as_ptr()
    }
}
