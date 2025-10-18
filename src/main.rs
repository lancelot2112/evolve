/// Evolve - Evolutionary Algorithm System
///
/// This system allows algorithms to evolve from user-defined primitives into
/// complex behaviors through mutation and crossover, with successful patterns
/// being saved as indexed templates for efficient reuse.

mod dna;
mod primitives;
mod template;
mod evolution;
mod execution;
mod storage;
mod cli;

use clap::{Parser, Subcommand};
use cli::{EvolutionRunner, format_dna};
use execution::{ExactMatchFitness, Executor};
use primitives::PrimitiveRegistry;
use storage::EvolutionHistory;

#[derive(Parser)]
#[command(name = "evolve")]
#[command(about = "Evolutionary Algorithm System", long_about = None)]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    /// Run evolution for a specific problem
    Run {
        /// Problem to solve: "double" (multiply input by 2)
        #[arg(default_value = "double")]
        problem: String,

        /// Number of generations
        #[arg(short, long, default_value_t = 50)]
        generations: u32,

        /// Population size
        #[arg(short, long, default_value_t = 100)]
        population: usize,

        /// Output file for history
        #[arg(short, long, default_value = "evolution_history.json")]
        output: String,

        /// Verbose output
        #[arg(short, long)]
        verbose: bool,
    },

    /// Inspect a specific generation from history
    Inspect {
        /// Generation number to inspect
        generation: u32,

        /// History file to load
        #[arg(short, long, default_value = "evolution_history.json")]
        file: String,

        /// Show top N individuals (default: 10)
        #[arg(short, long, default_value_t = 10)]
        top: usize,
    },

    /// Show detailed information about a specific DNA
    ShowDna {
        /// DNA ID to show
        id: u64,

        /// History file to load
        #[arg(short, long, default_value = "evolution_history.json")]
        file: String,
    },

    /// Show all evolved templates
    ShowTemplates {
        /// History file to load
        #[arg(short, long, default_value = "evolution_history.json")]
        file: String,
    },

    /// Replay a specific DNA with test inputs
    Replay {
        /// DNA ID to replay
        id: u64,

        /// History file to load
        #[arg(short, long, default_value = "evolution_history.json")]
        file: String,

        /// Input values (comma-separated, e.g., "1,2,3")
        #[arg(short, long)]
        input: Option<String>,
    },

    /// Show evolution statistics
    Stats {
        /// History file to load
        #[arg(short, long, default_value = "evolution_history.json")]
        file: String,
    },
}

fn main() {
    let cli = Cli::parse();

    match cli.command {
        Commands::Run {
            problem,
            generations,
            population,
            output,
            verbose,
        } => cmd_evolve(problem, generations, population, output, verbose),

        Commands::Inspect { generation, file, top } => cmd_inspect(generation, file, top),

        Commands::ShowDna { id, file } => cmd_show_dna(id, file),

        Commands::ShowTemplates { file } => cmd_show_templates(file),

        Commands::Replay { id, file, input } => cmd_replay(id, file, input),

        Commands::Stats { file } => cmd_stats(file),
    }
}

fn cmd_evolve(problem: String, generations: u32, population_size: usize, output: String, verbose: bool) {
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
        let template_registry = runner.template_registry;

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

fn cmd_inspect(generation: u32, file: String, top: usize) {
    let history = match EvolutionHistory::load_from_file(&file) {
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

fn cmd_show_dna(id: u64, file: String) {
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

fn cmd_show_templates(_file: String) {
    // Templates are not currently saved in history, only referenced by DNA
    // This would need enhancement to save template definitions
    println!("Note: Template definitions are not currently persisted in history.");
    println!("Templates are recreated during evolution runs.");
    println!("Future enhancement: Save template registry to history.");
}

fn cmd_replay(id: u64, file: String, input: Option<String>) {
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

fn cmd_stats(file: String) {
    let history = match EvolutionHistory::load_from_file(&file) {
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
