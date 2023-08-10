use std::collections::HashMap;

use serde::{Deserialize, Serialize};
use serde_json::Value;

#[derive(Serialize, Deserialize, Debug)]
pub struct Request {
    pub method: Method,
    pub url: String,
    pub params: Option<String>,
    pub data: Option<Body>,
    pub headers: Option<String>,
}

#[derive(Serialize, Deserialize, Debug)]
pub enum Body {
    Form(HashMap<String, String>),
}

impl Request {
    pub fn new(method: Method, url: String) -> Self {
        Request {
            method,
            url,
            params: None,
            data: None,
            headers: None,
        }
    }

    #[inline]
    pub fn get(url: String) -> Self {
        Request::new(Method::Get, url)
    }

    #[inline]
    pub fn post(url: String) -> Self {
        Request::new(Method::Post, url)
    }

    #[inline]
    pub fn form(mut self, value: HashMap<String, String>) -> Self {
        self.data = Some(Body::Form(value));
        self
    }

    pub fn json_params(mut self, value: &Value) -> Result<Self, serde_json::Error> {
        let params = serde_json::to_string(value)?;
        self.params = Some(params);
        Ok(self)
    }

    pub fn json_headers(mut self, value: &Value) -> Result<Self, serde_json::Error> {
        let headers = serde_json::to_string(value)?;
        self.headers = Some(headers);
        Ok(self)
    }
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
    pub body: Option<Vec<u8>>,
    pub headers: Option<String>,
}

impl Response {
    pub fn text(&self) -> Result<Option<&str>, std::str::Utf8Error> {
        self.body
            .as_ref()
            .map(|body| std::str::from_utf8(body))
            .transpose()
    }
}

#[derive(Serialize, Deserialize, thiserror::Error, Debug)]
pub struct BoxedRequestError(Box<RequestError>);

#[derive(Serialize, Deserialize, Debug)]
pub struct RequestError {
    pub kind: RequestErrorKind,
    pub url: Option<String>,
    pub message: String,
}

impl std::fmt::Display for BoxedRequestError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.0)
    }
}

impl std::fmt::Display for RequestError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        writeln!(f, "[{:?}] {:?}: {}", self.kind, self.url, self.message)?;
        Ok(())
    }
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
impl From<Method> for reqwest::Method {
    fn from(value: Method) -> Self {
        match value {
            Method::Get => reqwest::Method::GET,
            Method::Post => reqwest::Method::POST,
            Method::Put => reqwest::Method::PUT,
            Method::Patch => reqwest::Method::PATCH,
            Method::Delete => reqwest::Method::DELETE,
        }
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
