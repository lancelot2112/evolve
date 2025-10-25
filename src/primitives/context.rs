//! Execution Context and Errors
//!
//! Provides the execution environment for primitives including registers, stack,
//! I/O, and instruction limits.
//!
//! The ExecutionContext maintains all runtime state for executing DNA sequences.
//! It enforces safety limits on instruction count and stack depth to prevent
//! runaway execution.

use crate::dna::Argument;
use std::fmt;

/// Errors that can occur during primitive execution
#[derive(Debug, Clone, PartialEq)]
pub enum ExecutionError {
    InvalidArgumentCount {
        expected: usize,
        got: usize,
    },
    InvalidArgumentType {
        expected: &'static str,
        got: &'static str,
    },
    RegisterOutOfBounds {
        register: u8,
        max: u8,
    },
    StackUnderflow,
    StackOverflow,
    DivisionByZero,
    MaxInstructionsExceeded,
    Other(String),
}

impl fmt::Display for ExecutionError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            Self::InvalidArgumentCount { expected, got } => {
                write!(
                    f,
                    "Invalid argument count: expected {}, got {}",
                    expected, got
                )
            }
            Self::InvalidArgumentType { expected, got } => {
                write!(
                    f,
                    "Invalid argument type: expected {}, got {}",
                    expected, got
                )
            }
            Self::RegisterOutOfBounds { register, max } => {
                write!(f, "Register R{} out of bounds (max: R{})", register, max)
            }
            Self::StackUnderflow => write!(f, "Stack underflow"),
            Self::StackOverflow => write!(f, "Stack overflow"),
            Self::DivisionByZero => write!(f, "Division by zero"),
            Self::MaxInstructionsExceeded => write!(f, "Maximum instruction count exceeded"),
            Self::Other(msg) => write!(f, "{}", msg),
        }
    }
}

impl std::error::Error for ExecutionError {}

/// Execution context for primitives
pub struct ExecutionContext {
    /// Register file (variables)
    pub registers: Vec<i64>,
    /// Data stack
    pub stack: Vec<i64>,
    /// Input data
    pub input: Vec<i64>,
    /// Output data
    pub output: Vec<i64>,
    /// Instruction counter
    pub instruction_count: usize,
    /// Max allowed instructions
    pub max_instructions: usize,
    /// Max stack depth
    pub max_stack_depth: usize,
    /// Input cursor for reading
    pub input_cursor: usize,
}

impl ExecutionContext {
    pub fn new(
        register_count: usize,
        input: Vec<i64>,
        max_instructions: usize,
        max_stack_depth: usize,
    ) -> Self {
        Self {
            registers: vec![0; register_count],
            stack: Vec::new(),
            input,
            output: Vec::new(),
            instruction_count: 0,
            max_instructions,
            max_stack_depth,
            input_cursor: 0,
        }
    }

    /// Read a value from an argument
    pub fn read_arg(&self, arg: &Argument) -> Result<i64, ExecutionError> {
        match arg {
            Argument::Literal(val) => Ok(*val),
            Argument::Register(reg) => {
                if (*reg as usize) < self.registers.len() {
                    Ok(self.registers[*reg as usize])
                } else {
                    Err(ExecutionError::RegisterOutOfBounds {
                        register: *reg,
                        max: (self.registers.len() - 1) as u8,
                    })
                }
            }
        }
    }

    /// Write a value to a register
    pub fn write_register(&mut self, reg: u8, value: i64) -> Result<(), ExecutionError> {
        if (reg as usize) < self.registers.len() {
            self.registers[reg as usize] = value;
            Ok(())
        } else {
            Err(ExecutionError::RegisterOutOfBounds {
                register: reg,
                max: (self.registers.len() - 1) as u8,
            })
        }
    }

    /// Push to stack
    pub fn stack_push(&mut self, value: i64) -> Result<(), ExecutionError> {
        if self.stack.len() >= self.max_stack_depth {
            Err(ExecutionError::StackOverflow)
        } else {
            self.stack.push(value);
            Ok(())
        }
    }

    /// Pop from stack
    pub fn stack_pop(&mut self) -> Result<i64, ExecutionError> {
        self.stack.pop().ok_or(ExecutionError::StackUnderflow)
    }

    /// Increment instruction counter and check limit
    pub fn tick(&mut self) -> Result<(), ExecutionError> {
        self.instruction_count += 1;
        if self.instruction_count > self.max_instructions {
            Err(ExecutionError::MaxInstructionsExceeded)
        } else {
            Ok(())
        }
    }
}
