use bootstrap::http::send_request;
use interface::{Method, Request};

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
