//! Template Marker Primitives
//!
//! Special marker genes (TEMPLATE_START, TEMPLATE_END) that define template boundaries.
//!
//! These primitives don't execute any operations - they serve only to mark the
//! beginning and end of template sequences. The template detection system scans
//! for these markers to identify reusable gene sequences.

use super::context::{ExecutionContext, ExecutionError};
use super::trait_def::Primitive;
use crate::dna::Argument;

/// TEMPLATE_START: Marker for template start (doesn't execute)
pub struct TemplateStart;

impl Primitive for TemplateStart {
    fn execute(
        &self,
        _args: &[Argument],
        _context: &mut ExecutionContext,
    ) -> Result<(), ExecutionError> {
        // Marker only - no execution
        Ok(())
    }

    fn arg_count(&self) -> usize {
        0
    }
    fn name(&self) -> &str {
        "TEMPLATE_START"
    }
    fn description(&self) -> &str {
        "Marks the beginning of a template sequence"
    }
}

/// TEMPLATE_END: Marker for template end (doesn't execute)
pub struct TemplateEnd;

impl Primitive for TemplateEnd {
    fn execute(
        &self,
        _args: &[Argument],
        _context: &mut ExecutionContext,
    ) -> Result<(), ExecutionError> {
        // Marker only - no execution
        Ok(())
    }

    fn arg_count(&self) -> usize {
        0
    }
    fn name(&self) -> &str {
        "TEMPLATE_END"
    }
    fn description(&self) -> &str {
        "Marks the end of a template sequence"
    }
}
