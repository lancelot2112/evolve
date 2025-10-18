/// Template module: Algorithm-evolved indexed sequences
///
/// Templates are successful DNA sequences between TEMPLATE_START and TEMPLATE_END markers.
/// They are identified by hashing their gene sequence and stored in a HashMap.
/// This creates configuration space compression and allows lineage-local template libraries.

use crate::dna::{Gene, OperationId};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::collections::hash_map::DefaultHasher;
use std::hash::{Hash, Hasher};

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

/// Hash a sequence of genes to create a unique identifier
pub fn hash_genes(genes: &[Gene]) -> u64 {
    let mut hasher = DefaultHasher::new();

    for gene in genes {
        // Hash the operation
        match gene.operation {
            OperationId::Primitive(id) => {
                0u8.hash(&mut hasher);  // Tag for primitive
                id.hash(&mut hasher);
            }
            OperationId::Template(hash) => {
                1u8.hash(&mut hasher);  // Tag for template
                hash.hash(&mut hasher);
            }
        }

        // Hash the arguments
        gene.args.len().hash(&mut hasher);
        for arg in &gene.args {
            // Need to implement Hash for Argument
            match arg {
                crate::dna::Argument::Register(r) => {
                    0u8.hash(&mut hasher);
                    r.hash(&mut hasher);
                }
                crate::dna::Argument::Literal(l) => {
                    1u8.hash(&mut hasher);
                    l.hash(&mut hasher);
                }
            }
        }
    }

    hasher.finish()
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
            self.templates.entry(*hash).or_insert_with(|| template.clone());
        }
    }
}

impl Default for TemplateRegistry {
    fn default() -> Self {
        Self::new()
    }
}

/// Detect templates in DNA (sequences between TEMPLATE_START and TEMPLATE_END)
/// Returns list of (start_index, end_index, hash) for found templates
pub fn detect_templates(genes: &[Gene]) -> Vec<(usize, usize, u64)> {
    const TEMPLATE_START_ID: u16 = 10;  // ID of TEMPLATE_START primitive
    const TEMPLATE_END_ID: u16 = 11;     // ID of TEMPLATE_END primitive

    let mut templates = Vec::new();
    let mut start_idx = None;

    for (i, gene) in genes.iter().enumerate() {
        match gene.operation {
            OperationId::Primitive(id) if id == TEMPLATE_START_ID => {
                // Found start marker
                start_idx = Some(i);
            }
            OperationId::Primitive(id) if id == TEMPLATE_END_ID => {
                // Found end marker
                if let Some(start) = start_idx {
                    // Extract genes between markers (excluding the markers themselves)
                    let template_genes = &genes[start + 1..i];
                    if !template_genes.is_empty() {
                        let hash = hash_genes(template_genes);
                        templates.push((start, i, hash));
                    }
                    start_idx = None;
                }
            }
            _ => {}
        }
    }

    templates
}

/// Strategy for deciding when to create templates from DNA
#[derive(Debug, Clone)]
pub enum TemplateCreationStrategy {
    /// Create template if fitness is in top N percent
    TopPercentile(f64),
    /// Create template every N generations for best performer
    GenerationalBest { interval: u32 },
    /// Create template if fitness exceeds threshold
    FitnessThreshold(f64),
    /// Manually promoted (no automatic creation)
    Manual,
}

impl TemplateCreationStrategy {
    /// Determine if a DNA should become a template
    pub fn should_create_template(
        &self,
        fitness: f64,
        generation: u32,
        population_fitness: &[f64],
    ) -> bool {
        match self {
            Self::TopPercentile(percentile) => {
                if population_fitness.is_empty() {
                    return false;
                }
                let mut sorted = population_fitness.to_vec();
                sorted.sort_by(|a, b| b.partial_cmp(a).unwrap_or(std::cmp::Ordering::Equal));

                // For top N%, find the cutoff index
                // e.g., top 10% of 10 items = top 1 item, cutoff at index 0
                let count = (percentile * sorted.len() as f64).ceil() as usize;
                let threshold_index = (count.saturating_sub(1)).min(sorted.len().saturating_sub(1));

                fitness >= sorted[threshold_index]
            }
            Self::GenerationalBest { interval } => generation % interval == 0,
            Self::FitnessThreshold(threshold) => fitness >= *threshold,
            Self::Manual => false,
        }
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

        let index = registry.register(genes.clone(), 0.9, 5);
        assert_eq!(index, 0);
        assert_eq!(registry.count(), 1);

        let template = registry.get(index).unwrap();
        assert_eq!(template.genes.len(), 2);
        assert_eq!(template.fitness_when_saved, 0.9);
        assert_eq!(template.generation_created, 5);
    }

    #[test]
    fn test_template_usage_tracking() {
        let mut registry = TemplateRegistry::new();
        let index = registry.register(vec![], 0.8, 0);

        let template = registry.get_mut(index).unwrap();
        assert_eq!(template.usage_count, 0);

        template.record_usage();
        template.record_usage();
        assert_eq!(template.usage_count, 2);
    }

    #[test]
    fn test_template_sorting() {
        let mut registry = TemplateRegistry::new();
        registry.register(vec![], 0.5, 0);
        registry.register(vec![], 0.9, 1);
        registry.register(vec![], 0.7, 2);

        let by_fitness = registry.by_fitness();
        assert_eq!(by_fitness[0].fitness_when_saved, 0.9);
        assert_eq!(by_fitness[1].fitness_when_saved, 0.7);
        assert_eq!(by_fitness[2].fitness_when_saved, 0.5);
    }

    #[test]
    fn test_creation_strategy() {
        let strategy = TemplateCreationStrategy::TopPercentile(0.1);
        let population = vec![0.1, 0.2, 0.3, 0.4, 0.5, 0.6, 0.7, 0.8, 0.9, 1.0];

        // Top 10% of 10 items = top 1 item, threshold is 1.0
        assert!(strategy.should_create_template(1.0, 0, &population));
        assert!(!strategy.should_create_template(0.9, 0, &population));
        assert!(!strategy.should_create_template(0.5, 0, &population));

        // Test with top 20% = top 2 items, threshold is 0.9
        let strategy2 = TemplateCreationStrategy::TopPercentile(0.2);
        assert!(strategy2.should_create_template(1.0, 0, &population));
        assert!(strategy2.should_create_template(0.9, 0, &population));
        assert!(!strategy2.should_create_template(0.8, 0, &population));
    }
}
