//! Primitives Module
//!
//! User-defined base operations that serve as the fundamental building blocks
//! for the evolutionary algorithm.
//!
//! This module provides the execution context, primitive trait, registry, and
//! all standard primitive implementations organized by category.

mod arithmetic;
mod context;
mod data;
mod io;
mod markers;
pub mod neuron;
mod registry;
mod stack;
mod trait_def;

// Re-export core types and traits
pub use context::{ExecutionContext, ExecutionError};
pub use registry::PrimitiveRegistry;
pub use trait_def::Primitive;

// Re-export all primitive implementations
