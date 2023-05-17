use crate::Size;
use wasmer::{Engine, Module};

#[derive(Debug, Clone)]
pub struct CachedModule {
    pub engine: Engine,
    pub module: Module,
    pub store_memory_limit: Option<Size>,
    /// The estimated size of this element in memory
    pub size: usize,
}
