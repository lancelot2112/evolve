//! Arithmetic Primitives
//!
//! Basic mathematical operations (ADD, SUB, MUL, DIV) that operate on two values
//! and store the result in a register.
//!
//! All operations use wrapping arithmetic to prevent panics on overflow.
//! Division checks for division by zero and returns an error if encountered.

use super::context::{ExecutionContext, ExecutionError};
use super::trait_def::Primitive;
use crate::dna::Argument;

/// ADD: args[0] + args[1] -> register args[2]
pub struct Add;

impl Primitive for Add {
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

        let a = context.read_arg(&args[0])?;
        let b = context.read_arg(&args[1])?;
        let result = a.wrapping_add(b);

        if let Argument::Register(reg) = args[2] {
            context.write_register(reg, result)?;
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
        "ADD"
    }
    fn description(&self) -> &str {
        "Add two values and store in register"
    }
}

/// SUB: args[0] - args[1] -> register args[2]
pub struct Sub;

impl Primitive for Sub {
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

        let a = context.read_arg(&args[0])?;
        let b = context.read_arg(&args[1])?;
        let result = a.wrapping_sub(b);

        if let Argument::Register(reg) = args[2] {
            context.write_register(reg, result)?;
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
        "SUB"
    }
    fn description(&self) -> &str {
        "Subtract two values and store in register"
    }
}

/// MUL: args[0] * args[1] -> register args[2]
pub struct Mul;

impl Primitive for Mul {
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

        let a = context.read_arg(&args[0])?;
        let b = context.read_arg(&args[1])?;
        let result = a.wrapping_mul(b);

        if let Argument::Register(reg) = args[2] {
            context.write_register(reg, result)?;
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
        "MUL"
    }
    fn description(&self) -> &str {
        "Multiply two values and store in register"
    }
}

/// DIV: args[0] / args[1] -> register args[2]
pub struct Div;

impl Primitive for Div {
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

        let a = context.read_arg(&args[0])?;
        let b = context.read_arg(&args[1])?;

        if b == 0 {
            return Err(ExecutionError::DivisionByZero);
        }

        let result = a / b;

        if let Argument::Register(reg) = args[2] {
            context.write_register(reg, result)?;
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
        "DIV"
    }
    fn description(&self) -> &str {
        "Divide two values and store in register"
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_add_primitive() {
        let mut ctx = ExecutionContext::new(4, vec![], 1000, 100);
        ctx.registers[0] = 5;
        ctx.registers[1] = 3;

        let add = Add;
        let args = vec![
            Argument::Register(0),
            Argument::Register(1),
            Argument::Register(2),
        ];

        add.execute(&args, &mut ctx).unwrap();
        assert_eq!(ctx.registers[2], 8);
    }
}
