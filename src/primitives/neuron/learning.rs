//! Learning Rule Primitives
//!
//! Local plasticity rules for updating channel counts based on neural activity.
//! All learning rules use fast operations (additions, bit shifts, masks).

use super::interface_bits::*;
use crate::dna::Argument;
use crate::primitives::{ExecutionContext, ExecutionError, Primitive};

/// ApplyHebbian: Strengthen connections when pre and post fire together
/// Args: [interface_reg, pre_fired, post_fired, rate, output_reg]
pub struct ApplyHebbian;

impl Primitive for ApplyHebbian {
    fn execute(
        &self,
        args: &[Argument],
        context: &mut ExecutionContext,
    ) -> Result<(), ExecutionError> {
        if args.len() != 5 {
            return Err(ExecutionError::InvalidArgumentCount {
                expected: 5,
                got: args.len(),
            });
        }

        let mut state = context.read_arg(&args[0])?;
        let pre_fired = context.read_arg(&args[1])? != 0;
        let post_fired = context.read_arg(&args[2])? != 0;
        let rate = context.read_arg(&args[3])? as i32;

        // Hebbian: if both fired, strengthen excitatory connection
        if pre_fired && post_fired {
            let mut exc_channels = (state & EXC_CHANNELS_MASK) as i32;
            exc_channels = exc_channels.saturating_add(rate);
            exc_channels = exc_channels.max(0).min(0xFFFF); // Clamp to 16 bits

            state &= !EXC_CHANNELS_MASK;
            state |= (exc_channels as i64) & EXC_CHANNELS_MASK;
        }

        if let Argument::Register(reg) = args[4] {
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
        5
    }

    fn name(&self) -> &str {
        "ApplyHebbian"
    }

    fn description(&self) -> &str {
        "Hebbian learning: strengthen excitatory channels when both neurons fire"
    }
}

/// ApplyAntiHebbian: Weaken connections when pre and post fire together (decorrelation)
/// Args: [interface_reg, pre_fired, post_fired, rate, output_reg]
pub struct ApplyAntiHebbian;

impl Primitive for ApplyAntiHebbian {
    fn execute(
        &self,
        args: &[Argument],
        context: &mut ExecutionContext,
    ) -> Result<(), ExecutionError> {
        if args.len() != 5 {
            return Err(ExecutionError::InvalidArgumentCount {
                expected: 5,
                got: args.len(),
            });
        }

        let mut state = context.read_arg(&args[0])?;
        let pre_fired = context.read_arg(&args[1])? != 0;
        let post_fired = context.read_arg(&args[2])? != 0;
        let rate = context.read_arg(&args[3])? as i32;

        // Anti-Hebbian: if both fired, weaken excitatory connection
        if pre_fired && post_fired {
            let mut exc_channels = (state & EXC_CHANNELS_MASK) as i32;
            exc_channels = exc_channels.saturating_sub(rate);
            exc_channels = exc_channels.max(0).min(0xFFFF); // Clamp to 16 bits

            state &= !EXC_CHANNELS_MASK;
            state |= (exc_channels as i64) & EXC_CHANNELS_MASK;
        }

        if let Argument::Register(reg) = args[4] {
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
        5
    }

    fn name(&self) -> &str {
        "ApplyAntiHebbian"
    }

    fn description(&self) -> &str {
        "Anti-Hebbian learning: weaken excitatory channels when both neurons fire"
    }
}

/// ApplySTDP: Spike-timing dependent plasticity
/// Args: [interface_reg, time_diff (post - pre), rate, output_reg]
/// Positive time_diff (post after pre) -> strengthen
/// Negative time_diff (pre after post) -> weaken
pub struct ApplySTDP;

impl Primitive for ApplySTDP {
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

        let mut state = context.read_arg(&args[0])?;
        let time_diff = context.read_arg(&args[1])? as i32;
        let rate = context.read_arg(&args[2])? as i32;

        let mut exc_channels = (state & EXC_CHANNELS_MASK) as i32;

        // STDP window: exponential decay based on time difference
        // Use bit shifts for fast exponential approximation
        // delta = rate * (1 >> |time_diff|) with sign of time_diff
        if time_diff > 0 {
            // Post after pre: strengthen (LTP)
            let window = 1.max(10 - time_diff.abs()); // Simple linear window 0-10
            let delta = (rate * window) >> 3; // Fast approximate division
            exc_channels = exc_channels.saturating_add(delta);
        } else if time_diff < 0 {
            // Pre after post: weaken (LTD)
            let window = 1.max(10 - time_diff.abs());
            let delta = (rate * window) >> 3;
            exc_channels = exc_channels.saturating_sub(delta);
        }

        exc_channels = exc_channels.max(0).min(0xFFFF); // Clamp to 16 bits

        state &= !EXC_CHANNELS_MASK;
        state |= (exc_channels as i64) & EXC_CHANNELS_MASK;

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
        "ApplySTDP"
    }

    fn description(&self) -> &str {
        "Spike-timing dependent plasticity based on timing difference"
    }
}

/// ApplyHomeostatic: Maintain target firing rate by scaling channels
/// Args: [interface_reg, target_rate, actual_rate, rate, output_reg]
pub struct ApplyHomeostatic;

impl Primitive for ApplyHomeostatic {
    fn execute(
        &self,
        args: &[Argument],
        context: &mut ExecutionContext,
    ) -> Result<(), ExecutionError> {
        if args.len() != 5 {
            return Err(ExecutionError::InvalidArgumentCount {
                expected: 5,
                got: args.len(),
            });
        }

        let mut state = context.read_arg(&args[0])?;
        let target_rate = context.read_arg(&args[1])? as i32;
        let actual_rate = context.read_arg(&args[2])? as i32;
        let rate = context.read_arg(&args[3])? as i32;

        let mut exc_channels = (state & EXC_CHANNELS_MASK) as i32;
        let mut inh_channels = ((state & INH_CHANNELS_MASK) >> INH_CHANNELS_SHIFT) as i32;

        // Calculate error: (target - actual)
        let error = target_rate - actual_rate;

        // Update channels proportionally to error
        // delta = (error * rate) >> 8 (fast approximate division by 256)
        let delta = (error * rate) >> 8;

        if error > 0 {
            // Firing too little: increase excitation or decrease inhibition
            exc_channels = exc_channels.saturating_add(delta);
        } else {
            // Firing too much: decrease excitation or increase inhibition
            exc_channels = exc_channels.saturating_sub(delta.abs());
        }

        exc_channels = exc_channels.max(0).min(0xFFFF); // Clamp to 16 bits
        inh_channels = inh_channels.max(0).min(0xFFFF);

        state &= !(EXC_CHANNELS_MASK | INH_CHANNELS_MASK);
        state |= (exc_channels as i64) & EXC_CHANNELS_MASK;
        state |= ((inh_channels as i64) << INH_CHANNELS_SHIFT) & INH_CHANNELS_MASK;

        if let Argument::Register(reg) = args[4] {
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
        5
    }

    fn name(&self) -> &str {
        "ApplyHomeostatic"
    }

    fn description(&self) -> &str {
        "Homeostatic plasticity: scale channels to maintain target firing rate"
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::primitives::neuron::interface::CreateInterface;

    #[test]
    fn test_hebbian_learning() {
        let mut ctx = ExecutionContext::new(4, vec![], 1000, 100);

        // Create interface with exc=100, inh=50
        let create = CreateInterface;
        create
            .execute(
                &[
                    Argument::Literal(100),
                    Argument::Literal(50),
                    Argument::Register(0),
                ],
                &mut ctx,
            )
            .unwrap();

        let hebbian = ApplyHebbian;
        hebbian
            .execute(
                &[
                    Argument::Register(0),
                    Argument::Literal(1), // pre_fired
                    Argument::Literal(1), // post_fired
                    Argument::Literal(10), // rate
                    Argument::Register(1),
                ],
                &mut ctx,
            )
            .unwrap();

        let state = ctx.registers[1];
        let exc = (state & EXC_CHANNELS_MASK) as i32;
        assert_eq!(exc, 110); // 100 + 10
    }

    #[test]
    fn test_stdp_ltp() {
        let mut ctx = ExecutionContext::new(4, vec![], 1000, 100);

        // Create interface
        let create = CreateInterface;
        create
            .execute(
                &[
                    Argument::Literal(100),
                    Argument::Literal(50),
                    Argument::Register(0),
                ],
                &mut ctx,
            )
            .unwrap();

        let stdp = ApplySTDP;
        stdp.execute(
            &[
                Argument::Register(0),
                Argument::Literal(5),  // post fired 5 steps after pre (positive)
                Argument::Literal(20), // rate
                Argument::Register(1),
            ],
            &mut ctx,
        )
        .unwrap();

        let state = ctx.registers[1];
        let exc = (state & EXC_CHANNELS_MASK) as i32;
        assert!(exc > 100); // Should have strengthened
    }
}
