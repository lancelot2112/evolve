//! Compartment Primitives
//!
//! Operations for creating and managing compartment state.

use super::compartment_bits::*;
use crate::dna::Argument;
use crate::primitives::{ExecutionContext, ExecutionError, Primitive};

/// CreateCompartment: Creates a new compartment
/// Args: [threshold (literal/reg), decay_shift (literal/reg), learning_rule (literal/reg), output_reg]
pub struct CreateCompartment;

impl Primitive for CreateCompartment {
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

        let threshold = context.read_arg(&args[0])? & 0xFFF; // 12 bits max
        let decay_shift = context.read_arg(&args[1])? & 0xF; // 4 bits max
        let learning_rule = context.read_arg(&args[2])? & 0xF; // 4 bits max

        // Pack compartment state (excitatory=0, inhibitory=0, no flags)
        let state = (threshold << THRESHOLD_SHIFT)
            | (decay_shift << DECAY_SHIFT)
            | (learning_rule << LEARNING_SHIFT);

        if let Argument::Register(reg) = args[3] {
            context.write_register(reg, state)?;
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
        "CreateCompartment"
    }

    fn description(&self) -> &str {
        "Create a new compartment with threshold, decay rate, and learning rule"
    }
}

/// UpdateCompartment: Applies decay and checks threshold
/// Args: [compartment_reg, output_reg]
pub struct UpdateCompartment;

impl Primitive for UpdateCompartment {
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

        let mut state = context.read_arg(&args[0])?;

        // Extract fields
        let mut excitatory = ((state & EXC_MASK) as i32) << 12 >> 12; // Sign extend 20->32 bits
        let mut inhibitory = (((state & INH_MASK) >> INH_SHIFT) as i32) << 12 >> 12; // Sign extend
        let threshold = ((state & THRESHOLD_MASK) >> THRESHOLD_SHIFT) as i32;
        let decay_shift = ((state & DECAY_MASK) >> DECAY_SHIFT) as u8;

        // Apply decay (fast bit shift!)
        excitatory >>= decay_shift;
        inhibitory >>= decay_shift;

        // Calculate net level
        let net_level = excitatory - inhibitory;

        // Clear old flags
        state &= !(FLAGS_MASK);

        // Check threshold
        if net_level >= threshold {
            state |= FLAG_FIRED;
        }

        // Update excitatory and inhibitory fields
        state &= !(EXC_MASK | INH_MASK);
        state |= (excitatory as i64) & EXC_MASK;
        state |= ((inhibitory as i64) << INH_SHIFT) & INH_MASK;

        if let Argument::Register(reg) = args[1] {
            context.write_register(reg, state)?;
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
        "UpdateCompartment"
    }

    fn description(&self) -> &str {
        "Apply decay to compartment levels and check firing threshold"
    }
}

/// GetCompartmentLevel: Extract net level (excitatory - inhibitory)
/// Args: [compartment_reg, output_reg]
pub struct GetCompartmentLevel;

impl Primitive for GetCompartmentLevel {
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

        let state = context.read_arg(&args[0])?;

        // Extract and sign-extend excitatory and inhibitory
        let excitatory = ((state & EXC_MASK) as i32) << 12 >> 12;
        let inhibitory = (((state & INH_MASK) >> INH_SHIFT) as i32) << 12 >> 12;

        let net_level = (excitatory - inhibitory) as i64;

        if let Argument::Register(reg) = args[1] {
            context.write_register(reg, net_level)?;
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
        "GetCompartmentLevel"
    }

    fn description(&self) -> &str {
        "Get net level (excitatory - inhibitory) from compartment"
    }
}

/// CheckFired: Check if compartment fired
/// Args: [compartment_reg, output_reg]
pub struct CheckFired;

impl Primitive for CheckFired {
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

        let state = context.read_arg(&args[0])?;
        let fired = if (state & FLAG_FIRED) != 0 { 1 } else { 0 };

        if let Argument::Register(reg) = args[1] {
            context.write_register(reg, fired)?;
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
        "CheckFired"
    }

    fn description(&self) -> &str {
        "Check if compartment fired (1 if fired, 0 otherwise)"
    }
}

/// ResetCompartment: Clear fired flag and optionally reset levels
/// Args: [compartment_reg, reset_levels (0 or 1), output_reg]
pub struct ResetCompartment;

impl Primitive for ResetCompartment {
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

        let mut state = context.read_arg(&args[0])?;
        let reset_levels = context.read_arg(&args[1])? != 0;

        // Clear fired and refractory flags
        state &= !(FLAG_FIRED | FLAG_REFRACTORY);

        if reset_levels {
            // Clear excitatory and inhibitory levels
            state &= !(EXC_MASK | INH_MASK);
        }

        if let Argument::Register(reg) = args[2] {
            context.write_register(reg, state)?;
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
        "ResetCompartment"
    }

    fn description(&self) -> &str {
        "Reset compartment fired flag and optionally clear levels"
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_create_compartment() {
        let mut ctx = ExecutionContext::new(4, vec![], 1000, 100);
        let create = CreateCompartment;

        let args = vec![
            Argument::Literal(1000), // threshold
            Argument::Literal(2),    // decay_shift
            Argument::Literal(0),    // learning_rule (Hebbian)
            Argument::Register(0),   // output
        ];

        create.execute(&args, &mut ctx).unwrap();

        // Verify compartment was created
        assert_ne!(ctx.registers[0], 0);

        // Extract threshold
        let state = ctx.registers[0];
        let threshold = ((state & THRESHOLD_MASK) >> THRESHOLD_SHIFT) as i32;
        assert_eq!(threshold, 1000);
    }

    #[test]
    fn test_update_compartment() {
        let mut ctx = ExecutionContext::new(4, vec![], 1000, 100);

        // Create compartment with threshold=100, decay_shift=1
        let create = CreateCompartment;
        create
            .execute(
                &[
                    Argument::Literal(100),
                    Argument::Literal(1),
                    Argument::Literal(0),
                    Argument::Register(0),
                ],
                &mut ctx,
            )
            .unwrap();

        // Manually set excitatory level to 200
        ctx.registers[0] |= 200; // Set excitatory bits

        let update = UpdateCompartment;
        update
            .execute(&[Argument::Register(0), Argument::Register(1)], &mut ctx)
            .unwrap();

        // Check if fired
        let check = CheckFired;
        check
            .execute(&[Argument::Register(1), Argument::Register(2)], &mut ctx)
            .unwrap();

        assert_eq!(ctx.registers[2], 1); // Should have fired (200 >> 1 = 100 >= threshold)
    }
}
