//! Lineage Definition
//!
//! A matrilineal line tracking DNA that share template libraries through inheritance.

use crate::template::TemplateRegistry;
use serde::{Deserialize, Serialize};

/// A matrilineal lineage tracking template library inheritance
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Lineage {
    /// Unique lineage ID (same as progenitor DNA ID)
    pub id: u64,

    /// Generation this lineage was founded
    pub origin_generation: u32,

    /// Best fitness ever achieved by any member
    pub best_fitness: f64,

    /// Current alive status (false = extinct)
    pub alive: bool,

    /// Template library shared by all members (grows over time)
    #[serde(skip)] // Don't serialize - will be reconstructed from DNA
    pub template_library: TemplateRegistry,

    /// All DNA IDs in this lineage (chronological order)
    /// members[0] is always the progenitor
    pub members: Vec<u64>,
}

impl Lineage {
    /// Create a new lineage with a progenitor DNA
    pub fn new(progenitor_id: u64, origin_generation: u32) -> Self {
        Self {
            id: progenitor_id,
            origin_generation,
            best_fitness: 0.0,
            alive: true,
            template_library: TemplateRegistry::new(),
            members: vec![progenitor_id],
        }
    }

    /// Get the progenitor DNA ID
    pub fn progenitor(&self) -> Option<u64> {
        self.members.first().copied()
    }

    /// Add a new member to this lineage
    pub fn add_member(&mut self, dna_id: u64) {
        if !self.members.contains(&dna_id) {
            self.members.push(dna_id);
        }
    }

    /// Update best fitness if new fitness is higher
    pub fn update_best_fitness(&mut self, fitness: f64) {
        if fitness > self.best_fitness {
            self.best_fitness = fitness;
        }
    }

    /// Mark this lineage as extinct
    pub fn mark_extinct(&mut self) {
        self.alive = false;
    }

    /// Mark this lineage as alive
    pub fn mark_alive(&mut self) {
        self.alive = true;
    }

    /// Get the number of members in this lineage
    pub fn member_count(&self) -> usize {
        self.members.len()
    }

    /// Get all member IDs
    pub fn member_ids(&self) -> &[u64] {
        &self.members
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_lineage_creation() {
        let lineage = Lineage::new(0, 0);
        assert_eq!(lineage.id, 0);
        assert_eq!(lineage.origin_generation, 0);
        assert_eq!(lineage.progenitor(), Some(0));
        assert_eq!(lineage.member_count(), 1);
        assert!(lineage.alive);
    }

    #[test]
    fn test_lineage_add_member() {
        let mut lineage = Lineage::new(0, 0);
        lineage.add_member(1);
        lineage.add_member(2);

        assert_eq!(lineage.member_count(), 3);
        assert_eq!(lineage.member_ids(), &[0, 1, 2]);
    }

    #[test]
    fn test_lineage_add_duplicate_member() {
        let mut lineage = Lineage::new(0, 0);
        lineage.add_member(1);
        lineage.add_member(1); // Duplicate

        assert_eq!(lineage.member_count(), 2); // Should not duplicate
    }

    #[test]
    fn test_lineage_update_fitness() {
        let mut lineage = Lineage::new(0, 0);

        lineage.update_best_fitness(0.5);
        assert_eq!(lineage.best_fitness, 0.5);

        lineage.update_best_fitness(0.3); // Lower, should not update
        assert_eq!(lineage.best_fitness, 0.5);

        lineage.update_best_fitness(0.8); // Higher, should update
        assert_eq!(lineage.best_fitness, 0.8);
    }

    #[test]
    fn test_lineage_alive_status() {
        let mut lineage = Lineage::new(0, 0);
        assert!(lineage.alive);

        lineage.mark_extinct();
        assert!(!lineage.alive);

        lineage.mark_alive();
        assert!(lineage.alive);
    }
}
