//! Gene Sequence Hashing
//!
//! Provides the hash_genes function for creating unique identifiers from gene sequences.
//!
//! The hash includes both the operation ID and all arguments, ensuring that even
//! slight variations in a sequence produce different hashes. This enables precise
//! deduplication while allowing similar-but-different sequences to coexist.

use crate::dna::{Gene, OperationId};
use std::collections::hash_map::DefaultHasher;
use std::hash::{Hash, Hasher};

/// Hash a sequence of genes to create a unique identifier
pub fn hash_genes(genes: &[Gene]) -> u64 {
    let mut hasher = DefaultHasher::new();

    for gene in genes {
        // Hash the operation
        match gene.operation {
            OperationId::Primitive(id) => {
                0u8.hash(&mut hasher); // Tag for primitive
                id.hash(&mut hasher);
            }
            OperationId::Template(hash) => {
                1u8.hash(&mut hasher); // Tag for template
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
