#![forbid(unsafe_code)]

mod data;

#[cfg(feature = "epub")]
pub mod epub;

pub use data::{Bundle, Cover};
