/// Execution module: Sandboxed execution environment for DNA
///
/// This module provides safe execution of DNA strands, including template expansion
/// and safety limits to prevent runaway execution.

use crate::dna::{DNA, Gene, OperationId};
use crate::primitives::{ExecutionContext, ExecutionError, PrimitiveRegistry};
use crate::template::TemplateRegistry;

/// Result of executing DNA
#[derive(Debug, Clone)]
pub struct ExecutionResult {
    /// Output produced
    pub output: Vec<i64>,
    /// Number of instructions executed (including template expansions)
    pub instruction_count: usize,
    /// Maximum stack depth reached
    pub max_stack_depth: usize,
    /// Whether execution completed successfully
    pub success: bool,
    /// Error message if execution failed
    pub error: Option<String>,
}

/// Configuration for execution environment
#[derive(Debug, Clone)]
pub struct ExecutionConfig {
    /// Number of registers available
    pub register_count: usize,
    /// Maximum instruction count (prevents infinite loops)
    pub max_instructions: usize,
    /// Maximum stack depth
    pub max_stack_depth: usize,
    /// Maximum template recursion depth
    pub max_recursion_depth: usize,
}

impl Default for ExecutionConfig {
    fn default() -> Self {
        Self {
            register_count: 8,          // R0-R7
            max_instructions: 10_000,   // Reasonable limit
            max_stack_depth: 1_000,     // Deep enough for most uses
            max_recursion_depth: 100,   // Prevent infinite template recursion
        }
    }
}

/// Execute DNA in a sandboxed environment
pub struct Executor {
    config: ExecutionConfig,
}

impl Executor {
    pub fn new(config: ExecutionConfig) -> Self {
        Self { config }
    }

    pub fn with_defaults() -> Self {
        Self::new(ExecutionConfig::default())
    }

    /// Execute a single gene
    fn execute_gene(
        &self,
        gene: &Gene,
        context: &mut ExecutionContext,
        primitive_registry: &PrimitiveRegistry,
        template_registry: &TemplateRegistry,
        recursion_depth: usize,
    ) -> Result<(), ExecutionError> {
        // Check recursion depth
        if recursion_depth > self.config.max_recursion_depth {
            return Err(ExecutionError::Other(
                "Maximum template recursion depth exceeded".to_string(),
            ));
        }

        // Tick instruction counter
        context.tick()?;

        match gene.operation {
            OperationId::Primitive(prim_id) => {
                // Execute primitive
                if let Some(primitive) = primitive_registry.get(prim_id) {
                    primitive.execute(&gene.args, context)?;
                } else {
                    return Err(ExecutionError::Other(format!(
                        "Unknown primitive ID: {}",
                        prim_id
                    )));
                }
            }
            OperationId::Template(template_id) => {
                // Expand and execute template
                if let Some(template) = template_registry.get(template_id) {
                    for template_gene in &template.genes {
                        self.execute_gene(
                            template_gene,
                            context,
                            primitive_registry,
                            template_registry,
                            recursion_depth + 1,
                        )?;
                    }
                } else {
                    return Err(ExecutionError::Other(format!(
                        "Unknown template ID: {}",
                        template_id
                    )));
                }
            }
        }

        Ok(())
    }

    /// Execute DNA and return result
    pub fn execute(
        &self,
        dna: &DNA,
        input: Vec<i64>,
        primitive_registry: &PrimitiveRegistry,
    ) -> ExecutionResult {
        let mut context = ExecutionContext::new(
            self.config.register_count,
            input,
            self.config.max_instructions,
            self.config.max_stack_depth,
        );

        let mut success = true;
        let mut error = None;

        // Use DNA's lineage-local template library
        let template_registry = &dna.template_library;

        // Execute each gene
        for gene in &dna.genes {
            match self.execute_gene(gene, &mut context, primitive_registry, template_registry, 0)
            {
                Ok(_) => {}
                Err(e) => {
                    success = false;
                    error = Some(e.to_string());
                    break;
                }
            }
        }

        let max_stack_depth = context.stack.len();

        ExecutionResult {
            output: context.output,
            instruction_count: context.instruction_count,
            max_stack_depth,
            success,
            error,
        }
    }
}

/// Fitness function trait
pub trait FitnessFunction {
    /// Evaluate fitness based on output
    /// Returns a value typically between 0.0 (worst) and 1.0 (best)
    fn evaluate(&self, result: &ExecutionResult, expected: &[i64]) -> f64;
}

/// Simple exact match fitness function
pub struct ExactMatchFitness;

impl FitnessFunction for ExactMatchFitness {
    fn evaluate(&self, result: &ExecutionResult, expected: &[i64]) -> f64 {
        if !result.success {
            return 0.0;
        }

        if result.output == expected {
            1.0
        } else {
            0.0
        }
    }
}

/// Mean squared error fitness function (higher is better)
pub struct MSEFitness;

impl FitnessFunction for MSEFitness {
    fn evaluate(&self, result: &ExecutionResult, expected: &[i64]) -> f64 {
        if !result.success {
            return 0.0;
        }

        if result.output.is_empty() && !expected.is_empty() {
            return 0.0;
        }

        let len = result.output.len().min(expected.len());
        if len == 0 {
            return if expected.is_empty() { 1.0 } else { 0.0 };
        }

        let mse: f64 = (0..len)
            .map(|i| {
                let diff = result.output[i] - expected[i];
                (diff * diff) as f64
            })
            .sum::<f64>()
            / len as f64;

        // Convert MSE to fitness (lower MSE = higher fitness)
        // Use 1 / (1 + mse) to map to [0, 1]
        1.0 / (1.0 + mse)
    }
}

/// Partial match fitness (rewards each correct output element)
pub struct PartialMatchFitness;

impl FitnessFunction for PartialMatchFitness {
    fn evaluate(&self, result: &ExecutionResult, expected: &[i64]) -> f64 {
        if !result.success {
            return 0.0;
        }

        if expected.is_empty() {
            return if result.output.is_empty() { 1.0 } else { 0.0 };
        }

        let len = result.output.len().min(expected.len());
        if len == 0 {
            return 0.0;
        }

        let matches = (0..len)
            .filter(|&i| result.output[i] == expected[i])
            .count();

        matches as f64 / expected.len() as f64
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::dna::Argument;

    #[test]
    fn test_simple_execution() {
        let executor = Executor::with_defaults();
        let primitive_registry = PrimitiveRegistry::with_standard_primitives();

        // DNA: READ R0, WRITE R0 (echo program)
        let mut dna = DNA::empty(0);
        dna.push_gene(Gene::primitive(6, vec![Argument::Register(0)])); // READ_INPUT
        dna.push_gene(Gene::primitive(7, vec![Argument::Register(0)])); // WRITE_OUTPUT

        let result = executor.execute(&dna, vec![42], &primitive_registry);

        assert!(result.success);
        assert_eq!(result.output, vec![42]);
    }

    #[test]
    fn test_arithmetic_execution() {
        let executor = Executor::with_defaults();
        let primitive_registry = PrimitiveRegistry::with_standard_primitives();

        // DNA: ADD(5, 3, R0), WRITE R0
        let mut dna = DNA::empty(0);
        dna.push_gene(Gene::primitive(
            0,
            vec![
                Argument::Literal(5),
                Argument::Literal(3),
                Argument::Register(0),
            ],
        )); // ADD
        dna.push_gene(Gene::primitive(7, vec![Argument::Register(0)])); // WRITE_OUTPUT

        let result = executor.execute(&dna, vec![], &primitive_registry);

        assert!(result.success);
        assert_eq!(result.output, vec![8]);
    }

    #[test]
    fn test_template_execution() {
        let executor = Executor::with_defaults();
        let primitive_registry = PrimitiveRegistry::with_standard_primitives();

        // Create a template that doubles a value: ADD R0, R0 -> R0
        let template_genes = vec![Gene::primitive(
            0,
            vec![
                Argument::Register(0),
                Argument::Register(0),
                Argument::Register(0),
            ],
        )];

        // DNA: READ R0, Template(hash), WRITE R0
        let mut dna = DNA::empty(0);
        // Add template to DNA's local library
        let template_hash = dna.template_library.register(template_genes, 1.0, 0);

        dna.push_gene(Gene::primitive(6, vec![Argument::Register(0)])); // READ
        dna.push_gene(Gene::template(template_hash, vec![])); // Double
        dna.push_gene(Gene::primitive(7, vec![Argument::Register(0)])); // WRITE

        let result = executor.execute(&dna, vec![21], &primitive_registry);

        assert!(result.success);
        assert_eq!(result.output, vec![42]);
    }

    #[test]
    fn test_exact_match_fitness() {
        let fitness = ExactMatchFitness;

        let result = ExecutionResult {
            output: vec![1, 2, 3],
            instruction_count: 10,
            max_stack_depth: 0,
            success: true,
            error: None,
        };

        assert_eq!(fitness.evaluate(&result, &[1, 2, 3]), 1.0);
        assert_eq!(fitness.evaluate(&result, &[1, 2, 4]), 0.0);
    }

    #[test]
    fn test_partial_match_fitness() {
        let fitness = PartialMatchFitness;

        let result = ExecutionResult {
            output: vec![1, 2, 4],
            instruction_count: 10,
            max_stack_depth: 0,
            success: true,
            error: None,
        };

        // 2 out of 3 match
        assert_eq!(fitness.evaluate(&result, &[1, 2, 3]), 2.0 / 3.0);
    }
}
