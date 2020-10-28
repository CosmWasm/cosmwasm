use std::ptr::NonNull;
use std::sync::Arc;

use wasmer::{
    vm::{self, MemoryError, MemoryStyle, TableStyle, VMMemoryDefinition, VMTableDefinition},
    MemoryType, Pages, TableType,
};
use wasmer_engine::Tunables;

/// A custom tunables that allows you to set a memory limit.
///
/// After adjusting the memory limits, it delegates all other logic
/// to the base tunables.
pub struct LimitingTunables<T: Tunables> {
    /// The maxium a linear memory is allowed to be (in Wasm pages, 65 KiB each).
    /// Since Wasmer ensures there is only none or one memory, this is practically
    /// an upper limit for the guest memory.
    limit: Pages,
    /// The base implementation we delegate all the logic to
    base: T,
}

impl<T: Tunables> LimitingTunables<T> {
    pub fn new(base: T, limit: Pages) -> Self {
        Self { limit, base }
    }

    /// Takes in input type as requested by the guest and returns a
    /// limited memory type that is ready for processing.
    fn adjust_memory(&self, requested: &MemoryType) -> Result<MemoryType, MemoryError> {
        if requested.minimum > self.limit {
            return Err(MemoryError::Generic(
                "Minimum of requested memory exceeds the allowed memory limit".to_string(),
            ));
        }

        if let Some(max) = requested.maximum {
            if max > self.limit {
                return Err(MemoryError::Generic(
                    "Maximum of requested memory exceeds the allowed memory limit".to_string(),
                ));
            }
        }

        let mut adjusted = requested.clone();
        // We know that if `requested.maximum` is set, it is less than or equal to `self.limit`.
        // So this sets maximum to (pseudo-code) `min(requested.maximum, self.limit)`
        adjusted.maximum = Some(requested.maximum.unwrap_or(self.limit));
        Ok(adjusted)
    }
}

impl<T: Tunables> Tunables for LimitingTunables<T> {
    /// Construct a `MemoryStyle` for the provided `MemoryType`
    ///
    /// Delegated to base.
    fn memory_style(&self, memory: &MemoryType) -> MemoryStyle {
        self.base.memory_style(memory)
    }

    /// Construct a `TableStyle` for the provided `TableType`
    ///
    /// Delegated to base.
    fn table_style(&self, table: &TableType) -> TableStyle {
        self.base.table_style(table)
    }

    /// Create a memory owned by the host given a [`MemoryType`] and a [`MemoryStyle`].
    ///
    /// The requested memory type is validated, adjusted to the limited and then passed to base.
    fn create_host_memory(
        &self,
        ty: &MemoryType,
        style: &MemoryStyle,
    ) -> Result<Arc<dyn vm::Memory>, MemoryError> {
        let adjusted = self.adjust_memory(ty)?;
        self.base.create_host_memory(&adjusted, style)
    }

    /// Create a memory owned by the VM given a [`MemoryType`] and a [`MemoryStyle`].
    ///
    /// Delegated to base.
    unsafe fn create_vm_memory(
        &self,
        ty: &MemoryType,
        style: &MemoryStyle,
        vm_definition_location: NonNull<VMMemoryDefinition>,
    ) -> Result<Arc<dyn vm::Memory>, MemoryError> {
        let adjusted = self.adjust_memory(ty)?;
        self.base
            .create_vm_memory(&adjusted, style, vm_definition_location)
    }

    /// Create a table owned by the host given a [`TableType`] and a [`TableStyle`].
    ///
    /// Delegated to base.
    fn create_host_table(
        &self,
        ty: &TableType,
        style: &TableStyle,
    ) -> Result<Arc<dyn vm::Table>, String> {
        self.base.create_host_table(ty, style)
    }

    /// Create a table owned by the VM given a [`TableType`] and a [`TableStyle`].
    ///
    /// Delegated to base.
    unsafe fn create_vm_table(
        &self,
        ty: &TableType,
        style: &TableStyle,
        vm_definition_location: NonNull<VMTableDefinition>,
    ) -> Result<Arc<dyn vm::Table>, String> {
        self.base.create_vm_table(ty, style, vm_definition_location)
    }
}

#[cfg(test)]
mod test {
    use super::*;
    use wasmer::{Target, Tunables as ReferenceTunables};

    #[test]
    fn adjust_memory_works() {
        let limit = Pages(12);
        let limiting =
            LimitingTunables::new(ReferenceTunables::for_target(&Target::default()), limit);

        // No maximum
        let requested = MemoryType::new(3, None, true);
        let adjusted = limiting.adjust_memory(&requested).unwrap();
        assert_eq!(adjusted, MemoryType::new(3, Some(12), true));

        // Maximum smaller than limit
        let requested = MemoryType::new(3, Some(7), true);
        let adjusted = limiting.adjust_memory(&requested).unwrap();
        assert_eq!(adjusted, MemoryType::new(3, Some(7), true));

        // Maximum equal to limit
        let requested = MemoryType::new(3, Some(12), true);
        let adjusted = limiting.adjust_memory(&requested).unwrap();
        assert_eq!(adjusted, MemoryType::new(3, Some(12), true));

        // Maximum greater than limit
        let requested = MemoryType::new(3, Some(20), true);
        let result = limiting.adjust_memory(&requested);
        match result.unwrap_err() {
            MemoryError::Generic(msg) => assert_eq!(
                msg,
                "Maximum of requested memory exceeds the allowed memory limit"
            ),
            err => panic!("Unexpected error: {:?}", err),
        }

        // Minimum greater than maximum (not our problem)
        let requested = MemoryType::new(5, Some(3), true);
        let adjusted = limiting.adjust_memory(&requested).unwrap();
        assert_eq!(adjusted, MemoryType::new(5, Some(3), true));

        // Minimum greater than limit
        let requested = MemoryType::new(20, Some(20), true);
        let result = limiting.adjust_memory(&requested);
        match result.unwrap_err() {
            MemoryError::Generic(msg) => assert_eq!(
                msg,
                "Minimum of requested memory exceeds the allowed memory limit"
            ),
            err => panic!("Unexpected error: {:?}", err),
        }
    }
}
