//! Inspect Command
//!
//! Displays detailed information about a specific generation from evolution history.
//!
//! ## Lineage Support
//! - Displays lineage_id for each DNA
//! - Shows parent relationships (Mitochondrial and Genetic)
//! - TODO: Add lineage statistics when LineageRegistry is available in storage

use crate::storage::EvolutionHistory;

pub fn cmd_inspect(generation: u32, file: String, top: usize) {
    let history = match EvolutionHistory::load_from_file_auto(&file) {
        Ok(h) => h,
        Err(e) => {
            eprintln!("Failed to load history from {}: {}", file, e);
            std::process::exit(1);
        }
    };

    let record = match history.get_generation(generation) {
        Some(r) => r,
        None => {
            eprintln!("Generation {} not found in history", generation);
            std::process::exit(1);
        }
    };

    println!("=== Generation {} ===", generation);
    println!("Population size: {}", record.population.len());
    println!("Best fitness: {:.4}", record.best_fitness);
    println!("Average fitness: {:.4}", record.average_fitness);
    println!("Templates created: {}", record.templates_created.len());

    // TODO: Add lineage statistics once LineageRegistry is available in storage
    // - Number of active lineages
    // - Lineage diversity metrics
    // - Template library sizes per lineage

    println!("\nTop {} individuals:", top);

    let mut sorted_population = record.population.clone();
    sorted_population.sort_by(|a, b| {
        b.fitness
            .unwrap_or(0.0)
            .partial_cmp(&a.fitness.unwrap_or(0.0))
            .unwrap_or(std::cmp::Ordering::Equal)
    });

    for (i, dna) in sorted_population.iter().take(top).enumerate() {
        let parent_info = if dna.has_parents() {
            let mito_parent = dna.mitochondrial_parent();
            let genetic_parents = dna.genetic_parents();

            let mito_str = mito_parent
                .map(|p| format!("M:{}", p.dna_id))
                .unwrap_or_else(|| "M:none".to_string());

            let genetic_str = if !genetic_parents.is_empty() {
                format!(
                    " G:[{}]",
                    genetic_parents
                        .iter()
                        .map(|p| p.dna_id.to_string())
                        .collect::<Vec<_>>()
                        .join(",")
                )
            } else {
                String::new()
            };

            format!(" Parents:{}{}", mito_str, genetic_str)
        } else {
            " Parents:none".to_string()
        };

        println!(
            "  {}: DNA ID={} Lineage={} Fitness={:.4} Length={}{}",
            i + 1,
            dna.id.unwrap_or(0),
            dna.lineage_id,
            dna.fitness.unwrap_or(0.0),
            dna.len(),
            parent_info
        );
    }
}
