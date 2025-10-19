//! Replay Command
//!
//! Executes a specific DNA with custom input values to observe its behavior.

use crate::execution::Executor;
use crate::primitives::PrimitiveRegistry;
use crate::storage::EvolutionHistory;
use crate::template::TemplateRegistry;

pub fn cmd_replay(id: u64, file: String, input: Option<String>) {
    let history = match EvolutionHistory::load_from_file_auto(&file) {
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

    println!("Replaying DNA ID {}", id);
    println!("  Lineage: {}", dna.lineage_id);
    println!("  Generation: {}", dna.generation);
    println!("  Input: {:?}", input_values);

    let executor = Executor::with_defaults();
    let primitives = PrimitiveRegistry::with_standard_primitives();

    // TODO: Templates now live in lineages, not DNA
    // For proper replay, we need to:
    // 1. Load the LineageRegistry from storage
    // 2. Get the template library for dna.lineage_id
    // 3. Pass it to executor.execute()
    //
    // For now, use an empty template registry (DNA with no templates will still work)
    let template_registry = TemplateRegistry::new();
    eprintln!("WARNING: Using empty template registry - DNA with templates will fail");
    eprintln!("TODO: Load lineage template registry from storage");

    let result = executor.execute(dna, input_values, &primitives, &template_registry);

    println!("\nExecution Result:");
    println!("  Success: {}", result.success);
    println!("  Output: {:?}", result.output);
    if let Some(err) = result.error {
        println!("  Error: {}", err);
    }
}
