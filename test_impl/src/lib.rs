#[macro_use]
extern crate fenster_glue;

use fenster_core::{Method, Request};
use fenster_glue::{http::send_request, out::set_panic_hook};
use fenster_glue_derive::expose;

#[expose]
pub fn fetch_novel(url: String, skip: bool) -> Result<(), String> {
    Ok(())
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

    let response = send_request(request);
    println!("{response:#?}");
}
