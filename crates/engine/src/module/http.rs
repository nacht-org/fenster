use std::str::FromStr;

use log::{debug, trace};
use quelle_core::prelude::{Request, RequestError, RequestErrorKind, Response};
use wasmtime::Caller;

use crate::{
    module::utils::{read_string, write_string},
    Data,
};

pub fn ext_send_request(mut caller: Caller<'_, Data>, ptr: i32) -> i32 {
    trace!("executing exposed function 'ext_send_request'");

    let memory = caller.get_export("memory").unwrap().into_memory().unwrap();

    let request = read_string(&mut caller, &memory, ptr);
    let request = serde_json::from_str::<Request>(request).unwrap();
    debug!("Sending http request: {request:?}.");

    let client = &caller.data().client;
    let response = client.execute(reqwest::blocking::Request::new(
        reqwest::Method::GET,
        reqwest::Url::from_str(&request.url).unwrap(),
    ));

    let response = parse_response(response);
    let json = serde_json::to_string(&response).unwrap();

    write_string(&mut caller, &memory, json.as_str())
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
