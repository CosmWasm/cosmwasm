use wasmer::{Module, Store};

#[derive(Debug)]
pub struct SizedModule {
    pub store: Store,
    pub module: Module,
    pub size: usize,
}
