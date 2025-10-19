mod cli;
mod commands;
/// Evolve - Evolutionary Algorithm System
///
/// This system allows algorithms to evolve from user-defined primitives into
/// complex behaviors through mutation and crossover, with successful patterns
/// being saved as indexed templates for efficient reuse.
mod dna;
mod evolution;
mod execution;
pub mod lineage; // Make public for cross-module access
mod primitives;
mod storage;
mod template;

use clap::{Parser, Subcommand};
use commands::*;

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
        #[arg(short, long, default_value = "evolution_history.jsonl")]
        output: String,

        /// Verbose output
        #[arg(short, long)]
        verbose: bool,

        /// Save to disk every N generations (0 = save only at end)
        #[arg(long, default_value_t = 0)]
        save_interval: u32,

        /// Keep only last N generations in memory (0 = keep all)
        #[arg(long, default_value_t = 0)]
        keep_in_memory: usize,
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
            save_interval,
            keep_in_memory,
        } => cmd_evolve(
            problem,
            generations,
            population,
            output,
            verbose,
            save_interval,
            keep_in_memory,
        ),

        Commands::Inspect {
            generation,
            file,
            top,
        } => cmd_inspect(generation, file, top),

        Commands::ShowDna { id, file } => cmd_show_dna(id, file),

        Commands::ShowTemplates { file } => cmd_show_templates(file),

        Commands::Replay { id, file, input } => cmd_replay(id, file, input),

        Commands::Stats { file } => cmd_stats(file),
    }
}
