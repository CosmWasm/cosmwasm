use std::fmt;

use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use snafu::ResultExt;

use crate::errors::{Base64Err, Result};

#[derive(Serialize, Deserialize, Clone, Default, Debug, PartialEq, JsonSchema)]
pub struct Base64(pub String);

impl Base64 {
    // as_bytes will return a &[u8] reference to the string format. This should be good
    // for most apps (slightly longer, but saves the transform cost)
    pub fn as_bytes(&self) -> &[u8] {
        self.0.as_bytes()
    }
    // decode will return the underlying bytes after decoding base64
    pub fn decode(&self) -> Result<Vec<u8>> {
        base64::decode(&self.0).context(Base64Err {})
    }
    // encode will construct this from raw binary (output of decode)
    pub fn encode(data: &[u8]) -> Self {
        Base64(base64::encode(data))
    }
    pub fn len(&self) -> usize {
        self.0.len()
    }
    pub fn is_empty(&self) -> bool {
        self.0.is_empty()
    }
}

impl fmt::Display for Base64 {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}", &self.0)
    }
}

impl From<&str> for Base64 {
    fn from(data: &str) -> Self {
        Base64(data.to_string())
    }
}

impl From<String> for Base64 {
    fn from(data: String) -> Self {
        Base64(data)
    }
}

impl From<&Base64> for Base64 {
    fn from(data: &Base64) -> Self {
        Base64(data.0.to_string())
    }
}
