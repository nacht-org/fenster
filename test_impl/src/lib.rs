#[macro_use]
extern crate fenster_glue;

use fenster_core::prelude::*;
use fenster_glue::prelude::*;

#[expose]
pub fn fetch_novel(url: String) -> Result<(), String> {
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

    let response = http::send_request(request);
    println!("{response:#?}");
}
