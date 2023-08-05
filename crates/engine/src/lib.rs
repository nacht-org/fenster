pub mod data;
pub mod error;
pub mod module;

use data::DefaultImpl;
use error::Error;
use quelle_core::prelude::*;
use serde::{de::DeserializeOwned, Serialize};
use std::{future::Future, path::Path, slice};
use wasmtime::*;

type SendRequestFn<D> =
    fn(caller: Caller<'_, D>, ptr: i32, len: i32) -> Box<dyn Future<Output = i32> + Send + '_>;

type LogFn<D> = fn(caller: Caller<'_, D>, ptr: i32, len: i32);

pub struct RuntimeBuilder<D> {
    send_request: Option<SendRequestFn<D>>,
    log: Option<LogFn<D>>,
}

impl<D> Default for RuntimeBuilder<D> {
    fn default() -> Self {
        Self {
            send_request: Default::default(),
            log: Default::default(),
        }
    }
}

impl<D: Send + 'static> RuntimeBuilder<D> {
    pub fn send_request(mut self, f: SendRequestFn<D>) -> Self {
        self.send_request = Some(f);
        self
    }

    pub fn log(mut self, f: LogFn<D>) -> Self {
        self.log = Some(f);
        self
    }

    pub async fn build(self, path: &Path, data: D) -> error::Result<Runtime<D>> {
        let mut config = Config::new();
        config.async_support(true);
        // config.consume_fuel(true);

        let engine = Engine::new(&config)?;
        let mut linker: Linker<D> = Linker::new(&engine);
        let module = Module::from_file(&engine, path)?;

        let send_request = self.send_request.unwrap_or(module::http::send_request_noop);
        linker.func_wrap2_async("env", "http_send_request", send_request)?;

        let log_event = self.log.unwrap_or(module::log::event);
        linker.func_wrap("env", "log_event", log_event)?;

        linker.func_wrap("env", "io_print", module::io::print)?;
        linker.func_wrap("env", "io_eprint", module::io::eprint)?;
        linker.func_wrap("env", "io_trace", module::io::trace)?;

        let mut store = Store::new(&engine, data);

        let instance = linker.instantiate_async(&mut store, &module).await?;
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
            setup_default: get_func!("setup_default"),
            meta: get_func!("meta"),
            fetch_novel: get_func!("fetch_novel"),
            fetch_chapter_content: get_func!("fetch_chapter_content"),
            popular_url: get_func_optional!("popular_url"),
            popular: get_func_optional!("popular"),
            text_search: get_func_optional!("text_search"),
            filter_options: get_func_optional!("filter_options"),
            filter_search_url: get_func_optional!("filter_search_url"),
            filter_search: get_func_optional!("filter_search"),
        };

        Ok(Runtime {
            engine,
            module,
            store,
            instance,
            memory,
            functions,
        })
    }
}

#[allow(dead_code)]
pub struct Runtime<D> {
    engine: Engine,
    module: Module,
    store: Store<D>,
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

    // Extension
    setup: Option<TypedFunc<i32, ()>>,
    setup_default: TypedFunc<i32, ()>,

    meta: TypedFunc<(), i32>,

    fetch_novel: TypedFunc<i32, i32>,
    fetch_chapter_content: TypedFunc<i32, i32>,

    popular_url: Option<TypedFunc<i32, i32>>,
    popular: Option<TypedFunc<i32, i32>>,

    text_search: Option<TypedFunc<(i32, i32), i32>>,
    filter_options: Option<TypedFunc<(), i32>>,
    filter_search_url: Option<TypedFunc<(i32, i32), i32>>,
    filter_search: Option<TypedFunc<(i32, i32), i32>>,
}

impl Runtime<DefaultImpl> {
    pub async fn new(path: &Path) -> crate::error::Result<Self> {
        let data = DefaultImpl {
            client: reqwest::Client::builder()
                .user_agent("Mozilla/5.0 (X11; Fedora; Linux x86_64; rv:107.0) Gecko/20100101 Firefox/107.0")
                .build()
                .unwrap(),
        };

        RuntimeBuilder::default()
            .send_request(module::http::send_request)
            .build(path, data)
            .await
    }
}

impl<D> Runtime<D>
where
    D: Send,
{
    #[inline]
    pub fn builder() -> RuntimeBuilder<D> {
        RuntimeBuilder::default()
    }

    /// Call the extension's setup function
    pub async fn setup(&mut self, config: &ExtensionConfig) -> crate::error::Result<()> {
        let config = self.write_serialize(config).await?;

        self.functions
            .setup
            .unwrap_or(self.functions.setup_default)
            .call_async(&mut self.store, config)
            .await?;

        Ok(())
    }

    pub async fn meta(&mut self) -> Result<Meta, crate::error::Error> {
        let memloc = unsafe { self.meta_memloc().await? };
        let bytes = self.read_bytes_with_len(memloc.offset, memloc.len as usize);
        let meta = serde_json::from_slice(bytes).map_err(|_| Error::DeserializeError);
        self.dealloc_memory(memloc.offset, memloc.len).await?;
        meta
    }

    pub async unsafe fn meta_memloc(&mut self) -> error::Result<MemLoc> {
        let offset = self.functions.meta.call_async(&mut self.store, ()).await?;
        let len = self.stack_pop().await?;
        let ptr = self.memory.data_ptr(&self.store).offset(offset as isize);
        Ok(MemLoc { offset, ptr, len })
    }

    pub async fn fetch_novel(&mut self, url: &str) -> crate::error::Result<Novel> {
        let iptr = self.write_string(url).await?;
        let signed_len = self
            .functions
            .fetch_novel
            .call_async(&mut self.store, iptr)
            .await?;
        self.parse_result::<Novel, QuelleError>(signed_len).await
    }

    pub async unsafe fn fetch_novel_memloc(&mut self, url: &str) -> error::Result<MemLoc> {
        let iptr = self.write_string(url).await?;
        let len = self
            .functions
            .fetch_novel
            .call_async(&mut self.store, iptr)
            .await?;
        let offset = self
            .functions
            .last_result
            .call_async(&mut self.store, ())
            .await?;
        let ptr = self.memory.data_ptr(&self.store).offset(offset as isize);
        Ok(MemLoc { offset, ptr, len })
    }

    pub async fn fetch_chapter_content(&mut self, url: &str) -> error::Result<Content> {
        let iptr = self.write_string(url).await?;
        let offset = self
            .functions
            .fetch_chapter_content
            .call_async(&mut self.store, iptr)
            .await?;

        self.parse_result::<Content, QuelleError>(offset).await
    }

    pub async unsafe fn fetch_chapter_content_memloc(
        &mut self,
        url: &str,
    ) -> error::Result<MemLoc> {
        let iptr = self.write_string(url).await?;
        let len = self
            .functions
            .fetch_chapter_content
            .call_async(&mut self.store, iptr)
            .await?;
        let offset = self
            .functions
            .last_result
            .call_async(&mut self.store, ())
            .await?;
        let ptr = self.memory.data_ptr(&self.store).offset(offset as isize);
        Ok(MemLoc { offset, ptr, len })
    }

    pub fn popular_supported(&self) -> bool {
        self.functions.popular.is_some()
    }

    pub async fn popular_url(&mut self, page: i32) -> crate::error::Result<String> {
        if let Some(popular_url) = self.functions.popular_url {
            let offset = popular_url.call_async(&mut self.store, page).await?;
            let bytes = self.read_bytes(offset).await?;
            let string = String::from_utf8_lossy(bytes).to_string();

            let len = bytes.len() as i32;
            self.dealloc_memory(offset, len).await?;

            Ok(string)
        } else {
            Err(error::Error::NotSupported(error::AffectedFunction::Popular))
        }
    }

    pub async unsafe fn popular_url_memloc(&mut self, page: i32) -> error::Result<MemLoc> {
        if let Some(popular_url) = self.functions.popular_url {
            let offset = popular_url.call_async(&mut self.store, page).await?;
            let len = self.stack_pop().await?;
            let ptr = self.memory.data_ptr(&self.store).offset(offset as isize);
            Ok(MemLoc { offset, ptr, len })
        } else {
            Err(error::Error::NotSupported(error::AffectedFunction::Popular))
        }
    }

    async fn call_popular(&mut self, page: i32) -> error::Result<i32> {
        if let Some(popular) = self.functions.popular {
            popular
                .call_async(&mut self.store, page)
                .await
                .map_err(|e| e.into())
        } else {
            Err(error::Error::NotSupported(error::AffectedFunction::Popular))
        }
    }

    pub async fn popular(&mut self, page: i32) -> error::Result<Vec<BasicNovel>> {
        let signed_len = self.call_popular(page).await?;
        self.parse_result::<Vec<BasicNovel>, QuelleError>(signed_len)
            .await
    }

    pub async unsafe fn popular_memloc(&mut self, page: i32) -> error::Result<MemLoc> {
        let len = self.call_popular(page).await?;
        let offset = self
            .functions
            .last_result
            .call_async(&mut self.store, ())
            .await?;
        let ptr = self.memory.data_ptr(&self.store).offset(offset as isize);
        Ok(MemLoc { offset, ptr, len })
    }

    // --------------------------------------------------------------------------------
    // Text search
    // --------------------------------------------------------------------------------

    pub fn text_search_supported(&self) -> bool {
        self.functions.text_search.is_some()
    }

    async fn call_text_search(&mut self, query: &str, page: i32) -> crate::error::Result<i32> {
        if let Some(text_search) = self.functions.text_search {
            let query_ptr = self.write_string(query).await?;
            let signed_len = text_search
                .call_async(&mut self.store, (query_ptr, page))
                .await?;
            Ok(signed_len)
        } else {
            Err(error::Error::NotSupported(error::AffectedFunction::Search))
        }
    }

    pub async fn text_search(
        &mut self,
        query: &str,
        page: i32,
    ) -> crate::error::Result<Vec<BasicNovel>> {
        let signed_len = self.call_text_search(query, page).await?;
        self.parse_result::<Vec<BasicNovel>, QuelleError>(signed_len)
            .await
    }

    pub async unsafe fn text_search_memloc(
        &mut self,
        query: &str,
        page: i32,
    ) -> error::Result<MemLoc> {
        let len = self.call_text_search(query, page).await?;
        let offset = self
            .functions
            .last_result
            .call_async(&mut self.store, ())
            .await?;
        let ptr = self.memory.data_ptr(&self.store).offset(offset as isize);
        Ok(MemLoc { offset, ptr, len })
    }

    // --------------------------------------------------------------------------------
    // Filter search
    // --------------------------------------------------------------------------------

    pub fn filter_search_supported(&self) -> bool {
        self.functions.filter_options.is_some()
    }

    pub async fn filter_options(&mut self) -> error::Result<FieldMap> {
        let Some(filter_options) = self.functions.filter_options
            else { return Err(error::Error::NotSupported(error::AffectedFunction::Search)) };

        let offset = filter_options.call_async(&mut self.store, ()).await?;
        let len = self.stack_pop().await?;
        let bytes = self.read_bytes_with_len(offset, len as usize);
        let options = serde_json::from_slice(bytes).map_err(|_| Error::DeserializeError);
        self.dealloc_memory(offset, len).await?;
        options
    }

    pub async fn filter_search_url(&mut self, params: &str, page: i32) -> error::Result<String> {
        let Some(filter_search_url) = self.functions.filter_search_url
            else { return Err(error::Error::NotSupported(error::AffectedFunction::Search)) };

        let params_ptr = self.write_string(params).await?;
        let len = filter_search_url
            .call_async(&mut self.store, (params_ptr, page))
            .await?;

        self.parse_string_result::<QuelleError>(len).await
    }

    pub async fn filter_search(
        &mut self,
        params: &str,
        page: i32,
    ) -> error::Result<Vec<BasicNovel>> {
        let Some(filter_search) = self.functions.filter_search
            else { return Err(error::Error::NotSupported(error::AffectedFunction::Search)) };

        let params_ptr = self.write_string(params).await?;
        let len = filter_search
            .call_async(&mut self.store, (params_ptr, page))
            .await?;

        self.parse_result::<Vec<BasicNovel>, QuelleError>(len).await
    }

    // --------------------------------------------------------------------------------
    // Helpers
    // --------------------------------------------------------------------------------

    async fn read_bytes(&mut self, offset: i32) -> crate::error::Result<&[u8]> {
        let len = self.stack_pop().await? as usize;
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

    async fn parse_result<T, E>(&mut self, signed_len: i32) -> crate::error::Result<T>
    where
        T: serde::de::DeserializeOwned,
        E: serde::de::DeserializeOwned + Into<error::Error>,
    {
        match self.parse_option_result::<T, E>(signed_len).await {
            Ok(None) => Err(error::Error::FailedResultAttempt),
            Ok(Some(v)) => Ok(v),
            Err(e) => Err(e),
        }
    }

    async fn parse_option_result<T, E>(&mut self, signed_len: i32) -> error::Result<Option<T>>
    where
        T: serde::de::DeserializeOwned,
        E: serde::de::DeserializeOwned + Into<error::Error>,
    {
        log::debug!("parsing Result<T, E> from a result with length: {signed_len}");

        if signed_len > 0 {
            self.with_result_bytes(signed_len as usize, |bytes| {
                serde_json::from_reader::<_, T>(bytes)
                    .map(|v| Some(v))
                    .map_err(|_| Error::DeserializeError.into())
            })
            .await
        } else if signed_len < 0 {
            self.parse_result_error::<Option<T>, E>(signed_len).await
        } else {
            Ok(None)
        }
    }

    async fn parse_string_result<E>(&mut self, signed_len: i32) -> error::Result<String>
    where
        E: DeserializeOwned + Into<error::Error>,
    {
        log::debug!("parsing Result<String, E> from a result with length: {signed_len}");

        if signed_len > 0 {
            self.with_result_bytes(signed_len as usize, |bytes| {
                String::from_utf8(bytes.to_vec()).map_err(|e| e.into())
            })
            .await
        } else if signed_len < 0 {
            self.parse_result_error::<String, E>(signed_len).await
        } else {
            Ok(Default::default())
        }
    }

    async fn parse_result_error<T, E>(&mut self, signed_len: i32) -> error::Result<T>
    where
        E: DeserializeOwned + Into<error::Error>,
    {
        self.with_result_bytes((-signed_len) as usize, |bytes| {
            let err: Result<E, error::Error> =
                serde_json::from_reader::<_, E>(bytes).map_err(|_| Error::DeserializeError.into());

            match err {
                Ok(v) => Err(v.into()),
                Err(e) => Err(e),
            }
        })
        .await
    }

    async fn with_result_bytes<T>(
        &mut self,
        len: usize,
        f: impl Fn(&[u8]) -> crate::error::Result<T>,
    ) -> crate::error::Result<T> {
        let offset = self.last_result().await?;
        let bytes = self.read_bytes_with_len(offset, len);

        let out = f(bytes);

        let len = bytes.len() as i32;
        self.dealloc_memory(offset, len).await?;

        out
    }

    async fn write_serialize<T>(&mut self, value: &T) -> crate::error::Result<i32>
    where
        T: Serialize,
    {
        let string = serde_json::to_string(value).map_err(|_| Error::SerializeError)?;
        return self.write_string(&string).await;
    }

    async fn write_string(&mut self, value: &str) -> crate::error::Result<i32> {
        let ptr = self.alloc_memory(value.len() as i32).await?;
        self.stack_push(value.len() as i32).await?;

        self.memory
            .write(&mut self.store, ptr as usize, value.as_bytes())
            .map_err(|_| Error::MemoryAccessError)?;

        Ok(ptr)
    }

    async fn alloc_memory(&mut self, len: i32) -> crate::error::Result<i32> {
        self.functions
            .alloc
            .call_async(&mut self.store, len)
            .await
            .map_err(|e| e.into())
    }

    pub async fn dealloc_memory(&mut self, ptr: i32, len: i32) -> crate::error::Result<()> {
        self.functions
            .dealloc
            .call_async(&mut self.store, (ptr, len))
            .await
            .map_err(|e| e.into())
    }

    async fn stack_push(&mut self, size: i32) -> crate::error::Result<()> {
        self.functions
            .stack_push
            .call_async(&mut self.store, size)
            .await
            .map_err(|e| e.into())
    }

    async fn stack_pop(&mut self) -> crate::error::Result<i32> {
        self.functions
            .stack_pop
            .call_async(&mut self.store, ())
            .await
            .map_err(|e| e.into())
    }

    async fn last_result(&mut self) -> error::Result<i32> {
        self.functions
            .last_result
            .call_async(&mut self.store, ())
            .await
            .map_err(|e| e.into())
    }
}

#[derive(Debug)]
pub struct MemLoc {
    pub offset: i32,
    pub ptr: *mut u8,
    pub len: i32,
}
