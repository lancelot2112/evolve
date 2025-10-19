//! Show Templates Command
//!
//! Displays all evolved templates organized by lineage.
//!
//! TODO: This command needs a major rewrite for the new lineage-based template system:
//! - Load LineageRegistry from storage (currently only JSON format exists)
//! - Display templates grouped by lineage (lineage ID, origin generation, alive/extinct status)
//! - Show template inheritance patterns (which templates each lineage has)
//! - Display template usage statistics per lineage
//! - Show cross-lineage template similarities (if any templates happen to be identical)
//!
//! For now, this is stubbed out to compile with the new architecture.

pub fn cmd_show_templates(_file: String) {
    // TODO: Load lineage registry from storage file
    // let history = EvolutionHistory::load(&file).unwrap();
    // let lineage_registry = history.lineage_registry;

    // TODO: Display templates grouped by lineage
    // for lineage in lineage_registry.all() {
    //     println!("Lineage {} (Gen {}, {})",
    //         lineage.id,
    //         lineage.origin_generation,
    //         if lineage.alive { "alive" } else { "extinct" }
    //     );
    //     println!("  Templates: {}", lineage.template_library.count());
    //     // Display each template with ID, genes, first seen generation, fitness
    // }

    // TODO: Show template inheritance patterns
    // Display which templates were inherited from progenitor vs evolved later
    // Show template "family trees" across lineages

    println!("Note: Template system has been refactored to lineage-based architecture.");
    println!("Templates now live in Lineage objects, not individual DNA.");
    println!();
    println!("This command needs implementation to:");
    println!("  - Load LineageRegistry from storage");
    println!("  - Display templates grouped by lineage");
    println!("  - Show template inheritance patterns");
    println!();
    println!("See LINEAGE_REFACTOR_PROGRESS.md for details.");
}
