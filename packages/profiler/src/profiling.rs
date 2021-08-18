use std::{
    path::Path,
    sync::{Arc, Mutex},
};

use crate::{
    instrumentation::{InstrumentedInstance, Module},
    measure::Measurements,
};

use wasmer::WasmerEnv;

pub struct ProfiledInstance {
    instance: InstrumentedInstance<ProfilingEnv>,
    measurements: Arc<Mutex<Measurements>>,
}

impl ProfiledInstance {
    pub fn from_path(path: &impl AsRef<Path>) -> Self {
        Self::new(Module::from_path(path))
    }

    pub fn from_bytes(bytes: &[u8]) -> Self {
        Self::new(Module::from_bytes(bytes))
    }

    fn new(module: Module) -> Self {
        let env = ProfilingEnv::default();
        let measurements = env.measurements.clone();

        let start_fn = |env: &ProfilingEnv, fn_num: u32, block_num: u32| {
            env.measurements
                .lock()
                .unwrap()
                .start_measurement(fn_num, block_num);
        };
        let take_fn = |env: &ProfilingEnv, fn_num: u32, block_num: u32, block_hash: u64| {
            env.measurements
                .lock()
                .unwrap()
                .take_measurement(fn_num, block_num, block_hash);
        };

        let instance = module.instrument(env, start_fn, take_fn);

        Self {
            instance,
            measurements,
        }
    }
}

#[derive(Clone, WasmerEnv, Default)]
pub struct ProfilingEnv {
    measurements: Arc<Mutex<Measurements>>,
}
