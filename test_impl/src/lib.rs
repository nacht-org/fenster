#[macro_use]
extern crate bootstrap;

use bootstrap::http::send_request;
use interface::{Method, Request};

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
