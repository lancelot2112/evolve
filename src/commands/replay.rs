//! Replay Command
//!
//! Executes a specific DNA with custom input values to observe its behavior.

use crate::execution::Executor;
use crate::primitives::PrimitiveRegistry;
use crate::storage::EvolutionHistory;

pub fn cmd_replay(id: u64, file: String, input: Option<String>) {
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

    let input_values = if let Some(input_str) = input {
        input_str
            .split(',')
            .filter_map(|s| s.trim().parse::<i64>().ok())
            .collect()
    } else {
        println!("No input provided, using default: [5]");
        vec![5]
    };

    println!("Replaying DNA ID {} with input: {:?}", id, input_values);

    let executor = Executor::with_defaults();
    let primitives = PrimitiveRegistry::with_standard_primitives();

    let result = executor.execute(dna, input_values, &primitives);

    println!("\nExecution Result:");
    println!("  Success: {}", result.success);
    println!("  Output: {:?}", result.output);
    if let Some(err) = result.error {
        println!("  Error: {}", err);
    }
}
