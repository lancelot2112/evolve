/// Primitives module: User-defined base operations
///
/// Primitives are the fundamental operations that users define for their problem domain.
/// The evolutionary algorithm combines these primitives to create solutions.

use crate::dna::Argument;
use std::fmt;

/// Errors that can occur during primitive execution
#[derive(Debug, Clone, PartialEq)]
pub enum ExecutionError {
    InvalidArgumentCount { expected: usize, got: usize },
    InvalidArgumentType { expected: &'static str, got: &'static str },
    RegisterOutOfBounds { register: u8, max: u8 },
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
                write!(f, "Invalid argument count: expected {}, got {}", expected, got)
            }
            Self::InvalidArgumentType { expected, got } => {
                write!(f, "Invalid argument type: expected {}, got {}", expected, got)
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

/// Trait for primitive operations
pub trait Primitive: Send + Sync {
    /// Execute the primitive operation
    fn execute(&self, args: &[Argument], context: &mut ExecutionContext) -> Result<(), ExecutionError>;

    /// Expected number of arguments
    fn arg_count(&self) -> usize;

    /// Name of the primitive
    fn name(&self) -> &str;

    /// Description of what this primitive does
    fn description(&self) -> &str {
        "No description available"
    }
}

/// Registry of all available primitives
pub struct PrimitiveRegistry {
    primitives: Vec<Box<dyn Primitive>>,
}

impl PrimitiveRegistry {
    pub fn new() -> Self {
        Self {
            primitives: Vec::new(),
        }
    }

    /// Register a new primitive
    pub fn register(&mut self, primitive: Box<dyn Primitive>) {
        self.primitives.push(primitive);
    }

    /// Get a primitive by ID
    pub fn get(&self, id: u16) -> Option<&dyn Primitive> {
        self.primitives.get(id as usize).map(|b| b.as_ref())
    }

    /// Get the number of registered primitives
    pub fn count(&self) -> usize {
        self.primitives.len()
    }

    /// Create a registry with standard primitives
    pub fn with_standard_primitives() -> Self {
        let mut registry = Self::new();

        // Arithmetic
        registry.register(Box::new(Add));
        registry.register(Box::new(Sub));
        registry.register(Box::new(Mul));
        registry.register(Box::new(Div));

        // Stack operations
        registry.register(Box::new(Push));
        registry.register(Box::new(Pop));

        // I/O
        registry.register(Box::new(ReadInput));
        registry.register(Box::new(WriteOutput));

        // Data movement
        registry.register(Box::new(Copy));

        // No-op
        registry.register(Box::new(Nop));

        // Template markers (don't execute, used for detection)
        registry.register(Box::new(TemplateStart));
        registry.register(Box::new(TemplateEnd));

        registry
    }
}

impl Default for PrimitiveRegistry {
    fn default() -> Self {
        Self::new()
    }
}

// ============================================================================
// Standard Primitive Implementations
// ============================================================================

/// ADD: args[0] + args[1] -> register args[2]
pub struct Add;

impl Primitive for Add {
    fn execute(&self, args: &[Argument], context: &mut ExecutionContext) -> Result<(), ExecutionError> {
        if args.len() != 3 {
            return Err(ExecutionError::InvalidArgumentCount { expected: 3, got: args.len() });
        }

        let a = context.read_arg(&args[0])?;
        let b = context.read_arg(&args[1])?;
        let result = a.wrapping_add(b);

        if let Argument::Register(reg) = args[2] {
            context.write_register(reg, result)?;
        } else {
            return Err(ExecutionError::InvalidArgumentType { expected: "Register", got: "Literal" });
        }

        Ok(())
    }

    fn arg_count(&self) -> usize { 3 }
    fn name(&self) -> &str { "ADD" }
    fn description(&self) -> &str { "Add two values and store in register" }
}

/// SUB: args[0] - args[1] -> register args[2]
pub struct Sub;

impl Primitive for Sub {
    fn execute(&self, args: &[Argument], context: &mut ExecutionContext) -> Result<(), ExecutionError> {
        if args.len() != 3 {
            return Err(ExecutionError::InvalidArgumentCount { expected: 3, got: args.len() });
        }

        let a = context.read_arg(&args[0])?;
        let b = context.read_arg(&args[1])?;
        let result = a.wrapping_sub(b);

        if let Argument::Register(reg) = args[2] {
            context.write_register(reg, result)?;
        } else {
            return Err(ExecutionError::InvalidArgumentType { expected: "Register", got: "Literal" });
        }

        Ok(())
    }

    fn arg_count(&self) -> usize { 3 }
    fn name(&self) -> &str { "SUB" }
    fn description(&self) -> &str { "Subtract two values and store in register" }
}

/// MUL: args[0] * args[1] -> register args[2]
pub struct Mul;

impl Primitive for Mul {
    fn execute(&self, args: &[Argument], context: &mut ExecutionContext) -> Result<(), ExecutionError> {
        if args.len() != 3 {
            return Err(ExecutionError::InvalidArgumentCount { expected: 3, got: args.len() });
        }

        let a = context.read_arg(&args[0])?;
        let b = context.read_arg(&args[1])?;
        let result = a.wrapping_mul(b);

        if let Argument::Register(reg) = args[2] {
            context.write_register(reg, result)?;
        } else {
            return Err(ExecutionError::InvalidArgumentType { expected: "Register", got: "Literal" });
        }

        Ok(())
    }

    fn arg_count(&self) -> usize { 3 }
    fn name(&self) -> &str { "MUL" }
    fn description(&self) -> &str { "Multiply two values and store in register" }
}

/// DIV: args[0] / args[1] -> register args[2]
pub struct Div;

impl Primitive for Div {
    fn execute(&self, args: &[Argument], context: &mut ExecutionContext) -> Result<(), ExecutionError> {
        if args.len() != 3 {
            return Err(ExecutionError::InvalidArgumentCount { expected: 3, got: args.len() });
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
            return Err(ExecutionError::InvalidArgumentType { expected: "Register", got: "Literal" });
        }

        Ok(())
    }

    fn arg_count(&self) -> usize { 3 }
    fn name(&self) -> &str { "DIV" }
    fn description(&self) -> &str { "Divide two values and store in register" }
}

/// PUSH: Push args[0] onto stack
pub struct Push;

impl Primitive for Push {
    fn execute(&self, args: &[Argument], context: &mut ExecutionContext) -> Result<(), ExecutionError> {
        if args.len() != 1 {
            return Err(ExecutionError::InvalidArgumentCount { expected: 1, got: args.len() });
        }

        let value = context.read_arg(&args[0])?;
        context.stack_push(value)?;

        Ok(())
    }

    fn arg_count(&self) -> usize { 1 }
    fn name(&self) -> &str { "PUSH" }
    fn description(&self) -> &str { "Push value onto stack" }
}

/// POP: Pop from stack into register args[0]
pub struct Pop;

impl Primitive for Pop {
    fn execute(&self, args: &[Argument], context: &mut ExecutionContext) -> Result<(), ExecutionError> {
        if args.len() != 1 {
            return Err(ExecutionError::InvalidArgumentCount { expected: 1, got: args.len() });
        }

        let value = context.stack_pop()?;

        if let Argument::Register(reg) = args[0] {
            context.write_register(reg, value)?;
        } else {
            return Err(ExecutionError::InvalidArgumentType { expected: "Register", got: "Literal" });
        }

        Ok(())
    }

    fn arg_count(&self) -> usize { 1 }
    fn name(&self) -> &str { "POP" }
    fn description(&self) -> &str { "Pop value from stack into register" }
}

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

/// TEMPLATE_START: Marker for template start (doesn't execute)
pub struct TemplateStart;

impl Primitive for TemplateStart {
    fn execute(&self, _args: &[Argument], _context: &mut ExecutionContext) -> Result<(), ExecutionError> {
        // Marker only - no execution
        Ok(())
    }

    fn arg_count(&self) -> usize { 0 }
    fn name(&self) -> &str { "TEMPLATE_START" }
    fn description(&self) -> &str { "Marks the beginning of a template sequence" }
}

/// TEMPLATE_END: Marker for template end (doesn't execute)
pub struct TemplateEnd;

impl Primitive for TemplateEnd {
    fn execute(&self, _args: &[Argument], _context: &mut ExecutionContext) -> Result<(), ExecutionError> {
        // Marker only - no execution
        Ok(())
    }

    fn arg_count(&self) -> usize { 0 }
    fn name(&self) -> &str { "TEMPLATE_END" }
    fn description(&self) -> &str { "Marks the end of a template sequence" }
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
