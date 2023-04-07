use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Debug)]
pub struct LogEvent<'a> {
    pub level: log::Level,
    pub args: String,
    pub module_path: Option<&'a str>,
    pub file: Option<&'a str>,
    pub line: Option<u32>,
}

impl<'a> From<&'a log::Record<'a>> for LogEvent<'a> {
    fn from(value: &'a log::Record<'a>) -> Self {
        LogEvent {
            level: value.level(),
            args: value.args().to_string(),
            module_path: value.module_path(),
            file: value.file(),
            line: value.line(),
        }
    }
}
