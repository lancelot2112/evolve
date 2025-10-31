//! Template Module
//!
//! Algorithm-evolved indexed sequences that provide configuration space compression.
//!
//! Templates are successful DNA sequences between TEMPLATE_START and TEMPLATE_END markers.
//! They are identified by hashing their gene sequence and stored in a HashMap for
//! deduplication. Each DNA maintains its own template library for lineage-local
//! template inheritance (mitochondrial-style).

mod template;
mod hashing;
mod detection;
mod strategy;

// Re-export core types
pub use template::{Template, TemplateRegistry};
pub use hashing::hash_genes;
pub use detection::detect_templates;
pub use strategy::TemplateCreationStrategy;
