//! Packing Utility Primitives
//!
//! Utilities for extracting and setting individual fields in packed state.
//! Useful for debugging and manual state manipulation.

use super::interface_bits::*;
use crate::dna::Argument;
use crate::primitives::{ExecutionContext, ExecutionError, Primitive};

/// GetChannelCounts: Extract excitatory and inhibitory channel counts
/// Args: [interface_reg, exc_output_reg, inh_output_reg]
pub struct GetChannelCounts;

impl Primitive for GetChannelCounts {
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

        let state = context.read_arg(&args[0])?;

        let exc_channels = state & EXC_CHANNELS_MASK;
        let inh_channels = (state & INH_CHANNELS_MASK) >> INH_CHANNELS_SHIFT;

        if let Argument::Register(reg) = args[1] {
            context.write_register(reg, exc_channels)?;
        } else {
            return Err(ExecutionError::InvalidArgumentType {
                expected: "Register",
                got: "Literal",
            });
        }

        if let Argument::Register(reg) = args[2] {
            context.write_register(reg, inh_channels)?;
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
        "GetChannelCounts"
    }

    fn description(&self) -> &str {
        "Extract excitatory and inhibitory channel counts from interface"
    }
}

/// SetChannelCounts: Set excitatory and inhibitory channel counts
/// Args: [interface_reg, exc_count, inh_count, output_reg]
pub struct SetChannelCounts;

impl Primitive for SetChannelCounts {
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
        let exc_count = context.read_arg(&args[1])? & 0xFFFF; // 16 bits max
        let inh_count = context.read_arg(&args[2])? & 0xFFFF; // 16 bits max

        // Clear old channel counts
        state &= !(EXC_CHANNELS_MASK | INH_CHANNELS_MASK);

        // Set new channel counts
        state |= exc_count & EXC_CHANNELS_MASK;
        state |= (inh_count << INH_CHANNELS_SHIFT) & INH_CHANNELS_MASK;

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
        "SetChannelCounts"
    }

    fn description(&self) -> &str {
        "Set excitatory and inhibitory channel counts in interface"
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_get_set_channel_counts() {
        let mut ctx = ExecutionContext::new(8, vec![], 1000, 100);

        // Set channel counts
        let set = SetChannelCounts;
        set.execute(
            &[
                Argument::Literal(0), // empty interface
                Argument::Literal(1234),
                Argument::Literal(5678),
                Argument::Register(0),
            ],
            &mut ctx,
        )
        .unwrap();

        // Get channel counts back
        let get = GetChannelCounts;
        get.execute(
            &[
                Argument::Register(0),
                Argument::Register(1),
                Argument::Register(2),
            ],
            &mut ctx,
        )
        .unwrap();

        assert_eq!(ctx.registers[1], 1234);
        assert_eq!(ctx.registers[2], 5678);
    }
}
