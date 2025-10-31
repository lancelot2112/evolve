/// Evolution module: Mutation and crossover operators
///
/// This module provides the evolutionary operators that modify DNA strands.
/// It is decoupled from the DNA representation and operates on abstract gene sequences.

pub mod mutation;
pub mod crossover;

pub use mutation::{PointMutator, MutationConfig};
pub use crossover::{Crossover, CrossoverMode};

use crate::dna::DNA;

/// Trait for evolution operators
pub trait EvolutionOperator {
    /// Apply this operator to DNA (uses DNA's local template_library)
    fn apply(&self, dna: &DNA) -> DNA;
}
