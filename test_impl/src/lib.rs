#[macro_use]
extern crate fenster_glue;

use fenster_core::{Method, Request};
use fenster_glue::{http::send_request, out::set_panic_hook};

#[no_mangle]
pub extern "C" fn _main() {
    set_panic_hook();

    let request = Request {
        method: Method::Get,
        url: String::from("https://ggle.com"),
        params: None,
        data: None,
        headers: None,
    };

    let response = send_request(request);
    println!("{response:#?}");
}
