//! Run Command
//!
//! Executes the evolution process for a specified problem over multiple generations.

use crate::cli::{EvolutionRunner, format_dna};
use crate::execution::{ExactMatchFitness, Executor};
use crate::primitives::PrimitiveRegistry;

pub fn cmd_evolve(problem: String, generations: u32, population_size: usize, output: String, verbose: bool) {
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
    let history = runner.run(generations, &test_cases, &fitness_fn, verbose);

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

        println!("\nTest Results:");
        for (input, expected) in &test_cases {
            let result = executor.execute(
                best_dna,
                input.clone(),
                &primitive_registry,
            );
            println!(
                "  Input: {:?} -> Output: {:?} (Expected: {:?}) {}",
                input,
                result.output,
                expected,
                if result.output == *expected { "✓" } else { "✗" }
            );
        }
    }

    // Save history
    if let Err(e) = history.save_to_file(&output) {
        eprintln!("\nFailed to save history: {}", e);
    } else {
        println!("\nEvolution history saved to {}", output);
        println!("Total DNA created: {}", history.records.iter().map(|r| r.population.len()).sum::<usize>());
    }
}
