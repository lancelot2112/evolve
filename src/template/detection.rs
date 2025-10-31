//! Template Detection
//!
//! Scans DNA sequences to find genes between TEMPLATE_START and TEMPLATE_END markers.
//!
//! This module provides the detect_templates() function which identifies potential
//! templates in evolved DNA. Only non-empty sequences between valid markers are detected.
//! The function returns the start/end indices and hash of each template found.

use crate::dna::{Gene, OperationId};
use super::hashing::hash_genes;

/// Detect templates in DNA (sequences between TEMPLATE_START and TEMPLATE_END)
/// Returns list of (start_index, end_index, hash) for found templates
pub fn detect_templates(genes: &[Gene]) -> Vec<(usize, usize, u64)> {
    const TEMPLATE_START_ID: u16 = 10;  // ID of TEMPLATE_START primitive
    const TEMPLATE_END_ID: u16 = 11;     // ID of TEMPLATE_END primitive

    let mut templates = Vec::new();
    let mut start_idx = None;

    for (i, gene) in genes.iter().enumerate() {
        match gene.operation {
            OperationId::Primitive(id) if id == TEMPLATE_START_ID => {
                // Found start marker
                start_idx = Some(i);
            }
            OperationId::Primitive(id) if id == TEMPLATE_END_ID => {
                // Found end marker
                if let Some(start) = start_idx {
                    // Extract genes between markers (excluding the markers themselves)
                    let template_genes = &genes[start + 1..i];
                    if !template_genes.is_empty() {
                        let hash = hash_genes(template_genes);
                        templates.push((start, i, hash));
                    }
                    start_idx = None;
                }
            }
            _ => {}
        }
    }

    templates
}
