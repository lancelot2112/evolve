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
    /// Lineage this DNA belongs to
    pub lineage_id: u64,
    /// Parent(s) that created this DNA
    pub parents: Vec<super::lineage::ParentInfo>,
    /// The sequence of genes
    pub genes: Vec<Gene>,
    /// Fitness score (None if not yet evaluated)
    pub fitness: Option<f64>,
    /// Generation this DNA was created in
    pub generation: u32,
}

impl DNA {
    /// Create new DNA with lineage
    pub fn new(genes: Vec<Gene>, generation: u32, lineage_id: u64) -> Self {
        Self {
            id: None,
            lineage_id,
            parents: Vec::new(),
            genes,
            fitness: None,
            generation,
        }
    }

    /// Create DNA with parents
    pub fn with_parents(
        genes: Vec<Gene>,
        generation: u32,
        lineage_id: u64,
        parents: Vec<super::lineage::ParentInfo>,
    ) -> Self {
        Self {
            id: None,
            lineage_id,
            parents,
            genes,
            fitness: None,
            generation,
        }
    }

    /// Create empty DNA (used for generation 0 progenitors)
    pub fn empty(generation: u32, lineage_id: u64) -> Self {
        Self::new(Vec::new(), generation, lineage_id)
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

    /// Get the mitochondrial parent (if any)
    pub fn mitochondrial_parent(&self) -> Option<&super::lineage::ParentInfo> {
        self.parents.iter().find(|p| p.is_mitochondrial())
    }

    /// Get all genetic parents (non-mitochondrial)
    pub fn genetic_parents(&self) -> Vec<&super::lineage::ParentInfo> {
        self.parents.iter().filter(|p| p.is_genetic()).collect()
    }

    /// Check if this DNA has any parents
    pub fn has_parents(&self) -> bool {
        !self.parents.is_empty()
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
        let mut dna = DNA::empty(0, 0); // generation 0, lineage 0
        assert!(dna.is_empty());

        dna.push_gene(Gene::primitive(0, vec![Argument::Register(0)]));
        assert_eq!(dna.len(), 1);

        dna.set_fitness(0.95);
        assert_eq!(dna.fitness, Some(0.95));
    }

    #[test]
    fn test_dna_parents() {
        use super::super::lineage::ParentInfo;

        let parent1 = ParentInfo::mitochondrial(10, 0);
        let parent2 = ParentInfo::genetic(20, 1);

        let dna = DNA::with_parents(vec![], 1, 0, vec![parent1, parent2]);

        assert!(dna.has_parents());
        assert!(dna.mitochondrial_parent().is_some());
        assert_eq!(dna.mitochondrial_parent().unwrap().dna_id, 10);
        assert_eq!(dna.genetic_parents().len(), 1);
        assert_eq!(dna.genetic_parents()[0].dna_id, 20);
    }
}
