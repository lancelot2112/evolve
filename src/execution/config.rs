//! Execution Configuration
//!
//! Defines safety limits and resource constraints for DNA execution.

/// Configuration for execution environment
#[derive(Debug, Clone)]
pub struct ExecutionConfig {
    /// Number of registers available
    pub register_count: usize,
    /// Maximum instruction count (prevents infinite loops)
    pub max_instructions: usize,
    /// Maximum stack depth
    pub max_stack_depth: usize,
    /// Maximum template recursion depth
    pub max_recursion_depth: usize,
}

impl Default for ExecutionConfig {
    fn default() -> Self {
        Self {
            register_count: 8,        // R0-R7
            max_instructions: 10_000, // Reasonable limit
            max_stack_depth: 1_000,   // Deep enough for most uses
            max_recursion_depth: 100, // Prevent infinite template recursion
        }
    }
}
