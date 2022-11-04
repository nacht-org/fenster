use fenster_core::prelude::*;
use fenster_glue::prelude::*;

fn main() {
    let response = http::send_request(Request {
        method: Method::Get,
        url: String::from("http://google.com"),
        params: None,
        data: None,
        headers: None,
    })
    .unwrap();

    println!("{response:?}");
}
