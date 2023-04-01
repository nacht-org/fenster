use std::str::FromStr;

use log::{debug, trace};
use quelle_core::prelude::{Body, Request, RequestError, RequestErrorKind, Response};
use wasmtime::Caller;

use crate::{
    module::utils::{read_str, write_str},
    Data,
};

pub fn send_request(mut caller: Caller<'_, Data>, ptr: i32) -> i32 {
    trace!("executing exposed function 'ext_send_request'");

    let memory = caller.get_export("memory").unwrap().into_memory().unwrap();

    let request_data = read_str(&mut caller, &memory, ptr);
    let request_data = serde_json::from_str::<Request>(request_data).unwrap();
    debug!("Sending http request: {request_data:?}.");

    let client = &caller.data().client;
    let mut request = client.request(request_data.method.into(), &request_data.url);
    if let Some(body) = request_data.data {
        match body {
            Body::Form(data) => {
                let mut multipart = reqwest::blocking::multipart::Form::new();
                for (name, value) in data {
                    multipart = multipart.text(name, value);
                }
                request = request.multipart(multipart);
            }
        };
    }

    let response = parse_response(request.send());
    let json = serde_json::to_string(&response).unwrap();

    write_str(&mut caller, &memory, json.as_str())
}

fn parse_response(
    response: reqwest::Result<reqwest::blocking::Response>,
) -> Result<Response, RequestError> {
    let response = response?;
    let header_map = response
        .headers()
        .into_iter()
        .map(|(n, v)| (n.to_string(), v.to_str().unwrap_or_default().to_string()))
        .collect::<std::collections::HashMap<_, _>>();

    let headers = serde_json::to_string(&header_map).map_err(|_| RequestError {
        kind: RequestErrorKind::Serial,
        url: Some(response.url().as_str().to_string()),
        message: String::from("failed to serialize response"),
    })?;

    Ok(Response {
        status: response.status().as_u16() as usize,
        body: response.bytes().map(|data| data.to_vec()).ok(),
        headers: Some(headers),
    })
}
