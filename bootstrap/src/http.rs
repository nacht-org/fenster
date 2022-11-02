use std::{ffi::CString, os::raw::c_char};

use interface::{Request, Response};

extern "C" {
    fn ext_send_request(ptr: *const c_char) -> *mut c_char;
}

pub fn send_request(request: Request) -> serde_json::Result<Response> {
    let req = serde_json::to_string(&request)?;
    let req = CString::new(req).unwrap();

    let resp = unsafe {
        let ptr = ext_send_request(req.as_ptr());
        let resp = CString::from_raw(ptr);
        let resp = resp.into_string().unwrap();
        let resp = serde_json::from_str::<Response>(&resp)?;
        resp
    };

    Ok(resp)
}
