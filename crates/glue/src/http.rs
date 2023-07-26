use quelle_core::prelude::*;

use crate::prelude::FromWasmAbi;

extern "C" {
    fn http_send_request(ptr: *const u8, len: u32) -> *mut u8;
}

pub fn send_request(request: Request) -> Result<Response, BoxedRequestError> {
    let req = serde_json::to_string(&request).map_err(|_| RequestError {
        kind: RequestErrorKind::Serial,
        url: Some(request.url.clone()),
        message: String::from("request serialization failed"),
    })?;

    let resp = unsafe {
        let ptr = http_send_request(req.as_ptr(), req.len() as u32);
        let resp = String::from_wasm_abi(ptr);

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
