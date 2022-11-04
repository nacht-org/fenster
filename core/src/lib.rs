pub mod ext;
pub mod mem;

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
    pub status: usize,
    pub body: Option<String>,
    pub headers: Option<String>,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct BoxedRequestError(Box<RequestError>);

#[derive(Serialize, Deserialize, Debug)]
pub struct RequestError {
    pub kind: RequestErrorKind,
    pub url: Option<String>,
    pub message: String,
}

#[derive(Serialize, Deserialize, Debug)]
pub enum RequestErrorKind {
    Serial,
    Request,
    Redirect,
    Status(u16),
    Body,
    Timeout,
    Unknown,
}

impl From<RequestError> for BoxedRequestError {
    fn from(inner: RequestError) -> Self {
        BoxedRequestError(Box::new(inner))
    }
}

#[cfg(feature = "reqwest")]
impl From<reqwest::Error> for RequestError {
    fn from(error: reqwest::Error) -> Self {
        let url = error.url().map(|u| u.as_str().to_string());
        let message = error.to_string();

        let kind = if error.is_timeout() {
            RequestErrorKind::Timeout
        } else if error.is_decode() || error.is_body() {
            RequestErrorKind::Body
        } else if error.is_redirect() {
            RequestErrorKind::Redirect
        } else if error.is_request() {
            RequestErrorKind::Request
        } else if error.is_status() {
            RequestErrorKind::Status(error.status().unwrap_or_default().as_u16())
        } else {
            RequestErrorKind::Unknown
        };

        RequestError { kind, url, message }
    }
}
