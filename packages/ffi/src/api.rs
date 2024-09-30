use crate::error::ApiError;

#[uniffi::export(callback_interface)]
pub trait Api {
    fn humanize_address(&self, input: Vec<u8>) -> Result<String, ApiError>;
    fn canonicalize_address(&self, input: Vec<u8>) -> Result<Vec<u8>, ApiError>;
    fn validate_address(&self, input: Vec<u8>) -> Result<(), ApiError>;
}
