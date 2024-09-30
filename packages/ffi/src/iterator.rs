use crate::error::IteratorError;

#[derive(uniffi::Record)]
pub struct IteratorEntry {
    pub key: Vec<u8>,
    pub value: Vec<u8>,
}

#[uniffi::export(callback_interface)]
pub trait Iterator {
    fn next(&self) -> Result<IteratorEntry, IteratorError>;
    fn next_key(&self) -> Result<Vec<u8>, IteratorError>;
    fn next_value(&self) -> Result<Vec<u8>, IteratorError>;
}
