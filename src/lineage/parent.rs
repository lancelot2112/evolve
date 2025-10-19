//! Parent Information
//!
//! Tracks the role each parent plays in creating offspring DNA.

use serde::{Deserialize, Serialize};

/// Role a parent plays in creating offspring
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum ParentRole {
    /// Provided both genes AND template library (child joins this lineage)
    Mitochondrial,
    /// Provided genes only (child does NOT join this lineage)
    Genetic,
}

/// Information about a parent DNA
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ParentInfo {
    /// Parent DNA ID
    pub dna_id: u64,
    /// Parent's lineage ID
    pub lineage_id: u64,
    /// Role this parent played
    pub role: ParentRole,
}

impl ParentInfo {
    /// Create a mitochondrial parent info
    pub fn mitochondrial(dna_id: u64, lineage_id: u64) -> Self {
        Self {
            dna_id,
            lineage_id,
            role: ParentRole::Mitochondrial,
        }
    }

    /// Create a genetic parent info
    pub fn genetic(dna_id: u64, lineage_id: u64) -> Self {
        Self {
            dna_id,
            lineage_id,
            role: ParentRole::Genetic,
        }
    }

    /// Check if this is a mitochondrial parent
    pub fn is_mitochondrial(&self) -> bool {
        matches!(self.role, ParentRole::Mitochondrial)
    }

    /// Check if this is a genetic parent
    pub fn is_genetic(&self) -> bool {
        matches!(self.role, ParentRole::Genetic)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parent_info_mitochondrial() {
        let mito = ParentInfo::mitochondrial(10, 0);
        assert_eq!(mito.dna_id, 10);
        assert_eq!(mito.lineage_id, 0);
        assert!(mito.is_mitochondrial());
        assert!(!mito.is_genetic());
    }

    #[test]
    fn test_parent_info_genetic() {
        let genetic = ParentInfo::genetic(20, 1);
        assert_eq!(genetic.dna_id, 20);
        assert_eq!(genetic.lineage_id, 1);
        assert!(!genetic.is_mitochondrial());
        assert!(genetic.is_genetic());
    }
}
