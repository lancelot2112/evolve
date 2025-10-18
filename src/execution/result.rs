//! Execution Result
//!
//! Represents the outcome of executing a DNA sequence.

/// Result of executing DNA
#[derive(Debug, Clone)]
pub struct ExecutionResult {
    /// Output produced
    pub output: Vec<i64>,
    /// Number of instructions executed (including template expansions)
    pub instruction_count: usize,
    /// Maximum stack depth reached
    pub max_stack_depth: usize,
    /// Whether execution completed successfully
    pub success: bool,
    /// Error message if execution failed
    pub error: Option<String>,
}
