use crate::error::QuerierError;

#[uniffi::export(callback_interface)]
pub trait Querier {
    fn query_external(&self, gas_limit: u64, request: Vec<u8>) -> Result<Vec<u8>, QuerierError>;
}
