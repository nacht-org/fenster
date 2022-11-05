use serde::{Deserialize, Serialize};

use crate::http::BoxedRequestError;

#[derive(Serialize, Deserialize, Debug)]
pub enum FensterError {
    RequestFailed(BoxedRequestError),
}

impl From<BoxedRequestError> for FensterError {
    fn from(error: BoxedRequestError) -> Self {
        FensterError::RequestFailed(error)
    }
}
