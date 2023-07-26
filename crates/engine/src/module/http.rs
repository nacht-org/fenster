use std::future::Future;

use log::{debug, trace};
use quelle_core::prelude::{Body, Request, RequestError, RequestErrorKind, Response};
use wasmtime::Caller;

use crate::{
    data::DefaultImpl,
    module::utils::{read_str_with_len, write_str},
};

pub fn send_request_noop<'a, D>(
    _caller: Caller<'a, D>,
    _ptr: i32,
    _len: i32,
) -> Box<dyn Future<Output = i32> + Send> {
    Box::new(async move { 0 })
}

pub fn send_request<'a>(
    mut caller: Caller<'a, DefaultImpl>,
    ptr: i32,
    len: i32,
) -> Box<dyn Future<Output = i32> + Send + 'a> {
    Box::new(async move {
        trace!("executing exposed function 'ext_send_request'");

        let memory = caller.get_export("memory").unwrap().into_memory().unwrap();

        let request_data = read_str_with_len(&mut caller, &memory, ptr, len as usize);
        let request_data = serde_json::from_str::<Request>(request_data).unwrap();
        debug!("Sending http request: {request_data:?}.");

        let client = &caller.data().client;
        let mut request = client.request(request_data.method.into(), &request_data.url);
        if let Some(body) = request_data.data {
            match body {
                Body::Form(data) => {
                    let mut multipart = reqwest::multipart::Form::new();
                    for (name, value) in data {
                        multipart = multipart.text(name, value);
                    }
                    request = request.multipart(multipart);
                }
            };
        }

        let response = parse_response(request.send().await).await;
        let json = serde_json::to_string(&response).unwrap();

        write_str(&mut caller, &memory, json.as_str()).await
    })
}

async fn parse_response(
    response: reqwest::Result<reqwest::Response>,
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
        body: response.bytes().await.map(|data| data.to_vec()).ok(),
        headers: Some(headers),
    })
}
