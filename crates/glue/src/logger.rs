use crate::out::println;
use log::{LevelFilter, Log, Metadata, Record};
use quelle_core::log::LogEvent;

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
            println!("{e}");
        }
    }
}

impl Log for Logger {
    fn enabled(&self, metadata: &Metadata) -> bool {
        metadata.level().to_level_filter() <= self.filter
    }

    fn log(&self, record: &Record) {
        if self.enabled(&record.metadata()) {
            let bytes = serde_json::to_vec(&LogEvent::from(record)).unwrap();
            unsafe { log_event(bytes.as_ptr(), bytes.len()) };
        }
    }

    fn flush(&self) {}
}
