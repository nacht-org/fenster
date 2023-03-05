pub mod error;
mod module;

use error::Error;
use log::info;
use quelle_core::prelude::*;
use reqwest::blocking::Client;
use serde::de::DeserializeOwned;
use std::{path::Path, slice};
use wasmtime::*;

pub struct Data {
    client: Client,
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

        linker.func_wrap("env", "http_send_request", module::ext_send_request)?;
        linker.func_wrap("env", "io_print", module::ext_print)?;
        linker.func_wrap("env", "io_eprint", module::ext_eprint)?;
        linker.func_wrap("env", "io_trace", module::ext_trace)?;

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
            text_search: get_func_optional!("text_search"),
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

    pub fn meta_raw(&mut self) -> error::Result<String> {
        let ptr = self.functions.meta.call(&mut self.store, ())?;

        let bytes = self.read_bytes(ptr)?;
        let meta = String::from_utf8_lossy(bytes).to_string();

        let len = bytes.len() as i32;
        self.dealloc_memory(ptr, len)?;

        Ok(meta)
    }

    pub fn fetch_novel(&mut self, url: &str) -> crate::error::Result<Novel> {
        let iptr = self.write_string(url)?;
        let signed_len = self.functions.fetch_novel.call(&mut self.store, iptr)?;
        self.parse_result::<Novel, QuelleError>(signed_len)
    }

    pub fn fetch_novel_raw(&mut self, url: &str) -> error::Result<String> {
        let iptr = self.write_string(url)?;
        let signed_len = self.functions.fetch_novel.call(&mut self.store, iptr)?;
        self.parse_string_result::<QuelleError>(signed_len)
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

    fn call_text_search(&mut self, query: &str, page: i32) -> crate::error::Result<i32> {
        if let Some(text_search) = self.functions.text_search {
            let query_ptr = self.write_string(query)?;
            let signed_len = text_search.call(&mut self.store, (query_ptr, page))?;
            Ok(signed_len)
        } else {
            Err(error::Error::NotSupported(error::AffectedFunction::Search))
        }
    }

    pub fn text_search(&mut self, query: &str, page: i32) -> crate::error::Result<Vec<BasicNovel>> {
        let signed_len = self.call_text_search(query, page)?;
        self.parse_result::<Vec<BasicNovel>, QuelleError>(signed_len)
    }

    pub fn text_search_raw(&mut self, query: &str, page: i32) -> error::Result<String> {
        let signed_len = self.call_text_search(query, page)?;
        self.parse_string_result::<QuelleError>(signed_len)
    }

    pub fn popular_supported(&self) -> bool {
        self.functions.popular.is_some()
    }

    pub fn popular_url(&mut self, page: i32) -> crate::error::Result<String> {
        if let Some(popular_url) = self.functions.popular_url {
            let rptr = popular_url.call(&mut self.store, page)?;
            let bytes = self.read_bytes(rptr)?;
            let string = String::from_utf8_lossy(bytes).to_string();

            let len = bytes.len() as i32;
            self.dealloc_memory(rptr, len)?;

            Ok(string)
        } else {
            Err(error::Error::NotSupported(error::AffectedFunction::Popular))
        }
    }

    fn call_popular(&mut self, page: i32) -> error::Result<i32> {
        if let Some(popular) = self.functions.popular {
            popular.call(&mut self.store, page).map_err(|e| e.into())
        } else {
            Err(error::Error::NotSupported(error::AffectedFunction::Popular))
        }
    }

    pub fn popular(&mut self, page: i32) -> crate::error::Result<Vec<BasicNovel>> {
        let offset = self.call_popular(page)?;
        self.parse_result::<Vec<BasicNovel>, QuelleError>(offset)
    }

    fn read_bytes(&mut self, offset: i32) -> crate::error::Result<&[u8]> {
        let len = self.stack_pop()? as usize;
        let bytes = self.read_bytes_with_len(offset, len);
        Ok(bytes)
    }

    fn read_bytes_with_len(&self, offset: i32, len: usize) -> &[u8] {
        unsafe {
            let ptr = self.memory.data_ptr(&self.store).offset(offset as isize);
            let bytes = slice::from_raw_parts(ptr, len);
            bytes
        }
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
        let bytes = self.read_bytes_with_len(offset, len);

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
