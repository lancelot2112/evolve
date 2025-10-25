//! Batch Processing Primitives
//!
//! Monte Carlo cache-aware batch processing for large-scale neuron simulations.
//!
//! ## Cache-Aligned Processing
//! Instead of processing neurons sequentially, these primitives enable:
//! 1. Random selection of cache-aligned memory regions
//! 2. Full processing of all neurons in that cache line
//! 3. Jump to next random cache line
//!
//! This maximizes cache utilization and avoids systematic biases from
//! sequential processing, mimicking more realistic biological dynamics.
//!
//! ## Stack-Based Neuron Arrays
//! Neurons are stored on the execution stack as packed i64 values.
//! Cache line size is typically 64 bytes = 8 x i64 values.

use super::compartment_bits as comp;
use crate::dna::Argument;
use crate::primitives::{ExecutionContext, ExecutionError, Primitive};

/// CACHE_LINE_SIZE: Number of i64 values per cache line (64 bytes / 8 bytes)
pub const CACHE_LINE_SIZE: usize = 8;

/// GetRandomCacheOffset: Generate random cache-aligned offset
/// Args: [rng_seed_reg, array_size, output_offset_reg, output_seed_reg]
/// Uses LCG: next = (a * seed + c) mod m
pub struct GetRandomCacheOffset;

impl Primitive for GetRandomCacheOffset {
    fn execute(
        &self,
        args: &[Argument],
        context: &mut ExecutionContext,
    ) -> Result<(), ExecutionError> {
        if args.len() != 4 {
            return Err(ExecutionError::InvalidArgumentCount {
                expected: 4,
                got: args.len(),
            });
        }

        let seed = context.read_arg(&args[0])?;
        let array_size = context.read_arg(&args[1])? as usize;

        // Linear Congruential Generator (fast PRNG)
        // Constants from Numerical Recipes
        const A: i64 = 1664525;
        const C: i64 = 1013904223;
        let next_seed = seed.wrapping_mul(A).wrapping_add(C);

        // Calculate number of cache lines
        let num_cache_lines = (array_size + CACHE_LINE_SIZE - 1) / CACHE_LINE_SIZE;

        // Get random cache line index using bit mask (fast modulo for power of 2)
        let cache_line_idx = if num_cache_lines > 0 {
            (next_seed.abs() as usize) % num_cache_lines
        } else {
            0
        };

        // Convert to cache-aligned offset
        let offset = cache_line_idx * CACHE_LINE_SIZE;

        if let Argument::Register(reg) = args[2] {
            context.write_register(reg, offset as i64)?;
        } else {
            return Err(ExecutionError::InvalidArgumentType {
                expected: "Register",
                got: "Literal",
            });
        }

        if let Argument::Register(reg) = args[3] {
            context.write_register(reg, next_seed)?;
        } else {
            return Err(ExecutionError::InvalidArgumentType {
                expected: "Register",
                got: "Literal",
            });
        }

        Ok(())
    }

    fn arg_count(&self) -> usize {
        4
    }

    fn name(&self) -> &str {
        "GetRandomCacheOffset"
    }

    fn description(&self) -> &str {
        "Generate random cache-aligned offset for Monte Carlo neuron processing"
    }
}

/// ProcessCompartmentBatch: Update a batch of compartments from stack
/// Args: [start_offset, batch_size]
/// Processes compartments on stack, applying decay and threshold checking
pub struct ProcessCompartmentBatch;

impl Primitive for ProcessCompartmentBatch {
    fn execute(
        &self,
        args: &[Argument],
        context: &mut ExecutionContext,
    ) -> Result<(), ExecutionError> {
        if args.len() != 2 {
            return Err(ExecutionError::InvalidArgumentCount {
                expected: 2,
                got: args.len(),
            });
        }

        let start_offset = context.read_arg(&args[0])? as usize;
        let batch_size = context.read_arg(&args[1])? as usize;

        let stack_len = context.stack.len();
        if start_offset >= stack_len {
            return Ok(()); // Nothing to process
        }

        let end_offset = (start_offset + batch_size).min(stack_len);

        // Process each compartment in the batch
        for i in start_offset..end_offset {
            let mut state = context.stack[i];

            // Extract fields
            let mut excitatory = ((state & comp::EXC_MASK) as i32) << 12 >> 12; // Sign extend
            let mut inhibitory =
                (((state & comp::INH_MASK) >> comp::INH_SHIFT) as i32) << 12 >> 12;
            let threshold = ((state & comp::THRESHOLD_MASK) >> comp::THRESHOLD_SHIFT) as i32;
            let decay_shift = ((state & comp::DECAY_MASK) >> comp::DECAY_SHIFT) as u8;

            // Apply decay (fast bit shift!)
            excitatory >>= decay_shift;
            inhibitory >>= decay_shift;

            // Calculate net level
            let net_level = excitatory - inhibitory;

            // Clear old flags
            state &= !(comp::FLAGS_MASK);

            // Check threshold
            if net_level >= threshold {
                state |= comp::FLAG_FIRED;
            }

            // Update excitatory and inhibitory fields
            state &= !(comp::EXC_MASK | comp::INH_MASK);
            state |= (excitatory as i64) & comp::EXC_MASK;
            state |= ((inhibitory as i64) << comp::INH_SHIFT) & comp::INH_MASK;

            // Write back to stack
            context.stack[i] = state;
        }

        Ok(())
    }

    fn arg_count(&self) -> usize {
        2
    }

    fn name(&self) -> &str {
        "ProcessCompartmentBatch"
    }

    fn description(&self) -> &str {
        "Update batch of compartments from stack (decay and threshold check)"
    }
}

/// CountFiredNeurons: Count how many neurons fired in a batch
/// Args: [start_offset, batch_size, output_count_reg]
pub struct CountFiredNeurons;

impl Primitive for CountFiredNeurons {
    fn execute(
        &self,
        args: &[Argument],
        context: &mut ExecutionContext,
    ) -> Result<(), ExecutionError> {
        if args.len() != 3 {
            return Err(ExecutionError::InvalidArgumentCount {
                expected: 3,
                got: args.len(),
            });
        }

        let start_offset = context.read_arg(&args[0])? as usize;
        let batch_size = context.read_arg(&args[1])? as usize;

        let stack_len = context.stack.len();
        if start_offset >= stack_len {
            if let Argument::Register(reg) = args[2] {
                context.write_register(reg, 0)?;
            }
            return Ok(());
        }

        let end_offset = (start_offset + batch_size).min(stack_len);
        let mut count = 0i64;

        // Count fired neurons
        for i in start_offset..end_offset {
            let state = context.stack[i];
            if (state & comp::FLAG_FIRED) != 0 {
                count += 1;
            }
        }

        if let Argument::Register(reg) = args[2] {
            context.write_register(reg, count)?;
        } else {
            return Err(ExecutionError::InvalidArgumentType {
                expected: "Register",
                got: "Literal",
            });
        }

        Ok(())
    }

    fn arg_count(&self) -> usize {
        3
    }

    fn name(&self) -> &str {
        "CountFiredNeurons"
    }

    fn description(&self) -> &str {
        "Count how many neurons fired in a stack batch"
    }
}

/// LoadCompartmentFromStack: Load compartment from stack to register
/// Args: [stack_index, output_reg]
pub struct LoadCompartmentFromStack;

impl Primitive for LoadCompartmentFromStack {
    fn execute(
        &self,
        args: &[Argument],
        context: &mut ExecutionContext,
    ) -> Result<(), ExecutionError> {
        if args.len() != 2 {
            return Err(ExecutionError::InvalidArgumentCount {
                expected: 2,
                got: args.len(),
            });
        }

        let index = context.read_arg(&args[0])? as usize;

        if index >= context.stack.len() {
            return Err(ExecutionError::StackUnderflow);
        }

        let value = context.stack[index];

        if let Argument::Register(reg) = args[1] {
            context.write_register(reg, value)?;
        } else {
            return Err(ExecutionError::InvalidArgumentType {
                expected: "Register",
                got: "Literal",
            });
        }

        Ok(())
    }

    fn arg_count(&self) -> usize {
        2
    }

    fn name(&self) -> &str {
        "LoadCompartmentFromStack"
    }

    fn description(&self) -> &str {
        "Load compartment state from stack by index"
    }
}

/// StoreCompartmentToStack: Store compartment from register to stack
/// Args: [compartment_reg, stack_index]
pub struct StoreCompartmentToStack;

impl Primitive for StoreCompartmentToStack {
    fn execute(
        &self,
        args: &[Argument],
        context: &mut ExecutionContext,
    ) -> Result<(), ExecutionError> {
        if args.len() != 2 {
            return Err(ExecutionError::InvalidArgumentCount {
                expected: 2,
                got: args.len(),
            });
        }

        let value = context.read_arg(&args[0])?;
        let index = context.read_arg(&args[1])? as usize;

        // Extend stack if needed
        while context.stack.len() <= index {
            context.stack.push(0);
        }

        context.stack[index] = value;

        Ok(())
    }

    fn arg_count(&self) -> usize {
        2
    }

    fn name(&self) -> &str {
        "StoreCompartmentToStack"
    }

    fn description(&self) -> &str {
        "Store compartment state from register to stack by index"
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::primitives::neuron::compartment::CreateCompartment;

    #[test]
    fn test_random_cache_offset() {
        let mut ctx = ExecutionContext::new(8, vec![], 1000, 100);
        ctx.registers[0] = 12345; // seed

        let random_offset = GetRandomCacheOffset;
        random_offset
            .execute(
                &[
                    Argument::Register(0),  // seed
                    Argument::Literal(100), // array size
                    Argument::Register(1),  // output offset
                    Argument::Register(2),  // output seed
                ],
                &mut ctx,
            )
            .unwrap();

        let offset = ctx.registers[1] as usize;
        let new_seed = ctx.registers[2];

        // Offset should be cache-aligned
        assert_eq!(offset % CACHE_LINE_SIZE, 0);
        // Seed should have changed
        assert_ne!(new_seed, 12345);
    }

    #[test]
    fn test_batch_processing() {
        let mut ctx = ExecutionContext::new(8, vec![], 1000, 100);

        // Create some compartments and push to stack
        let create = CreateCompartment;
        for i in 0..CACHE_LINE_SIZE {
            create
                .execute(
                    &[
                        Argument::Literal(100),    // threshold
                        Argument::Literal(1),      // decay_shift
                        Argument::Literal(0),      // learning_rule
                        Argument::Register(0),
                    ],
                    &mut ctx,
                )
                .unwrap();

            // Add excitatory level: 200 + i*10
            // After decay (>>1): 100 + i*5
            // First neuron will be at threshold (100), later ones above
            ctx.registers[0] |= (200 + i as i64 * 10) as i64;

            // Push to stack
            ctx.stack.push(ctx.registers[0]);
        }

        // Process batch
        let batch = ProcessCompartmentBatch;
        batch
            .execute(
                &[
                    Argument::Literal(0),                    // start offset
                    Argument::Literal(CACHE_LINE_SIZE as i64), // batch size
                ],
                &mut ctx,
            )
            .unwrap();

        // Count fired neurons
        let count = CountFiredNeurons;
        count
            .execute(
                &[
                    Argument::Literal(0),
                    Argument::Literal(CACHE_LINE_SIZE as i64),
                    Argument::Register(1),
                ],
                &mut ctx,
            )
            .unwrap();

        // All neurons should have fired (level after decay >= threshold)
        // First: 200>>1=100 >= 100 (fires)
        // Last (i=7): 270>>1=135 >= 100 (fires)
        assert_eq!(ctx.registers[1], CACHE_LINE_SIZE as i64);
    }
}
