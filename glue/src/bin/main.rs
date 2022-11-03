use fenster_core::{Method, Request};
use fenster_glue::http::send_request;

fn main() {
    let response = send_request(Request {
        method: Method::Get,
        url: String::from("http://google.com"),
        params: None,
        data: None,
        headers: None,
    })
    .unwrap();

    println!("{response:?}");
}
