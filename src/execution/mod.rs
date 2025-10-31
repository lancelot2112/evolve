//! Execution Module
//!
//! Sandboxed execution environment for DNA with safety limits and fitness evaluation.
//!
//! This module provides the Executor for running DNA sequences, ExecutionResult for
//! capturing outcomes, and various FitnessFunction implementations for evaluating
//! how well DNA performs on test cases.

mod executor;
mod config;
mod result;
mod fitness;

// Re-export core types
pub use executor::Executor;
pub use config::ExecutionConfig;
pub use result::ExecutionResult;
pub use fitness::{FitnessFunction, ExactMatchFitness, MSEFitness, PartialMatchFitness};
