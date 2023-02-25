use std::sync::Mutex;

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
