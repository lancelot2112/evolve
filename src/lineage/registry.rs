//! Lineage Registry
//!
//! Manages the collection of all lineages and provides lookup and statistics.

use super::lineage::Lineage;
use serde::{Deserialize, Serialize};

/// Registry to manage all lineages
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct LineageRegistry {
    lineages: Vec<Lineage>,
}

impl LineageRegistry {
    /// Create a new empty registry
    pub fn new() -> Self {
        Self {
            lineages: Vec::new(),
        }
    }

    /// Register a new lineage
    pub fn register(&mut self, lineage: Lineage) {
        self.lineages.push(lineage);
    }

    /// Get a lineage by ID
    pub fn get(&self, id: u64) -> Option<&Lineage> {
        self.lineages.iter().find(|l| l.id == id)
    }

    /// Get a mutable lineage by ID
    pub fn get_mut(&mut self, id: u64) -> Option<&mut Lineage> {
        self.lineages.iter_mut().find(|l| l.id == id)
    }

    /// Get all lineages
    pub fn all(&self) -> &[Lineage] {
        &self.lineages
    }

    /// Get count of all lineages
    pub fn count(&self) -> usize {
        self.lineages.len()
    }

    /// Get count of alive lineages
    pub fn alive_count(&self) -> usize {
        self.lineages.iter().filter(|l| l.alive).count()
    }

    /// Get count of extinct lineages
    pub fn extinct_count(&self) -> usize {
        self.lineages.iter().filter(|l| !l.alive).count()
    }

    /// Mark lineages as alive or extinct based on which ones have members in the current generation
    pub fn update_alive_status(&mut self, current_generation_lineages: &[u64]) {
        for lineage in &mut self.lineages {
            lineage.alive = current_generation_lineages.contains(&lineage.id);
        }
    }

    /// Get best fitness across all lineages
    pub fn best_fitness(&self) -> f64 {
        self.lineages
            .iter()
            .map(|l| l.best_fitness)
            .max_by(|a, b| a.partial_cmp(b).unwrap_or(std::cmp::Ordering::Equal))
            .unwrap_or(0.0)
    }

    /// Get lineages alive in the current generation
    pub fn alive_lineages(&self) -> Vec<&Lineage> {
        self.lineages.iter().filter(|l| l.alive).collect()
    }

    /// Get lineages that have gone extinct
    pub fn extinct_lineages(&self) -> Vec<&Lineage> {
        self.lineages.iter().filter(|l| !l.alive).collect()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_lineage_registry_creation() {
        let registry = LineageRegistry::new();
        assert_eq!(registry.count(), 0);
    }

    #[test]
    fn test_lineage_registry_register() {
        let mut registry = LineageRegistry::new();

        let lineage1 = Lineage::new(0, 0);
        let lineage2 = Lineage::new(1, 0);

        registry.register(lineage1);
        registry.register(lineage2);

        assert_eq!(registry.count(), 2);
        assert_eq!(registry.alive_count(), 2);
        assert_eq!(registry.extinct_count(), 0);
    }

    #[test]
    fn test_lineage_registry_get() {
        let mut registry = LineageRegistry::new();

        registry.register(Lineage::new(0, 0));
        registry.register(Lineage::new(1, 0));

        assert!(registry.get(0).is_some());
        assert!(registry.get(1).is_some());
        assert!(registry.get(2).is_none());
    }

    #[test]
    fn test_lineage_registry_extinction() {
        let mut registry = LineageRegistry::new();

        registry.register(Lineage::new(0, 0));
        registry.register(Lineage::new(1, 0));

        // Mark lineage 0 as extinct
        registry.get_mut(0).unwrap().mark_extinct();

        assert_eq!(registry.alive_count(), 1);
        assert_eq!(registry.extinct_count(), 1);
    }

    #[test]
    fn test_lineage_registry_update_status() {
        let mut registry = LineageRegistry::new();

        registry.register(Lineage::new(0, 0));
        registry.register(Lineage::new(1, 0));
        registry.register(Lineage::new(2, 0));

        // Simulate that only lineages 0 and 2 have members in current gen
        registry.update_alive_status(&[0, 2]);

        assert_eq!(registry.alive_count(), 2);
        assert!(registry.get(0).unwrap().alive);
        assert!(!registry.get(1).unwrap().alive); // Extinct
        assert!(registry.get(2).unwrap().alive);
    }

    #[test]
    fn test_lineage_registry_best_fitness() {
        let mut registry = LineageRegistry::new();

        let mut lineage1 = Lineage::new(0, 0);
        lineage1.update_best_fitness(0.5);

        let mut lineage2 = Lineage::new(1, 0);
        lineage2.update_best_fitness(0.9);

        let mut lineage3 = Lineage::new(2, 0);
        lineage3.update_best_fitness(0.3);

        registry.register(lineage1);
        registry.register(lineage2);
        registry.register(lineage3);

        assert_eq!(registry.best_fitness(), 0.9);
    }

    #[test]
    fn test_lineage_registry_alive_extinct_lists() {
        let mut registry = LineageRegistry::new();

        let lineage1 = Lineage::new(0, 0);
        let mut lineage2 = Lineage::new(1, 0);
        lineage2.mark_extinct();

        registry.register(lineage1);
        registry.register(lineage2);

        let alive = registry.alive_lineages();
        let extinct = registry.extinct_lineages();

        assert_eq!(alive.len(), 1);
        assert_eq!(extinct.len(), 1);
        assert_eq!(alive[0].id, 0);
        assert_eq!(extinct[0].id, 1);
    }
}
