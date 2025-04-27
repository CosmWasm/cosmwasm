#[cfg(test)]
mod security_tests {
    use crate::compatibility::check_wasm;
    use crate::config::WasmLimits;
    
    use crate::errors::RegionValidationError;
    use crate::memory::{test_validate_region, Region};
    use std::collections::HashSet;

    #[test]
    fn linear_gas_cost_handles_large_inputs() {
        // Create a LinearGasCost by directly testing the total_cost functionality
        // We'll use the read_region_small_cost from the GasConfig default values
        let gas_cost = crate::environment::GasConfig::default().read_region_small_cost;

        // Test using the public total_cost method
        // Normal cases work
        let small_result = gas_cost.total_cost(0);
        assert!(small_result.is_ok());
        let base_cost = small_result.unwrap();

        let ten_items_result = gas_cost.total_cost(10);
        assert!(ten_items_result.is_ok());
        let ten_items_cost = ten_items_result.unwrap();
        // Verify cost increases with more items
        assert!(ten_items_cost > base_cost);

        // Excessive inputs should fail (causing gas depletion)
        assert!(gas_cost.total_cost(u64::MAX).is_err());
        assert!(gas_cost.total_cost(u32::MAX as u64 + 1).is_err());
    }

    #[test]
    fn region_validation_prevents_overflow() {
        // Create a region with offset and capacity that would cause overflow
        let region = Region {
            offset: u32::MAX - 10,
            capacity: 20,
            length: 5,
        };

        // This should fail with an overflow error
        let result = test_validate_region(&region);
        assert!(result.is_err());
        match result.unwrap_err() {
            RegionValidationError::OutOfRange { .. } => {
                // This is the expected error
            }
            err => panic!("Unexpected error: {:?}", err),
        }
    }

    #[test]
    fn module_size_limits_are_enforced() {
        let wasm_code = vec![0u8; 5 * 1024 * 1024]; // 5 MiB

        // Default limits should reject this large module
        let mut limits = WasmLimits::default();
        limits.module_size_limit = Some(1024 * 1024); // 1 MiB

        let result = check_wasm(
            &wasm_code,
            &HashSet::new(),
            &limits,
            crate::compatibility::Logger::default(),
        );

        assert!(result.is_err());
        let error = result.unwrap_err();
        assert!(
            error.to_string().contains("module size"),
            "Expected error message about module size, got: {}",
            error
        );
    }
}
