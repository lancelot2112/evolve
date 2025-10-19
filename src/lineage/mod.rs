//! Lineage Module
//!
//! Tracks matrilineal lines of DNA that share template libraries through inheritance.
//!
//! A lineage is founded by a progenitor DNA in generation 0 and continues through
//! mitochondrial inheritance (template library + genes). DNA from other lineages can
//! contribute genes through crossover (genetic role) without the child joining that lineage.

mod lineage;
mod parent;
mod registry;

// Re-export core types
pub use lineage::Lineage;
pub use parent::{ParentInfo, ParentRole};
pub use registry::LineageRegistry;
