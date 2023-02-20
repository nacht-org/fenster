use quelle_core::prelude::QuelleError;
use wasmtime::Trap;

pub type Result<T> = std::result::Result<T, Error>;

#[derive(thiserror::Error, Debug)]
pub enum Error {
    #[error("{0}")]
    ReturnedError(#[from] QuelleError),

    #[error("{0}")]
    Trap(#[from] Trap),

    #[error("failed to deserialize returned value")]
    DeserializeError,

    #[error("{0}")]
    Other(#[from] anyhow::Error),
}
