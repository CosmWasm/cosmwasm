use wasmer::Module;

#[derive(Debug, Clone)]
pub struct CachedModule {
    pub module: Module,
    /// The estimated size of this element in memory
    pub size: usize,
}
