#[macro_use]
extern crate fenster_glue;

use fenster_core::{Method, Request};
use fenster_glue::http::send_request;

#[no_mangle]
pub extern "C" fn _main() {
    let request = Request {
        method: Method::Get,
        url: String::from("https://google.com"),
        params: None,
        data: None,
        headers: None,
    };

    let response = send_request(request).unwrap();
    println!("{response:?}");
}
