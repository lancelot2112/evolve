//! Stack Primitives
//!
//! Operations for manipulating the data stack (PUSH, POP).
//!
//! The stack provides temporary storage for values during computation.
//! Stack overflow and underflow are checked and return errors when limits
//! are exceeded.

use super::context::{ExecutionContext, ExecutionError};
use super::trait_def::Primitive;
use crate::dna::Argument;

/// PUSH: Push args[0] onto stack
pub struct Push;

impl Primitive for Push {
    fn execute(
        &self,
        args: &[Argument],
        context: &mut ExecutionContext,
    ) -> Result<(), ExecutionError> {
        if args.len() != 1 {
            return Err(ExecutionError::InvalidArgumentCount {
                expected: 1,
                got: args.len(),
            });
        }

        let value = context.read_arg(&args[0])?;
        context.stack_push(value)?;

        Ok(())
    }

    fn arg_count(&self) -> usize {
        1
    }
    fn name(&self) -> &str {
        "PUSH"
    }
    fn description(&self) -> &str {
        "Push value onto stack"
    }
}

/// POP: Pop from stack into register args[0]
pub struct Pop;

impl Primitive for Pop {
    fn execute(
        &self,
        args: &[Argument],
        context: &mut ExecutionContext,
    ) -> Result<(), ExecutionError> {
        if args.len() != 1 {
            return Err(ExecutionError::InvalidArgumentCount {
                expected: 1,
                got: args.len(),
            });
        }

        let value = context.stack_pop()?;

        if let Argument::Register(reg) = args[0] {
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
        1
    }
    fn name(&self) -> &str {
        "POP"
    }
    fn description(&self) -> &str {
        "Pop value from stack into register"
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_stack_operations() {
        let mut ctx = ExecutionContext::new(4, vec![], 1000, 100);

        let push = Push;
        push.execute(&[Argument::Literal(42)], &mut ctx).unwrap();
        assert_eq!(ctx.stack.len(), 1);

        let pop = Pop;
        pop.execute(&[Argument::Register(0)], &mut ctx).unwrap();
        assert_eq!(ctx.registers[0], 42);
        assert_eq!(ctx.stack.len(), 0);
    }
}
