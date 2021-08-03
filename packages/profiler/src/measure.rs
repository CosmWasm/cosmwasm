use std::collections::HashMap;
use std::time;

use crate::code_blocks::BlockId;

#[derive(Default, Debug)]
pub struct Measurements {
    data: Vec<Measurement>,
}

impl Measurements {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn start_measurement(&mut self) -> MeasurementId {
        todo!()
    }

    pub fn take_measurement(&mut self, id: impl Into<MeasurementId>, block: BlockId) {
        todo!()
    }
}

#[derive(Debug)]
pub enum Measurement {
    Started(time::Instant),
    Taken(time::Duration, BlockId),
}

pub struct MeasurementId(u32);

impl From<u32> for MeasurementId {
    fn from(hash: u32) -> Self {
        Self(hash)
    }
}

impl PartialEq<u32> for MeasurementId {
    fn eq(&self, rhs: &u32) -> bool {
        self.0 == *rhs
    }
}

impl PartialEq<MeasurementId> for u32 {
    fn eq(&self, rhs: &MeasurementId) -> bool {
        rhs.0 == *self
    }
}
