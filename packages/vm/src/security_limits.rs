//! This module centralizes security limits for the CosmWasm VM.
//! It provides constants and validation functions to enforce those limits.

/// The maximum number of bytes allowed for deserialization
pub const MAX_DESERIALIZATION_BYTES: usize = 512 * 1024; // 512 KiB

/// The maximum recursion depth allowed during JSON deserialization
pub const MAX_DESERIALIZATION_DEPTH: u32 = 64;

/// Default memory limit in WebAssembly pages (64 KiB per page)
pub const DEFAULT_MEMORY_LIMIT_PAGES: u32 = 512; // 32 MiB

/// Maximum memory growth in WebAssembly pages - prevents memory.grow from exceeding this value
pub const MAX_MEMORY_GROWTH_PAGES: u32 = 1024; // 64 MiB

/// Enforce a limit on memory.grow operations to prevent unbounded memory usage.
/// Returns true if the memory growth is allowed, false otherwise.
pub fn check_memory_growth(current_pages: u32, additional_pages: u32) -> bool {
    if additional_pages == 0 {
        return true;
    }

    // Check if the addition would overflow
    match current_pages.checked_add(additional_pages) {
        Some(new_size) => new_size <= MAX_MEMORY_GROWTH_PAGES,
        None => false, // Integer overflow - definitely not allowed
    }
}

/// Enforce a global limit on deserialization size.
pub fn enforce_deserialization_limit(size: usize, specified_limit: usize) -> usize {
    std::cmp::min(specified_limit, MAX_DESERIALIZATION_BYTES)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn check_memory_growth_enforces_limits() {
        // Zero growth is always allowed
        assert!(check_memory_growth(0, 0));
        assert!(check_memory_growth(100, 0));
        assert!(check_memory_growth(MAX_MEMORY_GROWTH_PAGES, 0));

        // Normal growth within limits
        assert!(check_memory_growth(0, 1));
        assert!(check_memory_growth(10, 10));
        assert!(check_memory_growth(100, 100));

        // Growth that reaches the limit exactly is allowed
        assert!(check_memory_growth(0, MAX_MEMORY_GROWTH_PAGES));
        assert!(check_memory_growth(
            MAX_MEMORY_GROWTH_PAGES / 2,
            MAX_MEMORY_GROWTH_PAGES / 2
        ));

        // Growth that exceeds the limit is rejected
        assert!(!check_memory_growth(0, MAX_MEMORY_GROWTH_PAGES + 1));
        assert!(!check_memory_growth(1, MAX_MEMORY_GROWTH_PAGES));
        assert!(!check_memory_growth(MAX_MEMORY_GROWTH_PAGES, 1));

        // Growth that would cause integer overflow is rejected
        assert!(!check_memory_growth(u32::MAX - 10, 11));
        assert!(!check_memory_growth(u32::MAX, 1));
    }

    #[test]
    fn enforce_deserialization_limit_respects_global_maximum() {
        // Small limits are preserved
        assert_eq!(enforce_deserialization_limit(100, 1000), 1000);

        // Larger limits are capped
        let big_limit = MAX_DESERIALIZATION_BYTES * 2;
        assert_eq!(
            enforce_deserialization_limit(100, big_limit),
            MAX_DESERIALIZATION_BYTES
        );
    }
}
