use std::{
    error,
    ffi::{CStr, CString},
    str::FromStr,
};

use fenster_core::prelude::*;
use wasmtime::{
    AsContext, AsContextMut, Caller, Engine, Instance, Linker, Memory, Module, Store,
    StoreContextMut, TypedFunc,
};

pub fn ext_print(mut caller: Caller<'_, ()>, ptr: i32) {
    println!("print called");

    let memory = caller.get_export("memory").unwrap().into_memory().unwrap();
    unsafe {
        let ptr = memory.data_ptr(&caller).offset(ptr as isize);
        let bytes = CStr::from_ptr(ptr as *const i8);
        let string = bytes.clone().to_str().unwrap();
        print!("{string}");
    }
}

pub fn ext_eprint(mut caller: Caller<'_, ()>, ptr: i32) {
    let memory = caller.get_export("memory").unwrap().into_memory().unwrap();
    unsafe {
        let ptr = memory.data_ptr(&caller).offset(ptr as isize);
        let bytes = CStr::from_ptr(ptr as *const i8);
        let string = bytes.clone().to_str().unwrap();
        eprint!("{string}");
    }
}

pub fn ext_trace(mut caller: Caller<'_, ()>, ptr: i32) {
    let memory = caller.get_export("memory").unwrap().into_memory().unwrap();
    unsafe {
        let ptr = memory.data_ptr(&caller).offset(ptr as isize);
        let bytes = CStr::from_ptr(ptr as *const i8);
        let string = bytes.clone().to_str().unwrap();
        eprintln!("{string}");
    }
}

pub fn ext_send_request(mut caller: Caller<'_, ()>, ptr: i32) -> i32 {
    let memory = caller.get_export("memory").unwrap().into_memory().unwrap();

    let request = read_string(&mut caller.as_context_mut(), &memory, ptr);
    println!("{request:?}");
    let request = serde_json::from_str::<Request>(request).unwrap();
    println!("{request:?}");

    let client = reqwest::blocking::Client::new();
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
        body: response.text().ok(),
        headers: Some(headers),
    })
}

fn read_string<'c, 'm>(
    store: &'c mut StoreContextMut<'_, ()>,
    memory: &'m Memory,
    ptr: i32,
) -> &'m str {
    unsafe {
        let ptr = memory.data_ptr(&store).offset(ptr as isize);
        let cstr = CStr::from_ptr(ptr as *const i8);
        let str = cstr.to_str().unwrap();
        str
    }
}

fn write_string<'c, 'm>(caller: &'c mut Caller<'_, ()>, memory: &'m Memory, value: &str) -> i32 {
    let alloc_func = caller.get_export("alloc").unwrap().into_func().unwrap();

    let ptr = alloc_func
        .typed::<i32, i32, _>(caller.as_context())
        .expect("Failed to parse func type")
        .call(caller.as_context_mut(), value.len() as i32)
        .expect("Failed while calling alloc");

    memory
        .write(caller.as_context_mut(), ptr as usize, value.as_bytes())
        .unwrap();

    ptr
}

#[allow(dead_code)]
pub struct Runner {
    engine: Engine,
    module: Module,
    store: Store<()>,
    instance: Instance,
    memory: Memory,
    functions: Functions,
}

struct Functions {
    meta: TypedFunc<(), i32>,
    fetch_novel: TypedFunc<i32, i32>,
}

impl Runner {
    pub fn new(path: &str) -> Result<Self, Box<dyn error::Error>> {
        let engine = Engine::default();
        let mut linker = Linker::new(&engine);
        let module = Module::from_file(&engine, path)?;

        linker.func_wrap("env", "ext_send_request", ext_send_request)?;
        linker.func_wrap("env", "ext_print", ext_print)?;
        linker.func_wrap("env", "ext_eprint", ext_eprint)?;
        linker.func_wrap("env", "ext_trace", ext_trace)?;

        let mut store = Store::new(&engine, ());

        let instance = linker.instantiate(&mut store, &module)?;
        let memory = instance
            .get_memory(&mut store, "memory")
            .ok_or(anyhow::format_err!("failed to find `memory` export"))?;

        let functions = Functions {
            meta: instance
                .get_func(&mut store, "meta")
                .expect("'meta' is not an exported function")
                .typed::<(), i32, _>(&store)
                .unwrap(),
            fetch_novel: instance
                .get_func(&mut store, "fetch_novel")
                .expect("'fetch_novel' is not an exported function")
                .typed::<i32, i32, _>(&store)
                .unwrap(),
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

    pub fn meta(&mut self) -> Result<(), Box<dyn error::Error>> {
        let ptr = self.functions.meta.call(&mut self.store, ())?;

        let r = read_string(&mut self.store.as_context_mut(), &self.memory, ptr);
        println!("{r}");
        self.dealloc_string(ptr)?;

        Ok(())
    }

    pub fn fetch_novel(&mut self, url: &str) -> Result<(), Box<dyn error::Error>> {
        let iptr = self.write_string(url)?;
        let rptr = self.functions.fetch_novel.call(&mut self.store, iptr)?;

        let r = read_string(&mut self.store.as_context_mut(), &self.memory, rptr);
        println!("{r}");
        self.dealloc_string(rptr)?;

        Ok(())
    }

    fn write_string(&mut self, value: &str) -> Result<i32, Box<dyn error::Error>> {
        let string = CString::new(value).unwrap();

        // length of the string with trailing null byte
        let ptr = self.alloc_memory((value.len() + 1) as i32)?;

        self.memory
            .write(&mut self.store, ptr as usize, string.as_bytes_with_nul())
            .unwrap();

        Ok(ptr)
    }

    fn alloc_memory(&mut self, len: i32) -> Result<i32, Box<dyn error::Error>> {
        let alloc = self.instance.get_func(&mut self.store, "alloc").unwrap();

        let ptr = alloc
            .typed::<i32, i32, _>(&self.store)?
            .call(&mut self.store, len)?;

        Ok(ptr)
    }

    fn dealloc_string(&mut self, ptr: i32) -> Result<(), Box<dyn error::Error>> {
        let dealloc = self
            .instance
            .get_func(&mut self.store, "dealloc_string")
            .unwrap();

        dealloc
            .typed::<i32, (), _>(&self.store)?
            .call(&mut self.store, ptr)?;

        Ok(())
    }
}
