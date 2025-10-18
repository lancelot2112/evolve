/// Crossover operators for splicing two or more DNA vectors
///
/// Crossover operations combine genetic material from multiple parents to create offspring.

use crate::dna::{DNA, Gene, OperationId};
use crate::evolution::EvolutionOperator;
use crate::template::TemplateRegistry;
use rand::Rng;

/// Different modes of crossover
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CrossoverMode {
    /// Cut both strands at same position, swap tails
    SinglePoint,
    /// Extract segment from one, insert into other
    TwoPoint,
    /// Randomly select genes from multiple parents
    Uniform { parent_count: usize },
    /// Prefer cutting at template boundaries
    TemplateAware,
}

/// Crossover operator for combining DNA from multiple parents
pub struct Crossover {
    mode: CrossoverMode,
}

impl Crossover {
    pub fn new(mode: CrossoverMode) -> Self {
        Self { mode }
    }

    pub fn single_point() -> Self {
        Self::new(CrossoverMode::SinglePoint)
    }

    pub fn two_point() -> Self {
        Self::new(CrossoverMode::TwoPoint)
    }

    pub fn uniform(parent_count: usize) -> Self {
        Self::new(CrossoverMode::Uniform { parent_count })
    }

    pub fn template_aware() -> Self {
        Self::new(CrossoverMode::TemplateAware)
    }

    /// Perform single-point crossover on two parents
    fn single_point_crossover(&self, parent1: &DNA, parent2: &DNA) -> DNA {
        if parent1.is_empty() && parent2.is_empty() {
            return DNA::empty(parent1.generation);
        }

        if parent1.is_empty() {
            return parent2.clone();
        }

        if parent2.is_empty() {
            return parent1.clone();
        }

        let mut rng = rand::thread_rng();

        // Choose a crossover point
        let point = rng.gen_range(0..=parent1.len().min(parent2.len()));

        // Take head from parent1, tail from parent2
        let mut genes = Vec::new();
        genes.extend_from_slice(&parent1.genes[..point]);
        genes.extend_from_slice(&parent2.genes[point.min(parent2.len())..]);

        let mut child = DNA::new(genes, parent1.generation);
        child.fitness = None; // Reset fitness

        // Mitochondrial inheritance: randomly inherit template library from one parent
        child.template_library = if rng.r#gen::<bool>() {
            parent1.template_library.clone()
        } else {
            parent2.template_library.clone()
        };

        child
    }

    /// Perform two-point crossover on two parents
    fn two_point_crossover(&self, parent1: &DNA, parent2: &DNA) -> DNA {
        if parent1.is_empty() || parent2.is_empty() {
            return if !parent1.is_empty() {
                parent1.clone()
            } else {
                parent2.clone()
            };
        }

        let mut rng = rand::thread_rng();

        // Choose two crossover points
        let max_len = parent1.len().max(parent2.len());
        let mut point1 = rng.gen_range(0..=max_len);
        let mut point2 = rng.gen_range(0..=max_len);

        if point1 > point2 {
            std::mem::swap(&mut point1, &mut point2);
        }

        // Build child: parent1[..p1] + parent2[p1..p2] + parent1[p2..]
        let mut genes = Vec::new();

        // Head from parent1
        let p1_clamped = point1.min(parent1.len());
        genes.extend_from_slice(&parent1.genes[..p1_clamped]);

        // Middle from parent2
        let p2_start = point1.min(parent2.len());
        let p2_end = point2.min(parent2.len());
        if p2_start < p2_end {
            genes.extend_from_slice(&parent2.genes[p2_start..p2_end]);
        }

        // Tail from parent1
        let p2_clamped = point2.min(parent1.len());
        if p2_clamped < parent1.len() {
            genes.extend_from_slice(&parent1.genes[p2_clamped..]);
        }

        let mut child = DNA::new(genes, parent1.generation);
        child.fitness = None;

        // Mitochondrial inheritance: randomly inherit template library from one parent
        child.template_library = if rng.r#gen::<bool>() {
            parent1.template_library.clone()
        } else {
            parent2.template_library.clone()
        };

        child
    }

    /// Perform uniform crossover on multiple parents
    fn uniform_crossover(&self, parents: &[&DNA]) -> DNA {
        if parents.is_empty() {
            return DNA::empty(0);
        }

        if parents.len() == 1 {
            return parents[0].clone();
        }

        let mut rng = rand::thread_rng();

        // Find the maximum length
        let max_len = parents.iter().map(|p| p.len()).max().unwrap_or(0);

        let mut genes = Vec::new();

        for i in 0..max_len {
            // Randomly select a parent that has a gene at this position
            let available_parents: Vec<&DNA> = parents
                .iter()
                .filter(|p| i < p.len())
                .copied()
                .collect();

            if !available_parents.is_empty() {
                let chosen = available_parents[rng.gen_range(0..available_parents.len())];
                genes.push(chosen.genes[i].clone());
            }
        }

        let generation = parents[0].generation;
        let mut child = DNA::new(genes, generation);
        child.fitness = None;

        // Mitochondrial inheritance: randomly inherit template library from one parent
        let chosen_parent = parents[rng.gen_range(0..parents.len())];
        child.template_library = chosen_parent.template_library.clone();

        child
    }

    /// Find positions that are good crossover points (at template boundaries)
    fn find_template_boundaries(&self, dna: &DNA) -> Vec<usize> {
        let mut boundaries = vec![0]; // Always include start

        for (i, gene) in dna.genes.iter().enumerate() {
            // Add boundary after each template gene
            if matches!(gene.operation, OperationId::Template(_)) {
                boundaries.push(i + 1);
            }
        }

        boundaries.push(dna.len()); // Always include end
        boundaries.sort_unstable();
        boundaries.dedup();
        boundaries
    }

    /// Perform template-aware crossover
    fn template_aware_crossover(&self, parent1: &DNA, parent2: &DNA) -> DNA {
        if parent1.is_empty() || parent2.is_empty() {
            return if !parent1.is_empty() {
                parent1.clone()
            } else {
                parent2.clone()
            };
        }

        let mut rng = rand::thread_rng();

        let boundaries1 = self.find_template_boundaries(parent1);
        let boundaries2 = self.find_template_boundaries(parent2);

        // Choose crossover points from boundaries
        let point1 = boundaries1[rng.gen_range(0..boundaries1.len())];
        let point2 = boundaries2[rng.gen_range(0..boundaries2.len())];

        // Build child: parent1[..point1] + parent2[point2..]
        let mut genes = Vec::new();
        genes.extend_from_slice(&parent1.genes[..point1]);
        genes.extend_from_slice(&parent2.genes[point2..]);

        let mut child = DNA::new(genes, parent1.generation);
        child.fitness = None;

        // Mitochondrial inheritance: randomly inherit template library from one parent
        child.template_library = if rng.r#gen::<bool>() {
            parent1.template_library.clone()
        } else {
            parent2.template_library.clone()
        };

        child
    }

    /// Perform crossover on two parents
    pub fn cross(&self, parent1: &DNA, parent2: &DNA) -> DNA {
        match self.mode {
            CrossoverMode::SinglePoint => self.single_point_crossover(parent1, parent2),
            CrossoverMode::TwoPoint => self.two_point_crossover(parent1, parent2),
            CrossoverMode::Uniform { parent_count: _ } => {
                // For uniform with 2 parents, treat as uniform crossover
                self.uniform_crossover(&[parent1, parent2])
            }
            CrossoverMode::TemplateAware => self.template_aware_crossover(parent1, parent2),
        }
    }

    /// Perform crossover on multiple parents (for uniform crossover)
    pub fn cross_many(&self, parents: &[&DNA]) -> DNA {
        match self.mode {
            CrossoverMode::Uniform { .. } => self.uniform_crossover(parents),
            _ => {
                // For non-uniform modes, just use first two parents
                if parents.len() >= 2 {
                    self.cross(parents[0], parents[1])
                } else if parents.len() == 1 {
                    parents[0].clone()
                } else {
                    DNA::empty(0)
                }
            }
        }
    }
}

impl EvolutionOperator for Crossover {
    fn apply(&self, dna: &DNA) -> DNA {
        // Note: Crossover needs a second parent, so this implementation
        // just returns a clone. Use cross() or cross_many() directly instead.
        dna.clone()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::dna::Argument;

    #[test]
    fn test_single_point_crossover() {
        let mut parent1 = DNA::empty(0);
        parent1.push_gene(Gene::primitive(0, vec![Argument::Register(0)]));
        parent1.push_gene(Gene::primitive(1, vec![Argument::Register(1)]));
        parent1.push_gene(Gene::primitive(2, vec![Argument::Register(2)]));

        let mut parent2 = DNA::empty(0);
        parent2.push_gene(Gene::primitive(3, vec![Argument::Register(3)]));
        parent2.push_gene(Gene::primitive(4, vec![Argument::Register(4)]));

        let crossover = Crossover::single_point();
        let child = crossover.cross(&parent1, &parent2);

        // Child should have genes from both parents
        assert!(!child.is_empty());
        assert!(child.fitness.is_none());
    }

    #[test]
    fn test_two_point_crossover() {
        let mut parent1 = DNA::empty(0);
        parent1.push_gene(Gene::primitive(0, vec![]));
        parent1.push_gene(Gene::primitive(1, vec![]));
        parent1.push_gene(Gene::primitive(2, vec![]));

        let mut parent2 = DNA::empty(0);
        parent2.push_gene(Gene::primitive(3, vec![]));
        parent2.push_gene(Gene::primitive(4, vec![]));
        parent2.push_gene(Gene::primitive(5, vec![]));

        let crossover = Crossover::two_point();
        let child = crossover.cross(&parent1, &parent2);

        assert!(!child.is_empty());
    }

    #[test]
    fn test_uniform_crossover() {
        let mut parent1 = DNA::empty(0);
        parent1.push_gene(Gene::primitive(0, vec![]));
        parent1.push_gene(Gene::primitive(1, vec![]));

        let mut parent2 = DNA::empty(0);
        parent2.push_gene(Gene::primitive(2, vec![]));
        parent2.push_gene(Gene::primitive(3, vec![]));

        let mut parent3 = DNA::empty(0);
        parent3.push_gene(Gene::primitive(4, vec![]));
        parent3.push_gene(Gene::primitive(5, vec![]));

        let crossover = Crossover::uniform(3);
        let child = crossover.cross_many(&[&parent1, &parent2, &parent3]);

        assert_eq!(child.len(), 2);
    }

    #[test]
    fn test_template_aware_boundaries() {
        let mut dna = DNA::empty(0);
        dna.push_gene(Gene::primitive(0, vec![]));
        dna.push_gene(Gene::template(0, vec![]));
        dna.push_gene(Gene::primitive(1, vec![]));
        dna.push_gene(Gene::template(1, vec![]));

        let crossover = Crossover::template_aware();
        let boundaries = crossover.find_template_boundaries(&dna);

        // Should have boundaries at: 0, 2 (after first template), 4 (after second template), 4 (end)
        assert!(boundaries.contains(&0));
        assert!(boundaries.contains(&2));
        assert!(boundaries.contains(&4));
    }
}
