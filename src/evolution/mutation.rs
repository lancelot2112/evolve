/// Mutation operators for point mutations on a single DNA vector
///
/// Point mutations operate on one DNA strand at a time, making small random changes.
use crate::dna::{Argument, DNA, Gene, OperationId};
use crate::evolution::EvolutionOperator;
use crate::template::TemplateRegistry;
use rand::Rng;

/// Configuration for mutation rates
#[derive(Debug, Clone)]
pub struct MutationConfig {
    /// Probability of inserting a new gene
    pub insertion_rate: f64,
    /// Probability of deleting a gene
    pub deletion_rate: f64,
    /// Probability of substituting a gene
    pub substitution_rate: f64,
    /// Probability of mutating gene arguments
    pub argument_rate: f64,
    /// Probability of expanding a template (rare)
    pub expansion_rate: f64,
    /// Bias toward using templates vs primitives (0.0 = only primitives, 1.0 = only templates)
    pub template_usage_bias: f64,
    /// Maximum number of primitives available
    pub max_primitive_id: u16,
    /// Maximum number of registers available
    pub max_register: u8,
    /// Range for literal values
    pub literal_range: (i64, i64),
}

impl Default for MutationConfig {
    fn default() -> Self {
        Self {
            insertion_rate: 0.1,
            deletion_rate: 0.05,
            substitution_rate: 0.1,
            argument_rate: 0.15,
            expansion_rate: 0.01,
            template_usage_bias: 0.3,
            max_primitive_id: 9, // Assuming 10 primitives (0-9)
            max_register: 7,     // 8 registers (R0-R7)
            literal_range: (-100, 100),
        }
    }
}

/// Point mutator - operates on a single DNA vector
pub struct PointMutator {
    config: MutationConfig,
}

impl PointMutator {
    pub fn new(config: MutationConfig) -> Self {
        Self { config }
    }

    pub fn with_defaults() -> Self {
        Self::new(MutationConfig::default())
    }

    /// Generate a random gene
    fn random_gene<R: Rng>(&self, rng: &mut R, template_registry: &TemplateRegistry) -> Gene {
        let use_template =
            rng.r#gen::<f64>() < self.config.template_usage_bias && template_registry.count() > 0;

        let operation = if use_template {
            // Pick a random template from the registry
            let templates = template_registry.all();
            if !templates.is_empty() {
                let idx = rng.gen_range(0..templates.len());
                OperationId::Template(templates[idx].hash)
            } else {
                // No templates available, use primitive instead
                let prim_id = rng.gen_range(0..=self.config.max_primitive_id);
                OperationId::Primitive(prim_id)
            }
        } else {
            let prim_id = rng.gen_range(0..=self.config.max_primitive_id);
            OperationId::Primitive(prim_id)
        };

        // Generate 0-3 random arguments
        let arg_count = rng.gen_range(0..=3);
        let args = (0..arg_count).map(|_| self.random_argument(rng)).collect();

        Gene::new(operation, args)
    }

    /// Generate a random argument
    fn random_argument<R: Rng>(&self, rng: &mut R) -> Argument {
        if rng.r#gen::<bool>() {
            Argument::Register(rng.gen_range(0..=self.config.max_register))
        } else {
            let (min, max) = self.config.literal_range;
            Argument::Literal(rng.gen_range(min..=max))
        }
    }

    /// Mutate a gene's arguments
    fn mutate_arguments<R: Rng>(&self, rng: &mut R, gene: &Gene) -> Gene {
        let mut new_args = gene.args.clone();

        if !new_args.is_empty() {
            let idx = rng.gen_range(0..new_args.len());
            new_args[idx] = self.random_argument(rng);
        }

        Gene::new(gene.operation, new_args)
    }

    /// Apply insertion mutation
    fn apply_insertion<R: Rng>(
        &self,
        rng: &mut R,
        dna: &DNA,
        template_registry: &TemplateRegistry,
    ) -> DNA {
        let mut new_dna = dna.clone();
        let insert_pos = if new_dna.is_empty() {
            0
        } else {
            rng.gen_range(0..=new_dna.len())
        };

        let new_gene = self.random_gene(rng, template_registry);
        new_dna.insert_gene(insert_pos, new_gene);

        new_dna
    }

    /// Apply deletion mutation
    fn apply_deletion<R: Rng>(&self, rng: &mut R, dna: &DNA) -> DNA {
        if dna.is_empty() {
            return dna.clone();
        }

        let mut new_dna = dna.clone();
        let delete_pos = rng.gen_range(0..new_dna.len());
        new_dna.remove_gene(delete_pos);

        new_dna
    }

    /// Apply substitution mutation
    fn apply_substitution<R: Rng>(
        &self,
        rng: &mut R,
        dna: &DNA,
        template_registry: &TemplateRegistry,
    ) -> DNA {
        if dna.is_empty() {
            return dna.clone();
        }

        let mut new_dna = dna.clone();
        let sub_pos = rng.gen_range(0..new_dna.len());
        new_dna.remove_gene(sub_pos);

        let new_gene = self.random_gene(rng, template_registry);
        new_dna.insert_gene(sub_pos, new_gene);

        new_dna
    }

    /// Apply argument mutation
    fn apply_argument_mutation<R: Rng>(&self, rng: &mut R, dna: &DNA) -> DNA {
        if dna.is_empty() {
            return dna.clone();
        }

        let mut new_dna = dna.clone();
        let gene_pos = rng.gen_range(0..new_dna.len());

        let old_gene = &new_dna.genes[gene_pos];
        let mutated_gene = self.mutate_arguments(rng, old_gene);
        new_dna.genes[gene_pos] = mutated_gene;

        new_dna
    }

    /// Apply template expansion mutation (rare)
    fn apply_expansion<R: Rng>(
        &self,
        rng: &mut R,
        dna: &DNA,
        template_registry: &TemplateRegistry,
    ) -> DNA {
        if dna.is_empty() {
            return dna.clone();
        }

        // Find a gene that uses a template
        let template_positions: Vec<usize> = dna
            .genes
            .iter()
            .enumerate()
            .filter_map(|(idx, gene)| match gene.operation {
                OperationId::Template(_) => Some(idx),
                _ => None,
            })
            .collect();

        if template_positions.is_empty() {
            return dna.clone();
        }

        let pos = template_positions[rng.gen_range(0..template_positions.len())];
        let gene = &dna.genes[pos];

        if let OperationId::Template(template_hash) = gene.operation {
            if let Some(template) = template_registry.get(template_hash) {
                let mut new_dna = dna.clone();
                new_dna.remove_gene(pos);

                // Insert all genes from the template
                for (offset, template_gene) in template.genes.iter().enumerate() {
                    new_dna.insert_gene(pos + offset, template_gene.clone());
                }

                return new_dna;
            }
        }

        dna.clone()
    }

    /// Apply all mutations based on configured rates
    /// Requires the lineage's template registry to be passed in
    pub fn mutate(&self, dna: &DNA, template_registry: &TemplateRegistry) -> DNA {
        let mut rng = rand::thread_rng();
        let mut result = dna.clone();
        result.fitness = None; // Reset fitness - must be re-evaluated

        // Apply each mutation type based on probability
        if rng.r#gen::<f64>() < self.config.insertion_rate {
            result = self.apply_insertion(&mut rng, &result, template_registry);
        }

        if rng.r#gen::<f64>() < self.config.deletion_rate {
            result = self.apply_deletion(&mut rng, &result);
        }

        if rng.r#gen::<f64>() < self.config.substitution_rate {
            result = self.apply_substitution(&mut rng, &result, template_registry);
        }

        if rng.r#gen::<f64>() < self.config.argument_rate {
            result = self.apply_argument_mutation(&mut rng, &result);
        }

        if rng.r#gen::<f64>() < self.config.expansion_rate {
            result = self.apply_expansion(&mut rng, &result, template_registry);
        }

        result
    }
}

// NOTE: EvolutionOperator trait needs redesign to support lineage-based templates
// Commenting out until trait is updated
// impl EvolutionOperator for PointMutator {
//     fn apply(&self, dna: &DNA) -> DNA {
//         self.mutate(dna, template_registry)
//     }
// }

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_insertion_mutation() {
        let mutator = PointMutator::with_defaults();

        let dna = DNA::empty(0, 0); // generation 0, lineage 0
        let mutated = mutator.apply_insertion(
            &mut rand::thread_rng(),
            &dna,
            &crate::template::TemplateRegistry::new(),
        );

        assert_eq!(mutated.len(), 1);
    }

    #[test]
    fn test_deletion_mutation() {
        let mutator = PointMutator::with_defaults();

        let mut dna = DNA::empty(0, 0); // generation 0, lineage 0
        dna.push_gene(Gene::primitive(0, vec![Argument::Register(0)]));
        dna.push_gene(Gene::primitive(1, vec![Argument::Register(1)]));

        let mutated = mutator.apply_deletion(&mut rand::thread_rng(), &dna);
        assert_eq!(mutated.len(), 1);
    }

    #[test]
    fn test_argument_mutation() {
        let mutator = PointMutator::with_defaults();

        let mut dna = DNA::empty(0, 0); // generation 0, lineage 0
        dna.push_gene(Gene::primitive(0, vec![Argument::Register(0)]));

        let mutated = mutator.apply_argument_mutation(&mut rand::thread_rng(), &dna);
        assert_eq!(mutated.len(), 1);
        // Arguments should have changed (probabilistically)
    }

    #[test]
    fn test_full_mutation() {
        let mutator = PointMutator::with_defaults();

        let mut dna = DNA::empty(0, 0); // generation 0, lineage 0
        dna.push_gene(Gene::primitive(0, vec![Argument::Register(0)]));

        let mutated = mutator.mutate(&dna, &crate::template::TemplateRegistry::new());
        assert!(mutated.fitness.is_none()); // Fitness should be reset
    }
}
