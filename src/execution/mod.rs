//! Execution Module
//!
//! Sandboxed execution environment for DNA with safety limits and fitness evaluation.
//!
//! This module provides the Executor for running DNA sequences, ExecutionResult for
//! capturing outcomes, and various FitnessFunction implementations for evaluating
//! how well DNA performs on test cases.

mod config;
mod executor;
mod fitness;
mod result;

// Re-export core types
pub use config::ExecutionConfig;
pub use executor::Executor;
pub use fitness::{ExactMatchFitness, FitnessFunction, MSEFitness, PartialMatchFitness};
pub use result::ExecutionResult;
