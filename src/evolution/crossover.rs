/// Crossover operators for splicing two or more DNA vectors
///
/// Crossover operations combine genetic material from multiple parents to create offspring.
use crate::dna::{DNA, OperationId};
use crate::evolution::EvolutionOperator;
use crate::lineage::ParentInfo;
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
            return DNA::empty(parent1.generation, parent1.lineage_id);
        }

        if parent1.is_empty() {
            return parent2.clone();
        }

        if parent2.is_empty() {
            return parent1.clone();
        }

        let mut rng = rand::thread_rng();

        // Randomly choose which parent is mitochondrial (donates template library)
        let (mito_parent, gene_parent) = if rng.r#gen::<bool>() {
            (parent1, parent2)
        } else {
            (parent2, parent1)
        };

        // Choose a crossover point
        let point = rng.gen_range(0..=parent1.len().min(parent2.len()));

        // Take head from parent1, tail from parent2
        let mut genes = Vec::new();
        genes.extend_from_slice(&parent1.genes[..point]);
        genes.extend_from_slice(&parent2.genes[point.min(parent2.len())..]);

        // Create child in mitochondrial parent's lineage with parent tracking
        let parent_info = vec![
            ParentInfo::mitochondrial(
                mito_parent
                    .id
                    .expect("Mitochondrial parent must have an ID"),
                mito_parent.lineage_id,
            ),
            ParentInfo::genetic(
                gene_parent.id.expect("Genetic parent must have an ID"),
                gene_parent.lineage_id,
            ),
        ];

        DNA::with_parents(
            genes,
            parent1.generation,
            mito_parent.lineage_id,
            parent_info,
        )
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

        // Randomly choose which parent is mitochondrial (donates template library)
        let (mito_parent, gene_parent) = if rng.r#gen::<bool>() {
            (parent1, parent2)
        } else {
            (parent2, parent1)
        };

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

        // Create child in mitochondrial parent's lineage with parent tracking
        let parent_info = vec![
            ParentInfo::mitochondrial(
                mito_parent
                    .id
                    .expect("Mitochondrial parent must have an ID"),
                mito_parent.lineage_id,
            ),
            ParentInfo::genetic(
                gene_parent.id.expect("Genetic parent must have an ID"),
                gene_parent.lineage_id,
            ),
        ];

        DNA::with_parents(
            genes,
            parent1.generation,
            mito_parent.lineage_id,
            parent_info,
        )
    }

    /// Perform uniform crossover on multiple parents
    fn uniform_crossover(&self, parents: &[&DNA]) -> DNA {
        if parents.is_empty() {
            return DNA::empty(0, 0);
        }

        if parents.len() == 1 {
            return parents[0].clone();
        }

        let mut rng = rand::thread_rng();

        // Randomly choose which parent is mitochondrial (donates template library)
        let mito_parent_idx = rng.gen_range(0..parents.len());
        let mito_parent = parents[mito_parent_idx];

        // Find the maximum length
        let max_len = parents.iter().map(|p| p.len()).max().unwrap_or(0);

        let mut genes = Vec::new();

        for i in 0..max_len {
            // Randomly select a parent that has a gene at this position
            let available_parents: Vec<&DNA> =
                parents.iter().filter(|p| i < p.len()).copied().collect();

            if !available_parents.is_empty() {
                let chosen = available_parents[rng.gen_range(0..available_parents.len())];
                genes.push(chosen.genes[i].clone());
            }
        }

        let generation = parents[0].generation;

        // Build parent info: one mitochondrial, rest genetic
        let mut parent_info = vec![ParentInfo::mitochondrial(
            mito_parent
                .id
                .expect("Mitochondrial parent must have an ID"),
            mito_parent.lineage_id,
        )];

        for (idx, parent) in parents.iter().enumerate() {
            if idx != mito_parent_idx {
                parent_info.push(ParentInfo::genetic(
                    parent.id.expect("Genetic parent must have an ID"),
                    parent.lineage_id,
                ));
            }
        }

        DNA::with_parents(genes, generation, mito_parent.lineage_id, parent_info)
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

        // Randomly choose which parent is mitochondrial (donates template library)
        let (mito_parent, gene_parent) = if rng.r#gen::<bool>() {
            (parent1, parent2)
        } else {
            (parent2, parent1)
        };

        let boundaries1 = self.find_template_boundaries(parent1);
        let boundaries2 = self.find_template_boundaries(parent2);

        // Choose crossover points from boundaries
        let point1 = boundaries1[rng.gen_range(0..boundaries1.len())];
        let point2 = boundaries2[rng.gen_range(0..boundaries2.len())];

        // Build child: parent1[..point1] + parent2[point2..]
        let mut genes = Vec::new();
        genes.extend_from_slice(&parent1.genes[..point1]);
        genes.extend_from_slice(&parent2.genes[point2..]);

        // Create child in mitochondrial parent's lineage with parent tracking
        let parent_info = vec![
            ParentInfo::mitochondrial(
                mito_parent
                    .id
                    .expect("Mitochondrial parent must have an ID"),
                mito_parent.lineage_id,
            ),
            ParentInfo::genetic(
                gene_parent.id.expect("Genetic parent must have an ID"),
                gene_parent.lineage_id,
            ),
        ];

        DNA::with_parents(
            genes,
            parent1.generation,
            mito_parent.lineage_id,
            parent_info,
        )
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
                    DNA::empty(0, 0)
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
    use crate::dna::{Argument, Gene};

    #[test]
    fn test_single_point_crossover() {
        let mut parent1 = DNA::empty(0, 0); // generation 0, lineage 0
        parent1.id = Some(1); // Assign ID for crossover
        parent1.push_gene(Gene::primitive(0, vec![Argument::Register(0)]));
        parent1.push_gene(Gene::primitive(1, vec![Argument::Register(1)]));
        parent1.push_gene(Gene::primitive(2, vec![Argument::Register(2)]));

        let mut parent2 = DNA::empty(0, 1); // generation 0, lineage 1
        parent2.id = Some(2); // Assign ID for crossover
        parent2.push_gene(Gene::primitive(3, vec![Argument::Register(3)]));
        parent2.push_gene(Gene::primitive(4, vec![Argument::Register(4)]));

        let crossover = Crossover::single_point();
        let child = crossover.cross(&parent1, &parent2);

        // Child should have genes from both parents
        assert!(!child.is_empty());
        assert!(child.fitness.is_none());
        // Child should have parent tracking
        assert!(child.has_parents());
        assert_eq!(child.parents.len(), 2);
        // One parent should be mitochondrial, one genetic
        assert_eq!(
            child
                .parents
                .iter()
                .filter(|p| p.is_mitochondrial())
                .count(),
            1
        );
        assert_eq!(child.parents.iter().filter(|p| p.is_genetic()).count(), 1);
        // Child should be in mitochondrial parent's lineage
        let mito_parent = child.mitochondrial_parent().unwrap();
        assert_eq!(child.lineage_id, mito_parent.lineage_id);
    }

    #[test]
    fn test_two_point_crossover() {
        let mut parent1 = DNA::empty(0, 0);
        parent1.id = Some(10);
        parent1.push_gene(Gene::primitive(0, vec![]));
        parent1.push_gene(Gene::primitive(1, vec![]));
        parent1.push_gene(Gene::primitive(2, vec![]));

        let mut parent2 = DNA::empty(0, 1);
        parent2.id = Some(20);
        parent2.push_gene(Gene::primitive(3, vec![]));
        parent2.push_gene(Gene::primitive(4, vec![]));
        parent2.push_gene(Gene::primitive(5, vec![]));

        let crossover = Crossover::two_point();
        let child = crossover.cross(&parent1, &parent2);

        assert!(!child.is_empty());
        assert!(child.has_parents());
        assert_eq!(child.parents.len(), 2);
    }

    #[test]
    fn test_uniform_crossover() {
        let mut parent1 = DNA::empty(0, 0);
        parent1.id = Some(100);
        parent1.push_gene(Gene::primitive(0, vec![]));
        parent1.push_gene(Gene::primitive(1, vec![]));

        let mut parent2 = DNA::empty(0, 1);
        parent2.id = Some(200);
        parent2.push_gene(Gene::primitive(2, vec![]));
        parent2.push_gene(Gene::primitive(3, vec![]));

        let mut parent3 = DNA::empty(0, 2);
        parent3.id = Some(300);
        parent3.push_gene(Gene::primitive(4, vec![]));
        parent3.push_gene(Gene::primitive(5, vec![]));

        let crossover = Crossover::uniform(3);
        let child = crossover.cross_many(&[&parent1, &parent2, &parent3]);

        assert_eq!(child.len(), 2);
        assert!(child.has_parents());
        assert_eq!(child.parents.len(), 3); // One mitochondrial, two genetic
        assert_eq!(
            child
                .parents
                .iter()
                .filter(|p| p.is_mitochondrial())
                .count(),
            1
        );
        assert_eq!(child.parents.iter().filter(|p| p.is_genetic()).count(), 2);
    }

    #[test]
    fn test_template_aware_boundaries() {
        let mut dna = DNA::empty(0, 0);
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
