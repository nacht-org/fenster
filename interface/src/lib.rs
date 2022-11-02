use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Debug)]
pub struct Request {
    pub method: Method,
    pub url: String,
    pub params: Option<String>,
    pub data: Option<String>,
    pub headers: Option<String>,
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
pub struct Response {
    status: usize,
}
