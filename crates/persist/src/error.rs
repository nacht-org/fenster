use std::io;

pub type PersistResult<T> = Result<T, PersistError>;

#[derive(thiserror::Error, Debug)]
pub enum PersistError {
    #[error("failed to serialize or deserialize object")]
    SerializationError,

    #[error("{0}")]
    IO(#[from] io::Error),
}

impl From<serde_json::Error> for PersistError {
    fn from(_: serde_json::Error) -> Self {
        PersistError::SerializationError
    }
}
