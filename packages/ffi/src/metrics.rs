use std::collections::HashMap;

#[derive(uniffi::Record)]
pub struct Metrics {
    pub hits_pinned_memory_cache: u32,
    pub hits_memory_cache: u32,
    pub hits_fs_cache: u32,
    pub misses: u32,
    pub elements_pinned_memory_cache: u64,
    pub elements_memory_cache: u64,
    pub size_pinned_memory_cache: u64,
    pub size_memory_cache: u64,
}

#[derive(uniffi::Record)]
pub struct PerModuleMetrics {
    pub hits: u32,
    pub size: u64,
}

impl From<cosmwasm_vm::PerModuleMetrics> for PerModuleMetrics {
    fn from(value: cosmwasm_vm::PerModuleMetrics) -> Self {
        Self {
            hits: value.hits,
            size: value.size as u64,
        }
    }
}

#[derive(uniffi::Record)]
pub struct PinnedMetrics {
    pub per_module: HashMap<Vec<u8>, PerModuleMetrics>,
}
