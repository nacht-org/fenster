use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Debug)]
pub enum ExposeError {
    SerializeError,
    DeserializeError,
}
