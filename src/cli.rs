/// CLI module for command-line interface
///
/// Provides commands for evolving, inspecting, and analyzing DNA

use crate::dna::{Argument, DNA, Gene, OperationId};
use crate::evolution::{Crossover, MutationConfig, PointMutator};
use crate::execution::{ExactMatchFitness, Executor, FitnessFunction};
use crate::primitives::PrimitiveRegistry;
use crate::storage::{EvolutionHistory, GenerationRecord};

use rand::Rng;

/// Simple evolutionary algorithm runner
pub struct EvolutionRunner {
    pub population_size: usize,
    pub mutation_rate: f64,
    pub crossover_rate: f64,
    pub primitive_registry: PrimitiveRegistry,
    executor: Executor,
    mutator: PointMutator,
    crossover: Crossover,
    /// Save to disk every N generations (0 = save only at end)
    pub save_interval: u32,
    /// Keep only last N generations in memory (0 = keep all)
    pub keep_in_memory: usize,
}

impl EvolutionRunner {
    pub fn new() -> Self {
        let primitive_registry = PrimitiveRegistry::with_standard_primitives();
        let executor = Executor::with_defaults();

        let mut mutation_config = MutationConfig::default();
        mutation_config.max_primitive_id = (primitive_registry.count() - 1) as u16;

        let mutator = PointMutator::new(mutation_config);
        let crossover = Crossover::single_point();

        Self {
            population_size: 100,
            mutation_rate: 0.3,
            crossover_rate: 0.6,
            primitive_registry,
            executor,
            mutator,
            crossover,
            save_interval: 0,      // Default: save only at end
            keep_in_memory: 0,     // Default: keep all in memory
        }
    }

    /// Initialize random population
    fn initialize_population(&self, generation: u32) -> Vec<DNA> {
        let mut rng = rand::thread_rng();
        let mut population = Vec::new();

        for _ in 0..self.population_size {
            let mut dna = DNA::empty(generation);

            // Random initial length
            let length = rng.gen_range(1..=10);

            for _ in 0..length {
                let prim_id = rng.gen_range(0..self.primitive_registry.count()) as u16;
                let arg_count = rng.gen_range(0..=3);

                let args: Vec<Argument> = (0..arg_count)
                    .map(|_| {
                        if rng.r#gen::<bool>() {
                            Argument::Register(rng.gen_range(0..8))
                        } else {
                            Argument::Literal(rng.gen_range(-10..=10))
                        }
                    })
                    .collect();

                dna.push_gene(Gene::primitive(prim_id, args));
            }

            population.push(dna);
        }

        population
    }

    /// Evaluate fitness for entire population
    fn evaluate_population(
        &self,
        population: &mut [DNA],
        test_cases: &[(Vec<i64>, Vec<i64>)],
        fitness_fn: &dyn FitnessFunction,
    ) {
        for dna in population.iter_mut() {
            let mut total_fitness = 0.0;

            for (input, expected_output) in test_cases {
                let result = self.executor.execute(
                    dna,
                    input.clone(),
                    &self.primitive_registry,
                );

                total_fitness += fitness_fn.evaluate(&result, expected_output);
            }

            // Average fitness across test cases
            dna.set_fitness(total_fitness / test_cases.len() as f64);

            // Detect and register templates in this DNA's local library
            dna.detect_and_register_templates();
        }
    }

    /// Tournament selection
    fn select_parent<'a>(&self, population: &'a [DNA]) -> &'a DNA {
        let mut rng = rand::thread_rng();
        let tournament_size = 3;

        let contestants: Vec<&DNA> = (0..tournament_size)
            .map(|_| &population[rng.gen_range(0..population.len())])
            .collect();

        contestants
            .into_iter()
            .max_by(|a, b| {
                a.fitness
                    .unwrap_or(0.0)
                    .partial_cmp(&b.fitness.unwrap_or(0.0))
                    .unwrap_or(std::cmp::Ordering::Equal)
            })
            .unwrap()
    }

    /// Create next generation
    fn create_next_generation(&self, population: &[DNA], generation: u32) -> Vec<DNA> {
        let mut rng = rand::thread_rng();
        let mut next_generation = Vec::new();

        // Elitism: Keep best individual
        if let Some(best) = population
            .iter()
            .max_by(|a, b| {
                a.fitness
                    .unwrap_or(0.0)
                    .partial_cmp(&b.fitness.unwrap_or(0.0))
                    .unwrap_or(std::cmp::Ordering::Equal)
            })
        {
            next_generation.push(best.clone());
        }

        // Generate rest of population
        while next_generation.len() < self.population_size {
            let parent1 = self.select_parent(population);

            let mut child = if rng.r#gen::<f64>() < self.crossover_rate {
                // Crossover
                let parent2 = self.select_parent(population);
                self.crossover.cross(parent1, parent2)
            } else {
                // Clone parent
                parent1.clone()
            };

            // Mutation
            if rng.r#gen::<f64>() < self.mutation_rate {
                child = self.mutator.mutate(&child);
            }

            child.generation = generation;
            next_generation.push(child);
        }

        next_generation
    }


    /// Run evolution for N generations
    ///
    /// If save_interval > 0 and output_file is provided, saves incrementally to disk
    /// and optionally clears old generations from memory based on keep_in_memory setting.
    pub fn run(
        &mut self,
        generations: u32,
        test_cases: &[(Vec<i64>, Vec<i64>)],
        fitness_fn: &dyn FitnessFunction,
        verbose: bool,
        output_file: Option<&str>,
    ) -> EvolutionHistory {
        let mut history = EvolutionHistory::new();
        let use_incremental_save = self.save_interval > 0 && output_file.is_some();

        // Initialize population
        let mut population = self.initialize_population(0);

        for generation_num in 0..generations {
            if verbose {
                println!("Generation {}", generation_num);
            }

            // Evaluate fitness (also detects and registers templates in each DNA's local library)
            self.evaluate_population(&mut population, test_cases, fitness_fn);

            // Record statistics
            // Note: Templates are now tracked locally in each DNA's template_library
            let record = GenerationRecord::new(generation_num, population.clone(), Vec::new());

            if verbose {
                println!(
                    "  Best fitness: {:.4}, Avg fitness: {:.4}",
                    record.best_fitness,
                    record.average_fitness,
                );
            }

            // Incremental save logic
            if use_incremental_save && (generation_num % self.save_interval == 0 || generation_num == generations - 1) {
                if let Some(path) = output_file {
                    if let Err(e) = history.append_generation_to_file(record.clone(), path) {
                        eprintln!("Warning: Failed to save generation {}: {}", generation_num, e);
                        // Continue execution even if save fails
                        history.add_generation(record);
                    } else {
                        if verbose {
                            println!("  Saved to disk (generation {})", generation_num);
                        }

                        // Clear old records from memory if configured
                        if self.keep_in_memory > 0 {
                            history.clear_old_records(self.keep_in_memory);
                        }
                    }
                } else {
                    history.add_generation(record);
                }
            } else {
                history.add_generation(record);
            }

            // Create next generation
            if generation_num < generations - 1 {
                population = self.create_next_generation(&population, generation_num + 1);
            }
        }

        history
    }
}

impl Default for EvolutionRunner {
    fn default() -> Self {
        Self::new()
    }
}

/// Format DNA for display (uses DNA's local template_library)
pub fn format_dna(dna: &DNA, primitives: &PrimitiveRegistry) -> String {
    let mut output = String::new();

    if let Some(id) = dna.id {
        output.push_str(&format!("DNA ID: {}\n", id));
    }
    output.push_str(&format!("Generation: {}\n", dna.generation));
    output.push_str(&format!("Fitness: {:.4}\n", dna.fitness.unwrap_or(0.0)));
    output.push_str(&format!("Length: {} genes\n", dna.len()));
    output.push_str(&format!("Templates in library: {}\n\n", dna.template_library.count()));

    output.push_str("Genes:\n");
    for (i, gene) in dna.genes.iter().enumerate() {
        let op_name = match gene.operation {
            OperationId::Primitive(id) => {
                primitives.get(id)
                    .map(|p| p.name().to_string())
                    .unwrap_or_else(|| format!("PRIM_{}", id))
            }
            OperationId::Template(hash) => {
                format!("TEMPLATE_{:x}", hash)
            }
        };

        let args_str: Vec<String> = gene.args.iter().map(|arg| {
            match arg {
                Argument::Register(r) => format!("R{}", r),
                Argument::Literal(l) => l.to_string(),
            }
        }).collect();

        output.push_str(&format!("  {}: {} {}\n", i, op_name, args_str.join(" ")));
    }

    output
}
