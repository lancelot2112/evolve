//! Primitive Registry
//!
//! Manages the collection of available primitives and provides lookup by ID.
//!
//! The registry maintains all primitives in a Vec where the index corresponds
//! to the primitive ID used in DNA. The with_standard_primitives() constructor
//! registers the default set of primitives in the correct order.

use super::trait_def::Primitive;
use super::arithmetic::{Add, Sub, Mul, Div};
use super::stack::{Push, Pop};
use super::io::{ReadInput, WriteOutput};
use super::data::{Copy, Nop};
use super::markers::{TemplateStart, TemplateEnd};

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

        // Arithmetic (IDs 0-3)
        registry.register(Box::new(Add));
        registry.register(Box::new(Sub));
        registry.register(Box::new(Mul));
        registry.register(Box::new(Div));

        // Stack operations (IDs 4-5)
        registry.register(Box::new(Push));
        registry.register(Box::new(Pop));

        // I/O (IDs 6-7)
        registry.register(Box::new(ReadInput));
        registry.register(Box::new(WriteOutput));

        // Data movement (ID 8)
        registry.register(Box::new(Copy));

        // No-op (ID 9)
        registry.register(Box::new(Nop));

        // Template markers (IDs 10-11)
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
