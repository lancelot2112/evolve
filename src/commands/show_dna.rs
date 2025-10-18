//! Show DNA Command
//!
//! Displays detailed information about a specific DNA sequence including its genes.

use crate::cli::format_dna;
use crate::primitives::PrimitiveRegistry;
use crate::storage::EvolutionHistory;

pub fn cmd_show_dna(id: u64, file: String) {
    let history = match EvolutionHistory::load_from_file(&file) {
        Ok(h) => h,
        Err(e) => {
            eprintln!("Failed to load history from {}: {}", file, e);
            std::process::exit(1);
        }
    };

    let dna = match history.get_dna(id) {
        Some(d) => d,
        None => {
            eprintln!("DNA with ID {} not found", id);
            std::process::exit(1);
        }
    };

    let primitives = PrimitiveRegistry::with_standard_primitives();
    let templates = crate::template::TemplateRegistry::new();

    println!("{}", format_dna(dna, &primitives, &templates));
}
