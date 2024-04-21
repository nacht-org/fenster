use bindings::exports::quelle::http::main;

#[allow(warnings)]
mod bindings;

pub struct Reqwest;

impl main::Guest for Reqwest {
    type Client = ReqwestClient;
}

pub struct ReqwestClient {
    client: reqwest::blocking::Client,
}

impl main::GuestClient for ReqwestClient {
    fn request(&self, request: main::Request) -> Result<main::Response, main::ResponseError> {
        let mut builder = self.client.request(request.method.into(), request.url);

        if let Some(params) = request.params {
            builder = builder.query(&params);
        }

        if let Some(body) = request.data {
            match body {
                main::RequestBody::Form(data) => {
                    let multipart = create_multipart(data);
                    builder = builder.multipart(multipart);
                }
            };
        }

        builder.send().map(Into::into).map_err(Into::into)
    }
}

fn create_multipart(data: Vec<(String, main::FormPart)>) -> reqwest::blocking::multipart::Form {
    let mut form = reqwest::blocking::multipart::Form::new();
    for (name, part) in data {
        match part {
            main::FormPart::Text(value) => form = form.text(name, value),
            main::FormPart::Data(data) => {
                let mut part = reqwest::blocking::multipart::Part::bytes(data.data);

                if let Some(name) = data.name {
                    part = part.file_name(name);
                }

                if let Some(content_type) = data.content_type {
                    part = part.mime_str(&content_type).unwrap();
                }

                form = form.part(name, part);
            }
        };
    }

    form
}

impl From<main::Method> for reqwest::Method {
    fn from(value: main::Method) -> Self {
        match value {
            main::Method::Get => reqwest::Method::GET,
            main::Method::Post => reqwest::Method::POST,
            main::Method::Put => reqwest::Method::PUT,
            main::Method::Delete => reqwest::Method::DELETE,
            main::Method::Patch => reqwest::Method::PATCH,
            main::Method::Head => reqwest::Method::HEAD,
            main::Method::Options => reqwest::Method::OPTIONS,
        }
    }
}

impl From<reqwest::blocking::Response> for main::Response {
    fn from(value: reqwest::blocking::Response) -> Self {
        let status = value.status().as_u16();
        let headers = value
            .headers()
            .into_iter()
            .map(|(k, v)| (k.to_string(), v.to_str().unwrap().to_string()))
            .collect::<Vec<_>>();

        let data = value.bytes().unwrap().to_vec();

        main::Response {
            status,
            headers: Some(headers),
            data: Some(data),
        }
    }
}

impl From<reqwest::Error> for main::ResponseError {
    fn from(value: reqwest::Error) -> Self {
        main::ResponseError {
            kind: main::ResponseErrorKind::BadResponse,
            status: value.status().map(|v| v.as_u16()),
            response: None,
            message: value.to_string(),
        }
    }
}
