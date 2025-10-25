//! Fitness Functions
//!
//! Provides different strategies for evaluating how well a DNA's output matches expectations.
//!
//! This module defines the FitnessFunction trait and several implementations:
//! - ExactMatchFitness: Binary success/failure (1.0 or 0.0)
//! - MSEFitness: Gradual fitness based on mean squared error
//! - PartialMatchFitness: Proportional reward for each correct output element

use super::result::ExecutionResult;

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

        if result.output == expected { 1.0 } else { 0.0 }
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
