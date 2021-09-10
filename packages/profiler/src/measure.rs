use std::collections::{HashMap, VecDeque};
use std::sync::{Arc, Mutex};
use std::time;

use crate::code_blocks::{BlockId, BlockStore};
use crate::utils::InsertPush as _;

#[derive(wasmer::WasmerEnv, Default, Debug, Clone)]
pub struct Measurements {
    started: HashMap<(u32, u32), VecDeque<time::Instant>>,
    pub taken: HashMap<BlockId, VecDeque<time::Duration>>,
}

impl Measurements {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn start_measurement(&mut self, fn_index: u32, local_block_id: u32) {
        self.started
            .insert_push((fn_index, local_block_id), time::Instant::now());
    }

    // TODO: Error handling? This will be called from Wasm code probably.
    pub fn take_measurement(
        &mut self,
        fn_index: u32,
        local_block_id: u32,
        block_id: impl Into<BlockId>,
    ) {
        match self.started.get_mut(&(fn_index, local_block_id)) {
            Some(q) => {
                let start = q
                    .pop_front()
                    .expect("trying to finalize a measurement that was never started");
                self.taken.insert_push(block_id.into(), start.elapsed());
            }
            None => panic!("trying to finalize a measurement that was never started"),
        }
    }

    pub fn compile_csv(&self, block_store: Arc<Mutex<BlockStore>>, sink: impl std::io::Write) {
        let block_store = block_store.lock().unwrap();
        let mut wtr = csv::Writer::from_writer(sink);

        for (block_id, timings) in &self.taken {
            let block = format!("{:?}", block_store.get_block(*block_id).unwrap());
            wtr.write_record(
                vec![block]
                    .into_iter()
                    .chain(timings.into_iter().map(|t| t.as_nanos().to_string())),
            );
        }

        wtr.flush().unwrap();
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn take_measurements() {
        // TODO: This is probably very confusing. What's a good way to refactor?

        let mut measure = Measurements::new();

        measure.start_measurement(0, 0);
        measure.start_measurement(0, 1);
        std::thread::sleep(time::Duration::from_millis(100));
        measure.start_measurement(0, 0);
        measure.start_measurement(1, 0);

        measure.take_measurement(0, 0, 0);
        measure.take_measurement(0, 0, 0);
        measure.take_measurement(1, 0, 1);

        assert_eq!(measure.started[&(0, 0)].len(), 0);
        assert_eq!(measure.started[&(0, 1)].len(), 1);
        assert_eq!(measure.started[&(1, 0)].len(), 0);

        let ms0 = &measure.taken[&BlockId(0)];
        let ms1 = &measure.taken[&BlockId(1)];

        assert!(ms0[0] > time::Duration::from_millis(100));
        assert!(ms0[1] < time::Duration::from_millis(25));
        assert!(ms1[0] < time::Duration::from_millis(25));
    }
}
