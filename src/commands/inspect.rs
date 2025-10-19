//! Inspect Command
//!
//! Displays detailed information about a specific generation from evolution history.

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

    println!("\nTop {} individuals:", top);

    let mut sorted_population = record.population.clone();
    sorted_population.sort_by(|a, b| {
        b.fitness
            .unwrap_or(0.0)
            .partial_cmp(&a.fitness.unwrap_or(0.0))
            .unwrap_or(std::cmp::Ordering::Equal)
    });

    for (i, dna) in sorted_population.iter().take(top).enumerate() {
        println!(
            "  {}: DNA ID={} Fitness={:.4} Length={}",
            i + 1,
            dna.id.unwrap_or(0),
            dna.fitness.unwrap_or(0.0),
            dna.len()
        );
    }
}
