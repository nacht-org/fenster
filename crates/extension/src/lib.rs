mod bindings;
mod wasm;

use quelle_core::data::{Content, Meta, Novel};

pub struct ExtensionOptions {}

pub trait ExtensionMeta {
    fn info() -> Meta;
    fn setup(options: ExtensionOptions) -> Result<(), String>;
}

pub struct SourceOptions {}

pub trait ExtensionSource {
    fn new(options: SourceOptions) -> Self;
    fn novel_info(&self, url: &str) -> Result<Novel, String>;
    fn chapter_content(&self, url: &str) -> Result<Content, String>;
}
