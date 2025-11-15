/// Address Space Layout Randomization (ASLR) - Phase D
///
/// Randomizes memory layout to improve security by making
/// it harder to predict memory addresses for exploits.

use crate::security::random_u64;
use super::paging::PAGE_SIZE;

/// ASLR configuration
pub struct AslrConfig {
    /// Stack randomization range (28 bits = 256MB range)
    pub stack_random_bits: u32,
    /// Mmap randomization range (28 bits = 256MB range)
    pub mmap_random_bits: u32,
    /// Heap randomization range (24 bits = 16MB range)
    pub heap_random_bits: u32,
}

impl Default for AslrConfig {
    fn default() -> Self {
        Self {
            stack_random_bits: 28,  // 256MB range
            mmap_random_bits: 28,   // 256MB range
            heap_random_bits: 24,   // 16MB range
        }
    }
}

/// Base addresses for ASLR (without randomization)
pub const STACK_BASE: u64 = 0x0000_7FFF_0000_0000;
pub const MMAP_BASE: u64 = 0x0000_7000_0000_0000;
pub const HEAP_BASE: u64 = 0x0000_5555_5600_0000;

/// Get randomized stack top address
///
/// Randomizes the top of the user stack within a range to prevent
/// stack address prediction attacks.
pub fn randomize_stack_top(config: &AslrConfig) -> u64 {
    let random_offset = get_random_offset(config.stack_random_bits);

    // Align to page boundary
    let offset = random_offset & !(PAGE_SIZE as u64 - 1);

    // Stack grows down, so subtract offset from base
    STACK_BASE.wrapping_sub(offset)
}

/// Get randomized mmap base address
///
/// Randomizes the base address for mmap allocations to prevent
/// heap address prediction attacks.
pub fn randomize_mmap_base(config: &AslrConfig) -> u64 {
    let random_offset = get_random_offset(config.mmap_random_bits);

    // Align to page boundary
    let offset = random_offset & !(PAGE_SIZE as u64 - 1);

    // Add offset to base
    MMAP_BASE.wrapping_add(offset)
}

/// Get randomized heap start address
///
/// Randomizes the heap base address to prevent heap address prediction.
pub fn randomize_heap_start(config: &AslrConfig) -> u64 {
    let random_offset = get_random_offset(config.heap_random_bits);

    // Align to page boundary
    let offset = random_offset & !(PAGE_SIZE as u64 - 1);

    // Add offset to base
    HEAP_BASE.wrapping_add(offset)
}

/// Get random offset with specified number of random bits
fn get_random_offset(bits: u32) -> u64 {
    if bits == 0 {
        return 0;
    }

    // Get random value
    let random = random_u64();

    // Mask to specified number of bits
    let mask = (1u64 << bits) - 1;
    random & mask
}

/// Enable ASLR for new process
///
/// Returns randomized addresses for stack, heap, and mmap base.
pub fn randomize_address_space() -> (u64, u64, u64) {
    let config = AslrConfig::default();

    let stack_top = randomize_stack_top(&config);
    let heap_start = randomize_heap_start(&config);
    let mmap_base = randomize_mmap_base(&config);

    (stack_top, heap_start, mmap_base)
}

/// Check if ASLR is enabled (always enabled in Phase D)
pub fn is_aslr_enabled() -> bool {
    // In Phase D, ASLR is always enabled
    // Could be made configurable via sysctl in future
    true
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_randomize_addresses() {
        let (stack1, heap1, mmap1) = randomize_address_space();
        let (stack2, heap2, mmap2) = randomize_address_space();

        // Addresses should be different (with high probability)
        // Note: there's a tiny chance they could be the same
        assert!(
            stack1 != stack2 || heap1 != heap2 || mmap1 != mmap2,
            "ASLR should produce different addresses"
        );

        // Addresses should be page-aligned
        assert_eq!(stack1 % PAGE_SIZE as u64, 0, "Stack should be page-aligned");
        assert_eq!(heap1 % PAGE_SIZE as u64, 0, "Heap should be page-aligned");
        assert_eq!(mmap1 % PAGE_SIZE as u64, 0, "Mmap should be page-aligned");

        // Addresses should be within expected ranges
        assert!(stack1 < STACK_BASE, "Stack should be below base");
        assert!(heap1 >= HEAP_BASE, "Heap should be at or above base");
        assert!(mmap1 >= MMAP_BASE, "Mmap should be at or above base");
    }

    #[test]
    fn test_random_offset() {
        let offset1 = get_random_offset(16);
        let offset2 = get_random_offset(16);

        // Offsets should be within range
        assert!(offset1 < (1 << 16), "Offset should be within 16 bits");
        assert!(offset2 < (1 << 16), "Offset should be within 16 bits");

        // Zero bits should return zero
        assert_eq!(get_random_offset(0), 0, "Zero bits should give zero offset");
    }
}
