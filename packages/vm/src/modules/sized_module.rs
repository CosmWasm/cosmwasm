use wasmer::{Engine, Module};

#[derive(Debug, Clone)]
pub struct CachedModule {
    pub engine: Engine,
    pub module: Module,
    /// The estimated size of this element in memory
    pub size: usize,
}
