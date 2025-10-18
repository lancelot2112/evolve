/// Evolution module: Mutation and crossover operators
///
/// This module provides the evolutionary operators that modify DNA strands.
/// It is decoupled from the DNA representation and operates on abstract gene sequences.

pub mod mutation;
pub mod crossover;

pub use mutation::{PointMutator, MutationConfig};
pub use crossover::{Crossover, CrossoverMode};

use crate::dna::DNA;
use crate::template::TemplateRegistry;

/// Trait for evolution operators
pub trait EvolutionOperator {
    /// Apply this operator to DNA, potentially using template registry
    fn apply(&self, dna: &DNA, template_registry: &TemplateRegistry) -> DNA;
}
