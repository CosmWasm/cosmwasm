use std::collections::HashMap;
use std::time;

use crate::code_blocks::BlockId;

#[derive(Default, Debug)]
pub struct Measurements {
    pub data: Vec<Measurement>,
}

impl Measurements {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn start_measurement(&mut self) -> usize {
        self.data.push(Measurement::Started(time::Instant::now()));
        self.data.len() - 1
    }

    // TODO: Error handling? This will be called from Wasm code probably.
    pub fn take_measurement(&mut self, measurement_id: usize, block: impl Into<BlockId>) {
        // We're not sure if this id exists.
        if let Measurement::Started(start) = self.data[measurement_id] {
            self.data[measurement_id] = Measurement::Taken(start.elapsed(), block.into());
        }
    }
}

#[derive(Debug)]
pub enum Measurement {
    Started(time::Instant),
    Taken(time::Duration, BlockId),
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn take_measurements() {
        let mut measure = Measurements::new();

        let m_id1 = measure.start_measurement();
        let _m_id2 = measure.start_measurement();
        std::thread::sleep(time::Duration::from_millis(100));
        let m_id3 = measure.start_measurement();

        measure.take_measurement(m_id1, 0);
        measure.take_measurement(m_id3, 0);

        assert_eq!(measure.data.len(), 3);

        let mut measures = measure.data.iter();

        match measures.next().unwrap() {
            Measurement::Taken(duration, block) => {
                assert!(*duration > time::Duration::from_millis(100));
            }
            _ => panic!("failed to take measurement"),
        }
        if let Measurement::Taken(..) = measures.next().unwrap() {
            panic!("second measurement should be unfinished");
        }
        match measures.next().unwrap() {
            Measurement::Taken(duration, block) => {
                assert!(*duration < time::Duration::from_millis(25));
            }
            _ => panic!("failed to take measurement"),
        }
    }
}
