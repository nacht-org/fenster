use crate::out::println;
use log::{error, LevelFilter, Log, Metadata, Record};
use quelle_core::log::LogEvent;

use crate::prelude::FromWasmAbi;

extern "C" {
    fn log_event(ptr: *const u8, len: usize);
}

pub struct Logger {
    filter: LevelFilter,
}

impl Logger {
    pub fn new(filter: LevelFilter) -> Self {
        Logger { filter }
    }

    pub fn init(self) {
        log::set_max_level(self.filter);
        if let Err(e) = log::set_boxed_logger(Box::new(self)) {
            error!("{e}");
        }
    }
}

impl Log for Logger {
    fn enabled(&self, metadata: &Metadata) -> bool {
        metadata.level().to_level_filter() <= self.filter
    }

    fn log(&self, record: &Record) {
        println!("{record:?}");
        if self.enabled(&record.metadata()) {
            let bytes = serde_json::to_vec(&LogEvent::from(record)).unwrap();
            unsafe { log_event(bytes.as_ptr(), bytes.len()) };
        }
    }

    fn flush(&self) {}
}

impl FromWasmAbi for LevelFilter {
    type Type = i32;

    fn from_wasm_abi(value: Self::Type) -> Self {
        match value {
            ..=0 => LevelFilter::Off,
            1 => LevelFilter::Error,
            2 => LevelFilter::Warn,
            3 => LevelFilter::Info,
            4 => LevelFilter::Debug,
            5.. => LevelFilter::Trace,
        }
    }
}
