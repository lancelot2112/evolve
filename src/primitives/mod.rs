//! Primitives Module
//!
//! User-defined base operations that serve as the fundamental building blocks
//! for the evolutionary algorithm.
//!
//! This module provides the execution context, primitive trait, registry, and
//! all standard primitive implementations organized by category.

mod context;
mod trait_def;
mod registry;
mod arithmetic;
mod stack;
mod io;
mod data;
mod markers;

// Re-export core types and traits
pub use context::{ExecutionContext, ExecutionError};
pub use trait_def::Primitive;
pub use registry::PrimitiveRegistry;

// Re-export all primitive implementations
pub use arithmetic::{Add, Sub, Mul, Div};
pub use stack::{Push, Pop};
pub use io::{ReadInput, WriteOutput};
pub use data::{Copy, Nop};
pub use markers::{TemplateStart, TemplateEnd};
