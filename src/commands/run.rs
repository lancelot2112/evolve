//! Run Command
//!
//! Executes the evolution process for a specified problem over multiple generations.

use crate::cli::{EvolutionRunner, format_dna};
use crate::execution::{ExactMatchFitness, Executor};
use crate::primitives::PrimitiveRegistry;

pub fn cmd_evolve(
    problem: String,
    generations: u32,
    population_size: usize,
    output: String,
    verbose: bool,
    save_interval: u32,
    keep_in_memory: usize,
) {
    println!("Evolve - Evolutionary Algorithm System");
    println!("======================================\n");

    // Define test cases based on problem
    let test_cases = match problem.as_str() {
        "double" => {
            println!("Problem: Evolve a program that doubles its input\n");
            vec![
                (vec![0], vec![0]),
                (vec![1], vec![2]),
                (vec![5], vec![10]),
                (vec![10], vec![20]),
                (vec![-3], vec![-6]),
            ]
        }
        _ => {
            eprintln!("Unknown problem: {}. Available: double", problem);
            std::process::exit(1);
        }
    };

    let fitness_fn = ExactMatchFitness;

    let mut runner = EvolutionRunner::new();
    runner.population_size = population_size;
    runner.save_interval = save_interval;
    runner.keep_in_memory = keep_in_memory;

    if save_interval > 0 {
        println!(
            "Incremental save enabled: saving every {} generations",
            save_interval
        );
        if keep_in_memory > 0 {
            println!(
                "Memory management: keeping last {} generations in RAM\n",
                keep_in_memory
            );
        }
    }

    let history = runner.run(
        generations,
        &test_cases,
        &fitness_fn,
        verbose,
        Some(&output),
    );

    // Print best solution
    if let Some(best_dna) = history.best_dna() {
        println!("\n=== Best Solution Found ===");
        println!("DNA ID: {}", best_dna.id.unwrap_or(0));
        println!("Fitness: {:.4}", best_dna.fitness.unwrap_or(0.0));
        println!("Length: {} genes", best_dna.len());
        println!("Generation: {}", best_dna.generation);

        // Test the best solution
        let executor = Executor::with_defaults();
        let primitive_registry = PrimitiveRegistry::with_standard_primitives();
        // TODO: Use the lineage's template library instead of empty registry
        let template_registry = crate::template::TemplateRegistry::new();

        println!("\nTest Results:");
        for (input, expected) in &test_cases {
            let result = executor.execute(
                best_dna,
                input.clone(),
                &primitive_registry,
                &template_registry,
            );
            println!(
                "  Input: {:?} -> Output: {:?} (Expected: {:?}) {}",
                input,
                result.output,
                expected,
                if result.output == *expected {
                    "✓"
                } else {
                    "✗"
                }
            );
        }
    }

    // Save history (only if incremental save was not used)
    if save_interval == 0 {
        // Save as traditional JSON file
        let json_output = output.replace(".jsonl", ".json");
        if let Err(e) = history.save_to_file(&json_output) {
            eprintln!("\nFailed to save history: {}", e);
        } else {
            println!("\nEvolution history saved to {}", json_output);
            println!(
                "Total DNA created: {}",
                history
                    .records
                    .iter()
                    .map(|r| r.population.len())
                    .sum::<usize>()
            );
        }
    } else {
        println!(
            "\nEvolution complete! History saved incrementally to {}",
            output
        );
        println!("Generations in memory: {}", history.records.len());
        if keep_in_memory > 0 {
            println!("(Older generations were cleared from memory but are saved to disk)");
        }
    }
}
