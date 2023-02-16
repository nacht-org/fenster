use fenster_core::prelude::FensterError;
use wasmtime::Trap;

pub type Result<T> = std::result::Result<T, Error>;

#[derive(thiserror::Error, Debug)]
pub enum Error {
    #[error("{0}")]
    ReturnedError(#[from] FensterError),

    #[error("{0}")]
    Trap(#[from] Trap),

    #[error("failed to deserialize returned value")]
    DeserializeError,

    #[error("{0}")]
    Other(#[from] anyhow::Error),
}
