//! I/O Primitives
//!
//! Operations for reading input and writing output (READ_INPUT, WRITE_OUTPUT).
//!
//! These primitives provide the interface between the DNA program and the
//! external world. READ_INPUT consumes values from the input stream, and
//! WRITE_OUTPUT appends values to the output stream.

use crate::dna::Argument;
use super::context::{ExecutionContext, ExecutionError};
use super::trait_def::Primitive;

/// READ_INPUT: Read next input value into register args[0]
pub struct ReadInput;

impl Primitive for ReadInput {
    fn execute(&self, args: &[Argument], context: &mut ExecutionContext) -> Result<(), ExecutionError> {
        if args.len() != 1 {
            return Err(ExecutionError::InvalidArgumentCount { expected: 1, got: args.len() });
        }

        let value = if context.input_cursor < context.input.len() {
            let val = context.input[context.input_cursor];
            context.input_cursor += 1;
            val
        } else {
            0 // Default to 0 if input exhausted
        };

        if let Argument::Register(reg) = args[0] {
            context.write_register(reg, value)?;
        } else {
            return Err(ExecutionError::InvalidArgumentType { expected: "Register", got: "Literal" });
        }

        Ok(())
    }

    fn arg_count(&self) -> usize { 1 }
    fn name(&self) -> &str { "READ_INPUT" }
    fn description(&self) -> &str { "Read next input value into register" }
}

/// WRITE_OUTPUT: Write args[0] to output
pub struct WriteOutput;

impl Primitive for WriteOutput {
    fn execute(&self, args: &[Argument], context: &mut ExecutionContext) -> Result<(), ExecutionError> {
        if args.len() != 1 {
            return Err(ExecutionError::InvalidArgumentCount { expected: 1, got: args.len() });
        }

        let value = context.read_arg(&args[0])?;
        context.output.push(value);

        Ok(())
    }

    fn arg_count(&self) -> usize { 1 }
    fn name(&self) -> &str { "WRITE_OUTPUT" }
    fn description(&self) -> &str { "Write value to output" }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_io_operations() {
        let mut ctx = ExecutionContext::new(4, vec![10, 20, 30], 1000, 100);

        let read = ReadInput;
        read.execute(&[Argument::Register(0)], &mut ctx).unwrap();
        assert_eq!(ctx.registers[0], 10);

        let write = WriteOutput;
        write.execute(&[Argument::Register(0)], &mut ctx).unwrap();
        assert_eq!(ctx.output, vec![10]);
    }
}
