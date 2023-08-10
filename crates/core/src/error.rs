use std::{num::ParseIntError, str::Utf8Error};

use serde::{Deserialize, Serialize};

use crate::http::BoxedRequestError;

#[derive(Serialize, Deserialize, thiserror::Error, Debug)]
pub enum QuelleError {
    #[error("{0}")]
    RequestFailed(#[from] BoxedRequestError),

    #[error("filter verification failed: {0}")]
    FilterVerificationFailed(String),

    #[error("failed to decode utf-8 bytes")]
    Utf8Error,

    #[error("{0}")]
    ParseFailed(#[from] ParseError),

    #[error("{0}")]
    WasmAbiError(String),
}

#[derive(Serialize, Deserialize, thiserror::Error, Debug)]
pub enum ParseError {
    #[error("required element not found")]
    ElementNotFound,

    #[error("failed to serialize html tree to string")]
    SerializeFailed,

    #[error("failed to parse url")]
    FailedURLParse,

    #[error("failed to parse int from str")]
    ParseIntError,

    #[error("{0}")]
    Other(String),
}

impl ParseError {
    pub fn other<S: Into<String>>(s: S) -> Self {
        Self::Other(s.into())
    }
}

impl From<ParseIntError> for QuelleError {
    fn from(_: ParseIntError) -> Self {
        QuelleError::ParseFailed(ParseError::ParseIntError)
    }
}

impl From<Utf8Error> for QuelleError {
    fn from(_: Utf8Error) -> Self {
        QuelleError::Utf8Error
    }
}
