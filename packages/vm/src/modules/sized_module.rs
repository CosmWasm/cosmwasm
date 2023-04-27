use wasmer::{Module, Store};

#[derive(Debug)]
pub struct CachedModule {
    pub store: Store,
    pub module: Module,
    /// The estimated size of this element in memory
    pub size: usize,
}
