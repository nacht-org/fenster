use quelle_core::prelude::*;

use crate::prelude::{FromWasmAbi, ToWasmAbi};

#[link(name = "http")]
extern "C" {
    fn ext_send_request(ptr: *mut u8) -> *mut u8;
}

pub fn send_request(request: Request) -> Result<Response, BoxedRequestError> {
    let req = serde_json::to_string(&request).map_err(|_| RequestError {
        kind: RequestErrorKind::Serial,
        url: Some(request.url.clone()),
        message: String::from("request serialization failed"),
    })?;

    let resp = unsafe {
        let ptr = ext_send_request(req.to_wasm_abi());
        let resp = String::from_wasm_abi(ptr);

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
