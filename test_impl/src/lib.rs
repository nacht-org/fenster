#[macro_use]
extern crate fenster_glue;

use fenster_core::prelude::*;
use fenster_glue::prelude::*;

#[no_mangle]
pub extern "C" fn meta() -> i32 {
    0
}

#[no_mangle]
pub extern "C" fn fetch_novel(_: i32) -> i32 {
    0
}

#[expose]
pub fn main() {
    set_panic_hook();

    let request = Request {
        method: Method::Get,
        url: String::from("https://ggle.com"),
        params: None,
        data: None,
        headers: None,
    };

    let response = http::send_request(request);
    println!("{response:#?}");
}
