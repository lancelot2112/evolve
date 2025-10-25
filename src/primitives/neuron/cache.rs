//! Cache Line Size Detection
//!
//! Detects the CPU cache line size from the OS for optimal batch processing.

use std::fs;

/// Default cache line size in bytes (fallback if detection fails)
pub const DEFAULT_CACHE_LINE_BYTES: usize = 64;

/// Detect cache line size from OS
///
/// On Linux: reads from /sys/devices/system/cpu/cpu0/cache/index0/coherency_line_size
/// On other platforms: uses default of 64 bytes
///
/// Returns cache line size in bytes
pub fn detect_cache_line_size() -> usize {
    #[cfg(target_os = "linux")]
    {
        // Try to read from sysfs
        let paths = [
            "/sys/devices/system/cpu/cpu0/cache/index0/coherency_line_size",
            "/sys/devices/system/cpu/cpu0/cache/index1/coherency_line_size",
            "/sys/devices/system/cpu/cpu0/cache/index2/coherency_line_size",
        ];

        for path in &paths {
            if let Ok(contents) = fs::read_to_string(path) {
                if let Ok(size) = contents.trim().parse::<usize>() {
                    if size > 0 && size <= 256 {
                        // Sanity check: cache line between 1-256 bytes
                        return size;
                    }
                }
            }
        }
    }

    // Fallback to default
    DEFAULT_CACHE_LINE_BYTES
}

/// Get cache line size in number of i64 values
///
/// Since compartments are stored as i64 (8 bytes), this returns
/// how many compartments fit in one cache line.
pub fn cache_line_size_i64() -> usize {
    detect_cache_line_size() / std::mem::size_of::<i64>()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_detect_cache_line_size() {
        let size = detect_cache_line_size();
        assert!(size > 0, "Cache line size should be positive");
        assert!(
            size <= 256,
            "Cache line size should be reasonable (<=256 bytes)"
        );
        // Most common sizes are 64 or 128 bytes
        assert!(
            size == 64 || size == 128 || size == 32 || size == 256,
            "Unexpected cache line size: {}",
            size
        );
    }

    #[test]
    fn test_cache_line_i64() {
        let size = cache_line_size_i64();
        assert!(size > 0);
        // 64 bytes / 8 bytes per i64 = 8 i64 values
        // 128 bytes / 8 bytes per i64 = 16 i64 values
        println!("Cache line size: {} i64 values", size);
    }
}
