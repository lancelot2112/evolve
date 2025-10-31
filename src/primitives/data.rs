//! Data Movement Primitives
//!
//! Operations for moving and manipulating data (COPY, NOP).
//!
//! COPY moves values between registers or from literals to registers.
//! NOP performs no operation and is useful for mutation padding.

use crate::dna::Argument;
use super::context::{ExecutionContext, ExecutionError};
use super::trait_def::Primitive;

/// COPY: Copy args[0] to register args[1]
pub struct Copy;

impl Primitive for Copy {
    fn execute(&self, args: &[Argument], context: &mut ExecutionContext) -> Result<(), ExecutionError> {
        if args.len() != 2 {
            return Err(ExecutionError::InvalidArgumentCount { expected: 2, got: args.len() });
        }

        let value = context.read_arg(&args[0])?;

        if let Argument::Register(reg) = args[1] {
            context.write_register(reg, value)?;
        } else {
            return Err(ExecutionError::InvalidArgumentType { expected: "Register", got: "Literal" });
        }

        Ok(())
    }

    fn arg_count(&self) -> usize { 2 }
    fn name(&self) -> &str { "COPY" }
    fn description(&self) -> &str { "Copy value to register" }
}

/// NOP: No operation
pub struct Nop;

impl Primitive for Nop {
    fn execute(&self, _args: &[Argument], _context: &mut ExecutionContext) -> Result<(), ExecutionError> {
        Ok(())
    }

    fn arg_count(&self) -> usize { 0 }
    fn name(&self) -> &str { "NOP" }
    fn description(&self) -> &str { "No operation" }
}
