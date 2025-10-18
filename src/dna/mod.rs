/// DNA module: Core genetic code representation
///
/// This module defines the fundamental structure of genetic code in the evolution system.
/// DNA is composed of Genes, which reference either user-defined Primitives or
/// algorithm-evolved Templates.

use serde::{Deserialize, Serialize};

/// Identifies either a primitive operation or a template
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum OperationId {
    /// References a user-defined primitive by its index
    Primitive(u16),
    /// References an algorithm-evolved template by its hash
    Template(u64),
}

/// Arguments that can be passed to operations
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum Argument {
    /// References a register/variable (e.g., R0, R1, ...)
    Register(u8),
    /// A constant literal value
    Literal(i64),
}

/// A single gene - the atomic unit of genetic code
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Gene {
    /// The operation to perform (primitive or template)
    pub operation: OperationId,
    /// Arguments to the operation
    pub args: Vec<Argument>,
}

impl Gene {
    pub fn new(operation: OperationId, args: Vec<Argument>) -> Self {
        Self { operation, args }
    }

    /// Create a gene referencing a primitive
    pub fn primitive(id: u16, args: Vec<Argument>) -> Self {
        Self::new(OperationId::Primitive(id), args)
    }

    /// Create a gene referencing a template
    pub fn template(hash: u64, args: Vec<Argument>) -> Self {
        Self::new(OperationId::Template(hash), args)
    }
}

/// A DNA strand - a linear sequence of genes
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DNA {
    /// Unique incremental ID (assigned when saved to history)
    pub id: Option<u64>,
    /// The sequence of genes
    pub genes: Vec<Gene>,
    /// Fitness score (None if not yet evaluated)
    pub fitness: Option<f64>,
    /// Generation this DNA was created in
    pub generation: u32,
    /// Lineage-local template library (inherited from parent)
    #[serde(skip)]  // Don't serialize - will be reconstructed
    pub template_library: crate::template::TemplateRegistry,
}

impl DNA {
    pub fn new(genes: Vec<Gene>, generation: u32) -> Self {
        Self {
            id: None,
            genes,
            fitness: None,
            generation,
            template_library: crate::template::TemplateRegistry::new(),
        }
    }

    /// Create DNA with a specific template library
    pub fn with_template_library(genes: Vec<Gene>, generation: u32, template_library: crate::template::TemplateRegistry) -> Self {
        Self {
            id: None,
            genes,
            fitness: None,
            generation,
            template_library,
        }
    }

    /// Create empty DNA
    pub fn empty(generation: u32) -> Self {
        Self::new(Vec::new(), generation)
    }

    /// Set the fitness score
    pub fn set_fitness(&mut self, fitness: f64) {
        self.fitness = Some(fitness);
    }

    /// Get the length of the DNA (number of genes)
    pub fn len(&self) -> usize {
        self.genes.len()
    }

    /// Check if DNA is empty
    pub fn is_empty(&self) -> bool {
        self.genes.is_empty()
    }

    /// Add a gene to the end
    pub fn push_gene(&mut self, gene: Gene) {
        self.genes.push(gene);
    }

    /// Insert a gene at a specific position
    pub fn insert_gene(&mut self, index: usize, gene: Gene) {
        self.genes.insert(index, gene);
    }

    /// Remove a gene at a specific position
    pub fn remove_gene(&mut self, index: usize) -> Gene {
        self.genes.remove(index)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_gene_creation() {
        let gene = Gene::primitive(0, vec![Argument::Register(0), Argument::Literal(42)]);
        assert_eq!(gene.operation, OperationId::Primitive(0));
        assert_eq!(gene.args.len(), 2);
    }

    #[test]
    fn test_dna_operations() {
        let mut dna = DNA::empty(0);
        assert!(dna.is_empty());

        dna.push_gene(Gene::primitive(0, vec![Argument::Register(0)]));
        assert_eq!(dna.len(), 1);

        dna.set_fitness(0.95);
        assert_eq!(dna.fitness, Some(0.95));
    }
}
