use crate::{error::DbError, iterator::Iterator};

#[derive(uniffi::Enum)]
pub enum DbOrder {
    Ascending,
    Descending,
}

#[uniffi::export(callback_interface)]
pub trait Db {
    fn read_db(&self, key: Vec<u8>) -> Result<Vec<u8>, DbError>;

    fn write_db(&self, key: Vec<u8>, value: Vec<u8>) -> Result<(), DbError>;

    fn remove_db(&self, key: Vec<u8>) -> Result<(), DbError>;

    fn scan_db(
        &self,
        start: Vec<u8>,
        end: Vec<u8>,
        order: DbOrder,
    ) -> Result<Box<dyn Iterator>, DbError>;
}
