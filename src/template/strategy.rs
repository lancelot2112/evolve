//! Template Creation Strategy
//!
//! Defines when and how templates should be created from DNA sequences.
//!
//! This module provides the TemplateCreationStrategy enum which allows different
//! policies for template promotion: top percentile, generational best, fitness
//! threshold, or manual promotion. This flexibility enables experimentation with
//! different evolutionary pressures.

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
