use std::{error, fs, future::Future, path::PathBuf};

use quelle_engine::module::{
    http::{parse_response, read_request, send_request_reqwest},
    utils::write_str,
};
use slug::slugify;
use wasmtime::Caller;

pub struct CachingImpl {
    client: reqwest::Client,
    cache: Cache,
}

impl CachingImpl {
    pub fn new() -> Self {
        Self {

            client: reqwest::Client::builder()
            .user_agent("Mozilla/5.0 (X11; Fedora; Linux x86_64; rv:107.0) Gecko/20100101 Firefox/107.0")
            .build()
            .unwrap(),
            cache: Cache { dir: PathBuf::from(".cache") }
        }
    }
}

pub fn send_request<'a>(
    mut caller: Caller<'a, CachingImpl>,
    ptr: i32,
    len: i32,
) -> Box<dyn Future<Output = i32> + Send + 'a> {
    Box::new(async move {
        let memory = caller.get_export("memory").unwrap().into_memory().unwrap();
        let request = read_request(&mut caller, ptr, len, &memory);

        let cache = caller.data().cache.get(&request.url).unwrap();
        let json = cache.map(|data| String::from_utf8(data).ok()).flatten();

        let json = if let Some(json) = json {
            json
        } else {
            let key = request.url.clone();
            let client = &caller.data().client;

            let response = send_request_reqwest::<CachingImpl>(client, request).await;
            let response = parse_response(response).await;

            let json = serde_json::to_string(&response).unwrap();
            let _ = caller.data().cache.put(&key, &json.as_bytes());
            json
        };

        write_str(&mut caller, &memory, json.as_str()).await
    })
}

pub struct Cache {
    dir: PathBuf,
}

impl Cache {
    pub fn put(&self, key: &str, value: &[u8]) -> Result<(), Box<dyn error::Error>> {
        let file = self.get_file_path(key);
        if let Some(parent) = file.parent() {
            if !file.exists() {
                fs::create_dir_all(parent)?;
            }
        }

        fs::write(file, value)?;
        Ok(())
    }

    pub fn get(&self, key: &str) -> Result<Option<Vec<u8>>, Box<dyn error::Error>> {
        let file = self.get_file_path(key);
        if file.exists() {
            let value = fs::read(file)?;
            Ok(Some(value))
        } else {
            Ok(None)
        }
    }

    pub fn get_file_path(&self, key: &str) -> PathBuf {
        let mut file = self.dir.join("files");
        file.push(slugify(key));
        file
    }
}
