//! Primitive Trait Definition
//!
//! Defines the core Primitive trait that all operations must implement.
//!
//! Primitives are the fundamental building blocks that users define for their
//! problem domain. Each primitive takes arguments and modifies the execution
//! context (registers, stack, I/O) according to its specific behavior.

use super::context::{ExecutionContext, ExecutionError};
use crate::dna::Argument;

/// Trait for primitive operations
pub trait Primitive: Send + Sync {
    /// Execute the primitive operation
    fn execute(
        &self,
        args: &[Argument],
        context: &mut ExecutionContext,
    ) -> Result<(), ExecutionError>;

    /// Expected number of arguments
    fn arg_count(&self) -> usize;

    /// Name of the primitive
    fn name(&self) -> &str;

    /// Description of what this primitive does
    fn description(&self) -> &str {
        "No description available"
    }
}
