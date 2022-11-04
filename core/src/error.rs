use serde::{Deserialize, Serialize};

use crate::http::RequestError;

#[derive(Serialize, Deserialize, Debug)]
pub enum FensterError {
    RequestFailed(RequestError),
}

impl From<RequestError> for FensterError {
    fn from(error: RequestError) -> Self {
        FensterError::RequestFailed(error)
    }
}
