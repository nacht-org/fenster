use std::os::raw::c_char;

use serde::{Deserialize, Serialize};

extern "C" {
    fn ext_send_request(ptr: *const c_char) -> *const c_char;
}

pub fn send_request(request: Request) -> serde_json::Result<Response> {
    let request_string = serde_json::to_string(&request)?;
    println!("{request_string}");
    Ok(Response {})
}

#[derive(Serialize, Deserialize, Debug)]
pub struct Request {
    method: Method,
    url: String,
    params: String,
    data: String,
    headers: String,
}

#[derive(Serialize, Deserialize, Debug)]
pub enum Method {
    Get,
    Post,
    Put,
    Patch,
    Delete,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct Response {}
