pub mod error;

use error::Error;
use log::{debug, info, trace};
use quelle_core::prelude::*;
use reqwest::blocking::Client;
use serde::de::DeserializeOwned;
use std::{path::Path, slice, str::FromStr};
use wasmtime::*;

pub struct Data {
    client: Client,
}

pub fn ext_print(mut caller: Caller<'_, Data>, ptr: i32) {
    trace!("executing exposed function 'ext_print'");

    let memory = caller.get_export("memory").unwrap().into_memory().unwrap();
    let string = read_string(&mut caller, &memory, ptr);
    print!("{string}");
}

pub fn ext_eprint(mut caller: Caller<'_, Data>, ptr: i32) {
    trace!("executing exposed function 'ext_eprint'");

    let memory = caller.get_export("memory").unwrap().into_memory().unwrap();
    let string = read_string(&mut caller, &memory, ptr);
    eprint!("{string}");
}

pub fn ext_trace(mut caller: Caller<'_, Data>, ptr: i32) {
    trace!("executing exposed function 'ext_trace'");

    let memory = caller.get_export("memory").unwrap().into_memory().unwrap();
    let string = read_string(&mut caller, &memory, ptr);
    eprintln!("{string}");
}

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

fn read_string<'c, 'm>(caller: &'c mut Caller<'_, Data>, memory: &'m Memory, ptr: i32) -> &'m str {
    info!("reading string from wasm memory");

    let len = stack_pop(caller) as usize;
    debug!("retrieved byte length from stack: {len}");

    unsafe {
        let ptr = memory.data_ptr(&caller).offset(ptr as isize);
        let bytes = slice::from_raw_parts(ptr, len);
        std::str::from_utf8(bytes).unwrap()
    }
}

fn write_string<'c, 'm>(caller: &'c mut Caller<'_, Data>, memory: &'m Memory, value: &str) -> i32 {
    let alloc_func = caller.get_export("alloc").unwrap().into_func().unwrap();

    let ptr = alloc_func
        .typed::<i32, i32>(caller.as_context())
        .unwrap()
        .call(caller.as_context_mut(), value.len() as i32)
        .unwrap();

    stack_push(caller, value.len() as i32);

    memory
        .write(caller.as_context_mut(), ptr as usize, value.as_bytes())
        .unwrap();

    ptr
}

fn stack_push<'c, 'm>(caller: &'c mut Caller<'_, Data>, value: i32) {
    let push_fn = caller
        .get_export("stack_push")
        .unwrap()
        .into_func()
        .unwrap();

    push_fn
        .typed::<i32, ()>(&caller)
        .unwrap()
        .call(caller, value)
        .unwrap();
}

fn stack_pop<'c, 'm>(caller: &'c mut Caller<'_, Data>) -> i32 {
    let pop_fn = caller.get_export("stack_pop").unwrap().into_func().unwrap();

    let value = pop_fn
        .typed::<(), i32>(&caller)
        .unwrap()
        .call(caller, ())
        .unwrap();

    value
}

#[allow(dead_code)]
pub struct Runner {
    engine: Engine,
    module: Module,
    store: Store<Data>,
    instance: Instance,
    memory: Memory,
    functions: Functions,
}

struct Functions {
    // Memory
    alloc: TypedFunc<i32, i32>,
    dealloc: TypedFunc<(i32, i32), ()>,

    // Stack
    stack_push: TypedFunc<i32, ()>,
    stack_pop: TypedFunc<(), i32>,

    // Result
    last_result: TypedFunc<(), i32>,

    // User
    setup: Option<TypedFunc<(), ()>>,
    meta: TypedFunc<(), i32>,
    fetch_novel: TypedFunc<i32, i32>,
    fetch_chapter_content: TypedFunc<i32, i32>,
    text_search: Option<TypedFunc<(i32, i32), i32>>,
    popular_url: Option<TypedFunc<i32, i32>>,
    popular: Option<TypedFunc<i32, i32>>,
}

impl Runner {
    pub fn new(path: &Path) -> crate::error::Result<Self> {
        let engine = Engine::default();
        let mut linker: Linker<Data> = Linker::new(&engine);
        let module = Module::from_file(&engine, path)?;

        linker.func_wrap("env", "ext_send_request", ext_send_request)?;
        linker.func_wrap("env", "ext_print", ext_print)?;
        linker.func_wrap("env", "ext_eprint", ext_eprint)?;
        linker.func_wrap("env", "ext_trace", ext_trace)?;

        let data = Data {
            client: Client::builder()
                .user_agent("Mozilla/5.0 (X11; Fedora; Linux x86_64; rv:107.0) Gecko/20100101 Firefox/107.0")
                .build()
                .unwrap(),
        };

        let mut store = Store::new(&engine, data);

        let instance = linker.instantiate(&mut store, &module)?;
        let memory = instance
            .get_memory(&mut store, "memory")
            .ok_or(anyhow::format_err!("failed to find `memory` export"))?;

        macro_rules! get_func {
            ($name:literal) => {
                instance
                    .get_func(&mut store, $name)
                    .expect(r#"$name is not an exported function"#)
                    .typed(&store)?
            };
        }

        macro_rules! get_func_optional {
            ($name:literal) => {
                instance
                    .get_func(&mut store, $name)
                    .map(|f| f.typed(&store))
                    .transpose()?
            };
        }

        let functions = Functions {
            alloc: get_func!("alloc"),
            dealloc: get_func!("dealloc"),
            stack_push: get_func!("stack_push"),
            stack_pop: get_func!("stack_pop"),
            last_result: get_func!("last_result"),
            setup: get_func_optional!("setup"),
            meta: get_func!("meta"),
            fetch_novel: get_func!("fetch_novel"),
            fetch_chapter_content: get_func!("fetch_chapter_content"),
            text_search: get_func_optional!("query_search"),
            popular_url: get_func_optional!("popular_url"),
            popular: get_func_optional!("popular"),
        };

        Ok(Self {
            engine,
            module,
            store,
            instance,
            memory,
            functions,
        })
    }

    /// Call the wasm setup function if the function exists
    ///
    /// This is usually used during debugging to setup panic hooks
    pub fn setup(&mut self) -> crate::error::Result<()> {
        if let Some(func) = self.functions.setup.as_ref() {
            func.call(&mut self.store, ())?;
        }

        Ok(())
    }

    pub fn meta(&mut self) -> Result<Meta, crate::error::Error> {
        let ptr = self.functions.meta.call(&mut self.store, ())?;

        let bytes = self.read_bytes(ptr)?;
        let meta = serde_json::from_slice(bytes).map_err(|_| Error::DeserializeError)?;

        let len = bytes.len() as i32;
        self.dealloc_memory(ptr, len)?;

        Ok(meta)
    }

    pub fn meta_raw(&mut self) -> Result<String, crate::error::Error> {
        let ptr = self.functions.meta.call(&mut self.store, ())?;

        let bytes = self.read_bytes(ptr)?;
        let meta = String::from_utf8_lossy(bytes).to_string();

        let len = bytes.len() as i32;
        self.dealloc_memory(ptr, len)?;

        Ok(meta)
    }

    pub fn fetch_novel(&mut self, url: &str) -> crate::error::Result<Novel> {
        let iptr = self.write_string(url)?;
        let offset = self.functions.fetch_novel.call(&mut self.store, iptr)?;
        self.parse_result::<Novel, QuelleError>(offset)
    }

    pub fn fetch_chapter_content(&mut self, url: &str) -> crate::error::Result<String> {
        let iptr = self.write_string(url)?;
        let offset = self
            .functions
            .fetch_chapter_content
            .call(&mut self.store, iptr)?;

        self.parse_string_result::<QuelleError>(offset)
    }

    pub fn text_search_supported(&self) -> bool {
        self.functions.text_search.is_some()
    }

    pub fn text_search(&mut self, query: &str, page: i32) -> crate::error::Result<Vec<BasicNovel>> {
        if self.functions.text_search.is_none() {
            return Err(error::Error::NotSupported(error::AffectedFunction::Search));
        }

        let query_ptr = self.write_string(query)?;
        let offset = self
            .functions
            .text_search
            .unwrap()
            .call(&mut self.store, (query_ptr, page))?;

        self.parse_result::<Vec<BasicNovel>, QuelleError>(offset)
    }

    pub fn popular_supported(&self) -> bool {
        self.functions.popular.is_some()
    }

    pub fn popular_url(&mut self, page: i32) -> crate::error::Result<String> {
        if self.functions.popular_url.is_none() {
            return Err(error::Error::NotSupported(error::AffectedFunction::Popular));
        }

        let rptr = self
            .functions
            .popular_url
            .unwrap()
            .call(&mut self.store, page)?;

        let bytes = self.read_bytes(rptr)?;
        let string = String::from_utf8_lossy(bytes).to_string();

        let len = bytes.len() as i32;
        self.dealloc_memory(rptr, len)?;

        Ok(string)
    }

    pub fn popular(&mut self, page: i32) -> crate::error::Result<Vec<BasicNovel>> {
        if self.functions.popular.is_none() {
            return Err(error::Error::NotSupported(error::AffectedFunction::Popular));
        }

        let offset = self
            .functions
            .popular
            .unwrap()
            .call(&mut self.store, page)?;

        self.parse_result::<Vec<BasicNovel>, QuelleError>(offset)
    }

    fn read_bytes(&mut self, offset: i32) -> crate::error::Result<&[u8]> {
        let len = self.stack_pop()? as usize;
        self.read_bytes_with_len(offset, len)
    }

    fn read_bytes_with_len(&mut self, offset: i32, len: usize) -> crate::error::Result<&[u8]> {
        let value = unsafe {
            let ptr = self.memory.data_ptr(&self.store).offset(offset as isize);
            let bytes = slice::from_raw_parts(ptr, len);
            bytes
        };

        Ok(value)
    }

    fn parse_result<T, E>(&mut self, signed_len: i32) -> crate::error::Result<T>
    where
        T: serde::de::DeserializeOwned,
        E: serde::de::DeserializeOwned,
        crate::error::Error: From<E>,
    {
        match self.parse_option_result(signed_len) {
            Ok(None) => Err(error::Error::FailedResultAttempt),
            Ok(Some(v)) => Ok(v),
            Err(e) => Err(e),
        }
    }

    fn parse_option_result<T, E>(&mut self, signed_len: i32) -> error::Result<Option<T>>
    where
        T: serde::de::DeserializeOwned,
        E: serde::de::DeserializeOwned,
        crate::error::Error: From<E>,
    {
        info!("parsing Result<T, E> from a result with length: {signed_len}");

        if signed_len > 0 {
            self.with_result_bytes(signed_len as usize, |bytes| {
                serde_json::from_reader::<_, T>(bytes)
                    .map(|v| Some(v))
                    .map_err(|_| Error::DeserializeError.into())
            })
        } else if signed_len < 0 {
            self.parse_result_error(signed_len)
        } else {
            Ok(None)
        }
    }

    fn parse_string_result<E>(&mut self, signed_len: i32) -> error::Result<String>
    where
        E: DeserializeOwned,
        error::Error: From<E>,
    {
        info!("parsing Result<String, E> from a result with length: {signed_len}");

        if signed_len > 0 {
            self.with_result_bytes(signed_len as usize, |bytes| {
                String::from_utf8(bytes.to_vec()).map_err(|e| e.into())
            })
        } else if signed_len < 0 {
            self.parse_result_error(signed_len)
        } else {
            Ok(Default::default())
        }
    }

    fn parse_result_error<T, E>(&mut self, signed_len: i32) -> error::Result<T>
    where
        E: DeserializeOwned,
        error::Error: From<E>,
    {
        self.with_result_bytes((-signed_len) as usize, |bytes| {
            let err: Result<E, error::Error> =
                serde_json::from_reader::<_, E>(bytes).map_err(|_| Error::DeserializeError.into());

            match err {
                Ok(v) => Err(v.into()),
                Err(e) => Err(e),
            }
        })
    }

    fn with_result_bytes<T>(
        &mut self,
        len: usize,
        f: impl Fn(&[u8]) -> crate::error::Result<T>,
    ) -> crate::error::Result<T> {
        let offset = self.last_result()?;
        let bytes = self.read_bytes_with_len(offset, len)?;

        let out = f(bytes);

        let len = bytes.len() as i32;
        self.dealloc_memory(offset, len)?;

        out
    }

    fn write_string(&mut self, value: &str) -> crate::error::Result<i32> {
        // length of the string with trailing null byte
        let ptr = self.alloc_memory(value.len() as i32)?;
        self.stack_push(value.len() as i32)?;

        self.memory
            .write(&mut self.store, ptr as usize, value.as_bytes())
            .unwrap();

        Ok(ptr)
    }

    fn alloc_memory(&mut self, len: i32) -> crate::error::Result<i32> {
        self.functions
            .alloc
            .call(&mut self.store, len)
            .map_err(|e| e.into())
    }

    fn dealloc_memory(&mut self, ptr: i32, len: i32) -> crate::error::Result<()> {
        self.functions
            .dealloc
            .call(&mut self.store, (ptr, len))
            .map_err(|e| e.into())
    }

    fn stack_push(&mut self, size: i32) -> crate::error::Result<()> {
        self.functions
            .stack_push
            .call(&mut self.store, size)
            .map_err(|e| e.into())
    }

    fn stack_pop(&mut self) -> crate::error::Result<i32> {
        self.functions
            .stack_pop
            .call(&mut self.store, ())
            .map_err(|e| e.into())
    }

    fn last_result(&mut self) -> error::Result<i32> {
        self.functions
            .last_result
            .call(&mut self.store, ())
            .map_err(|e| e.into())
    }
}
