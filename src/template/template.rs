//! Template Struct and Registry
//!
//! Provides the Template struct for representing evolved gene sequences,
//! and TemplateRegistry for managing collections of templates.
//!
//! Templates are stored in a HashMap keyed by their content hash, which
//! provides automatic deduplication and enables lineage-local template
//! libraries for mitochondrial-style inheritance.

use super::hashing::hash_genes;
use crate::dna::Gene;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// A template - an evolved sequence identified by hash
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Template {
    /// Hash of the gene sequence (unique identifier)
    pub hash: u64,
    /// The evolved sequence of genes (between START and END markers)
    pub genes: Vec<Gene>,
    /// Fitness when this template was saved
    pub fitness_when_saved: f64,
    /// Generation when created
    pub generation_created: u32,
    /// How many times this template has been used
    pub usage_count: u64,
}

impl Template {
    pub fn new(genes: Vec<Gene>, fitness: f64, generation: u32) -> Self {
        let hash = hash_genes(&genes);
        Self {
            hash,
            genes,
            fitness_when_saved: fitness,
            generation_created: generation,
            usage_count: 0,
        }
    }

    /// Increment usage counter
    pub fn record_usage(&mut self) {
        self.usage_count += 1;
    }

    /// Get the length of this template (number of genes)
    pub fn len(&self) -> usize {
        self.genes.len()
    }

    /// Check if template is empty
    pub fn is_empty(&self) -> bool {
        self.genes.is_empty()
    }
}

/// Registry of templates (hash-based)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TemplateRegistry {
    templates: HashMap<u64, Template>,
}

impl TemplateRegistry {
    pub fn new() -> Self {
        Self {
            templates: HashMap::new(),
        }
    }

    /// Register a new template (returns hash, or existing if duplicate)
    pub fn register(&mut self, genes: Vec<Gene>, fitness: f64, generation: u32) -> u64 {
        let template = Template::new(genes, fitness, generation);
        let hash = template.hash;

        // Only insert if not already present (deduplication!)
        self.templates.entry(hash).or_insert(template);

        hash
    }

    /// Get a template by hash
    pub fn get(&self, hash: u64) -> Option<&Template> {
        self.templates.get(&hash)
    }

    /// Get a mutable reference to a template by hash
    pub fn get_mut(&mut self, hash: u64) -> Option<&mut Template> {
        self.templates.get_mut(&hash)
    }

    /// Get all templates
    pub fn all(&self) -> Vec<&Template> {
        self.templates.values().collect()
    }

    /// Get the number of registered templates
    pub fn count(&self) -> usize {
        self.templates.len()
    }

    /// Get templates sorted by usage count (most used first)
    pub fn by_usage(&self) -> Vec<&Template> {
        let mut sorted: Vec<&Template> = self.templates.values().collect();
        sorted.sort_by(|a, b| b.usage_count.cmp(&a.usage_count));
        sorted
    }

    /// Get templates sorted by fitness (highest first)
    pub fn by_fitness(&self) -> Vec<&Template> {
        let mut sorted: Vec<&Template> = self.templates.values().collect();
        sorted.sort_by(|a, b| {
            b.fitness_when_saved
                .partial_cmp(&a.fitness_when_saved)
                .unwrap_or(std::cmp::Ordering::Equal)
        });
        sorted
    }

    /// Get templates from a specific generation
    pub fn from_generation(&self, generation: u32) -> Vec<&Template> {
        self.templates
            .values()
            .filter(|t| t.generation_created == generation)
            .collect()
    }

    /// Merge another template library into this one (for inheritance)
    pub fn merge(&mut self, other: &TemplateRegistry) {
        for (hash, template) in &other.templates {
            self.templates
                .entry(*hash)
                .or_insert_with(|| template.clone());
        }
    }
}

impl Default for TemplateRegistry {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::dna::{Argument, Gene, OperationId};

    #[test]
    fn test_template_registration() {
        let mut registry = TemplateRegistry::new();

        let genes = vec![
            Gene::new(OperationId::Primitive(0), vec![Argument::Register(0)]),
            Gene::new(OperationId::Primitive(1), vec![Argument::Register(1)]),
        ];

        let hash = registry.register(genes.clone(), 0.9, 5);
        assert_eq!(registry.count(), 1);

        let template = registry.get(hash).unwrap();
        assert_eq!(template.genes.len(), 2);
        assert_eq!(template.fitness_when_saved, 0.9);
        assert_eq!(template.generation_created, 5);
    }

    #[test]
    fn test_template_usage_tracking() {
        let mut registry = TemplateRegistry::new();
        let hash = registry.register(vec![], 0.8, 0);

        let template = registry.get_mut(hash).unwrap();
        assert_eq!(template.usage_count, 0);

        template.record_usage();
        template.record_usage();
        assert_eq!(template.usage_count, 2);
    }

    #[test]
    fn test_template_sorting() {
        let mut registry = TemplateRegistry::new();

        // Create templates with different gene sequences to ensure different hashes
        let genes1 = vec![Gene::new(
            OperationId::Primitive(0),
            vec![Argument::Register(0)],
        )];
        let genes2 = vec![Gene::new(
            OperationId::Primitive(1),
            vec![Argument::Register(1)],
        )];
        let genes3 = vec![Gene::new(
            OperationId::Primitive(2),
            vec![Argument::Register(2)],
        )];

        registry.register(genes1, 0.5, 0);
        registry.register(genes2, 0.9, 1);
        registry.register(genes3, 0.7, 2);

        let by_fitness = registry.by_fitness();
        assert_eq!(by_fitness[0].fitness_when_saved, 0.9);
        assert_eq!(by_fitness[1].fitness_when_saved, 0.7);
        assert_eq!(by_fitness[2].fitness_when_saved, 0.5);
    }
}
