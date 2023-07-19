use quelle_core::config::ExtensionConfig;

use crate::{
    logger::Logger,
    prelude::{set_panic_hook, FromWasmAbi},
};

#[no_mangle]
pub fn setup_default(config: *mut u8) {
    #[cfg(debug_assertions)]
    set_panic_hook();

    let config = ExtensionConfig::from_wasm_abi(config);
    init_extension(&config);
}

pub fn init_extension(config: &ExtensionConfig) {
    Logger::new(config.level_filter).init();
}
