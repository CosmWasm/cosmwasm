use std::sync::Arc;

use tracing::warn;
use wasmer::{
    sys::{
        vm::{
            MemoryError, MemoryStyle, TableStyle, VMMemory, VMMemoryDefinition, VMTable,
            VMTableDefinition,
        },
        Tunables,
    },
    MemoryType, Pages, TableType,
};
use wasmer_engine::universal::Universal;
use wasmer_middlewares::{LinearMemory, MemoryStyle, Table, TableStyle, VMMemoryDefinition};

use crate::size::Size;

/// Enforces a maximum memory size for Wasm memory.
pub struct LimitingTunables {
    /// The parent tunables.
    parent: Universal,
    /// The maximum allowed memory size in bytes.
    memory_size: Size,
}

impl LimitingTunables {
    /// Creates a new instance with the given maximum memory size in bytes.
    pub fn new(parent: Universal, memory_size: Size) -> Self {
        Self {
            parent,
            memory_size,
        }
    }

    /// The maximum amount of pages the user can increase memory by.
    fn max_pages(&self) -> Pages {
        // Round the byte limit down to the nearest Wasm page size, as we can't have fractional pages.
        let pages_u64 = self.memory_size.0 / u64::from(wasmer_vm::WASM_PAGE_SIZE);
        // Ensure we don't exceed the maximum Pages
        let pages_u32 = u32::try_from(pages_u64).unwrap_or(u32::MAX);
        Pages(pages_u32)
    }
}

// We must implement this manually because derive(Clone) would require
// Universal to be Clone, which it is not.
impl Clone for LimitingTunables {
    fn clone(&self) -> Self {
        Self {
            parent: Universal::new(self.parent.target().clone()),
            memory_size: self.memory_size,
        }
    }
}

impl WasmerTunables for LimitingTunables {
    /// Creates a memory owned by the host given a [`MemoryType`].
    fn memory_style(&self, memory: &MemoryType) -> MemoryStyle {
        self.parent.memory_style(memory)
    }

    /// Construct a `VMMemory` for the provided [`MemoryType`].
    fn create_host_memory(
        &self,
        ty: &MemoryType,
        style: &MemoryStyle,
    ) -> Result<Arc<dyn LinearMemory>, MemoryError> {
        let max_pages = self.max_pages();

        // Enforce a maximum size for Wasm memories
        let limited_ty = MemoryType::new(
            ty.minimum,
            Some(std::cmp::min(ty.maximum.unwrap_or(max_pages), max_pages)),
            ty.shared,
        );
        self.parent.create_host_memory(&limited_ty, style)
    }

    /// Create a VM memory with the given memory definition.
    fn create_vm_memory(
        &self,
        ty: &MemoryType,
        style: &MemoryStyle,
        vm_definition: VMMemoryDefinition,
    ) -> Result<Arc<dyn LinearMemory>, MemoryError> {
        let max_pages = self.max_pages();

        // Enforce a maximum size for Wasm memories
        let limited_ty = MemoryType::new(
            ty.minimum,
            Some(std::cmp::min(ty.maximum.unwrap_or(max_pages), max_pages)),
            ty.shared,
        );
        self.parent
            .create_vm_memory(&limited_ty, style, vm_definition)
    }

    /// Memory grow operation for host memories.
    /// Returns `None` if memory can't be grown by the specified amount of pages.
    fn grow_host_memory(
        &self,
        memory: &mut dyn LinearMemory,
        delta_pages: Pages,
    ) -> Result<Pages, MemoryError> {
        let old_pages = memory.size();
        let new_pages = old_pages.checked_add(delta_pages).ok_or_else(|| {
            warn!(
                "Memory growth failed: growing by {} pages would overflow",
                delta_pages.0
            );
            MemoryError::CouldNotGrow {
                current: old_pages,
                attempted_delta: delta_pages,
            }
        })?;

        let max_pages = self.max_pages();
        if new_pages > max_pages {
            warn!(
                "Memory growth failed: new memory size of {} pages exceeds limit of {} pages",
                new_pages.0, max_pages.0
            );
            return Err(MemoryError::CouldNotGrow {
                current: old_pages,
                attempted_delta: delta_pages,
            });
        }

        self.parent.grow_host_memory(memory, delta_pages)
    }

    /// Memory grow operation for VM memories.
    /// Returns `None` if memory can't be grown by the specified amount of pages.
    fn grow_vm_memory(
        &self,
        memory: &mut dyn LinearMemory,
        delta_pages: Pages,
    ) -> Result<Pages, MemoryError> {
        let old_pages = memory.size();
        let new_pages = old_pages.checked_add(delta_pages).ok_or_else(|| {
            warn!(
                "Memory growth failed: growing by {} pages would overflow",
                delta_pages.0
            );
            MemoryError::CouldNotGrow {
                current: old_pages,
                attempted_delta: delta_pages,
            }
        })?;

        let max_pages = self.max_pages();
        if new_pages > max_pages {
            warn!(
                "Memory growth failed: new memory size of {} pages exceeds limit of {} pages",
                new_pages.0, max_pages.0
            );
            return Err(MemoryError::CouldNotGrow {
                current: old_pages,
                attempted_delta: delta_pages,
            });
        }

        self.parent.grow_vm_memory(memory, delta_pages)
    }

    /// Returns the table style for the specified [`TableType`].
    fn table_style(&self, table: &TableType) -> TableStyle {
        self.parent.table_style(table)
    }

    /// Create a table owned by the host from the given [`TableType`].
    fn create_host_table(&self, ty: &TableType) -> Result<Arc<dyn Table>, String> {
        self.parent.create_host_table(ty)
    }

    fn target(&self) -> &Target {
        self.parent.target()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::size::Size;
    use wasmer::{MemoryType, Pages, Target};
    use wasmer_engine::universal::Universal;

    #[test]
    fn create_host_memory_caps_memory_size() {
        let universal = Universal::new(Target::default());
        let tunables = LimitingTunables::new(universal, Size::mebi(64));
        let memory_ty = MemoryType::new(Pages(0), Some(Pages(10_000)), false);
        let memory_style = tunables.memory_style(&memory_ty);

        let memory = tunables
            .create_host_memory(&memory_ty, &memory_style)
            .expect("Memory creation failed");

        // We should be capped to 64 MiB of memory, regardless of the requested size
        assert!(memory.size() <= tunables.max_pages());
    }

    #[test]
    fn create_host_memory_allows_valid_size() {
        let universal = Universal::new(Target::default());
        let tunables = LimitingTunables::new(universal, Size::mebi(64));
        let memory_ty = MemoryType::new(Pages(10), Some(Pages(20)), false);
        let memory_style = tunables.memory_style(&memory_ty);

        let memory = tunables
            .create_host_memory(&memory_ty, &memory_style)
            .expect("Memory creation failed");

        // Should allow creation of memories that are smaller than the cap
        assert_eq!(memory.size(), Pages(10));
    }

    #[test]
    fn grow_host_memory_prevents_growing_beyond_limit() {
        let universal = Universal::new(Target::default());
        let limit = Size::mebi(1); // 1 MiB limit = 16 pages
        let tunables = LimitingTunables::new(universal, limit);
        let memory_ty = MemoryType::new(Pages(10), Some(Pages(20)), false);
        let memory_style = tunables.memory_style(&memory_ty);

        let mut memory = tunables
            .create_host_memory(&memory_ty, &memory_style)
            .expect("Memory creation failed");

        // Growth that would exceed the limit should fail
        let result = tunables.grow_host_memory(&mut *memory, Pages(10));
        assert!(result.is_err());
    }

    #[test]
    fn grow_host_memory_allows_valid_growth() {
        let universal = Universal::new(Target::default());
        let limit = Size::mebi(2); // 2 MiB limit = 32 pages
        let tunables = LimitingTunables::new(universal, limit);
        let memory_ty = MemoryType::new(Pages(10), Some(Pages(32)), false);
        let memory_style = tunables.memory_style(&memory_ty);

        let mut memory = tunables
            .create_host_memory(&memory_ty, &memory_style)
            .expect("Memory creation failed");

        // Growth within the limit should succeed
        let result = tunables.grow_host_memory(&mut *memory, Pages(5));
        assert!(result.is_ok());
        assert_eq!(memory.size(), Pages(15));
    }
}
