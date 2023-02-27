use std::{fmt::Display, string::FromUtf8Error};

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

    #[error("{0} is not supported by source extension")]
    NotSupported(AffectedFunction),

    #[error("failed to parse the result attemting to return")]
    FailedResultAttempt,

    #[error("{0}")]
    Utf8Error(#[from] FromUtf8Error),

    #[error("{0}")]
    Other(#[from] anyhow::Error),
}

#[derive(Debug)]
pub enum AffectedFunction {
    Search,
    Popular,
}

impl Display for AffectedFunction {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let value = match self {
            AffectedFunction::Search => "search",
            AffectedFunction::Popular => "popular",
        };

        write!(f, "{value}")
    }
}
