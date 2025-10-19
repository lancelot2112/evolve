//! Stats Command
//!
//! Displays statistics about the evolution run including fitness progression.

use crate::storage::EvolutionHistory;

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
    println!("Total DNA created: {}",
        history.records.iter().map(|r| r.population.len()).sum::<usize>());

    if let Some(best) = history.best_dna() {
        println!("\nBest DNA ever:");
        println!("  ID: {}", best.id.unwrap_or(0));
        println!("  Fitness: {:.4}", best.fitness.unwrap_or(0.0));
        println!("  Generation: {}", best.generation);
        println!("  Length: {} genes", best.len());
    }

    println!("\nFitness progression:");
    for record in &history.records {
        println!(
            "  Gen {}: Best={:.4} Avg={:.4}",
            record.generation, record.best_fitness, record.average_fitness
        );
    }
}
