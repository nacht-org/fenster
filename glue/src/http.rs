use std::{ffi::CString, os::raw::c_char};

use fenster_core::prelude::*;

extern "C" {
    fn ext_send_request(ptr: *const c_char) -> *mut c_char;
}

pub fn send_request(request: Request) -> Result<Response, BoxedRequestError> {
    let req = serde_json::to_string(&request).map_err(|_| RequestError {
        kind: RequestErrorKind::Serial,
        url: Some(request.url.clone()),
        message: String::from("request serialization failed"),
    })?;

    let req = CString::new(req).unwrap();

    let resp = unsafe {
        let ptr = ext_send_request(req.as_ptr());
        let resp = CString::from_raw(ptr);
        let resp = resp.into_string().unwrap();

        println!("{resp}");

        let resp = serde_json::from_str::<Result<Response, RequestError>>(&resp).map_err(|_| {
            RequestError {
                kind: RequestErrorKind::Serial,
                url: Some(request.url.clone()),
                message: String::from("response serialization failed"),
            }
        })?;

        resp
    };

    resp.map_err(|e| e.into())
}

pub trait SendRequest {
    fn send(self) -> Result<Response, BoxedRequestError>;
}

impl SendRequest for Request {
    #[inline]
    fn send(self) -> Result<Response, BoxedRequestError> {
        send_request(self)
    }
}
