//! Stats Command
//!
//! Displays statistics about the evolution run including fitness progression.

use crate::storage::EvolutionHistory;
use std::collections::HashSet;

pub fn cmd_stats(file: String) {
    let history = match EvolutionHistory::load_from_file_auto(&file) {
        Ok(h) => h,
        Err(e) => {
            eprintln!("Failed to load history from {}: {}", file, e);
            std::process::exit(1);
        }
    };

    println!("=== Evolution Statistics ===\n");
    println!("Total generations: {}", history.records.len());
    println!(
        "Total DNA created: {}",
        history
            .records
            .iter()
            .map(|r| r.population.len())
            .sum::<usize>()
    );

    // Lineage statistics
    // TODO: Once lineage registry is integrated into storage, add:
    // - Total lineages created
    // - Active lineages (have members in latest generation)
    // - Extinct lineages (no members in latest generation)
    // - Lineage diversity over time
    // - Average lineage lifespan (generations from birth to extinction)
    // - Lineage fitness comparison (best fitness per lineage)
    // - Template library size per lineage
    // - Cross-lineage template sharing metrics

    // For now, compute basic lineage stats from DNA
    let unique_lineages: HashSet<u64> = history
        .records
        .iter()
        .flat_map(|r| r.population.iter().map(|dna| dna.lineage_id))
        .collect();

    println!("\nLineage statistics:");
    println!("  Total lineages observed: {}", unique_lineages.len());

    // TODO: Add active vs extinct lineage count once we track lineage lifecycle
    // TODO: Add lineage best fitness comparison
    // TODO: Add lineage diversity metrics (how evenly distributed is population across lineages)

    if let Some(best) = history.best_dna() {
        println!("\nBest DNA ever:");
        println!("  ID: {}", best.id.unwrap_or(0));
        println!("  Lineage: {}", best.lineage_id);
        println!("  Fitness: {:.4}", best.fitness.unwrap_or(0.0));
        println!("  Generation: {}", best.generation);
        println!("  Length: {} genes", best.len());

        // TODO: Add parent information display
        // TODO: Show mitochondrial vs genetic parents
        // TODO: Show lineage inheritance path
    }

    println!("\nFitness progression:");
    for record in &history.records {
        println!(
            "  Gen {}: Best={:.4} Avg={:.4}",
            record.generation, record.best_fitness, record.average_fitness
        );
    }

    // TODO: Add per-lineage fitness progression graphs
    // TODO: Add template usage statistics (how often templates vs primitives are used)
    // TODO: Add gene diversity metrics (unique gene sequences)
}
