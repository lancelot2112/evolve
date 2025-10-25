//! Primitive Registry
//!
//! Manages the collection of available primitives and provides lookup by ID.
//!
//! The registry maintains all primitives in a Vec where the index corresponds
//! to the primitive ID used in DNA. The with_standard_primitives() constructor
//! registers the default set of primitives in the correct order.

use super::arithmetic::{Add, Div, Mul, Sub};
use super::data::{Copy, Nop};
use super::io::{ReadInput, WriteOutput};
use super::markers::{TemplateEnd, TemplateStart};
use super::neuron::{
    ApplyAntiHebbian, ApplyHebbian, ApplyHomeostatic, ApplySTDP, CheckFired, CountFiredNeurons,
    CreateCompartment, CreateInterface, DecayPacket, GetChannelCounts, GetCompartmentLevel,
    GetRandomCacheOffset, IntegratePacket, LoadCompartmentFromStack, ProcessCompartmentBatch,
    ResetCompartment, SendPacket, SetChannelCounts, StoreCompartmentToStack, UpdateCompartment,
};
use super::stack::{Pop, Push};
use super::trait_def::Primitive;

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

    /// Create a registry with neuron primitives only
    pub fn with_neuron_primitives() -> Self {
        let mut registry = Self::new();

        // Compartment operations (IDs 0-4)
        registry.register(Box::new(CreateCompartment));
        registry.register(Box::new(UpdateCompartment));
        registry.register(Box::new(GetCompartmentLevel));
        registry.register(Box::new(CheckFired));
        registry.register(Box::new(ResetCompartment));

        // Axon interface operations (IDs 5-8)
        registry.register(Box::new(CreateInterface));
        registry.register(Box::new(SendPacket));
        registry.register(Box::new(IntegratePacket));
        registry.register(Box::new(DecayPacket));

        // Learning operations (IDs 9-12)
        registry.register(Box::new(ApplyHebbian));
        registry.register(Box::new(ApplyAntiHebbian));
        registry.register(Box::new(ApplySTDP));
        registry.register(Box::new(ApplyHomeostatic));

        // Packing utilities (IDs 13-14)
        registry.register(Box::new(GetChannelCounts));
        registry.register(Box::new(SetChannelCounts));

        // Batch processing (IDs 15-19)
        registry.register(Box::new(GetRandomCacheOffset));
        registry.register(Box::new(ProcessCompartmentBatch));
        registry.register(Box::new(CountFiredNeurons));
        registry.register(Box::new(LoadCompartmentFromStack));
        registry.register(Box::new(StoreCompartmentToStack));

        // Template markers (IDs 20-21)
        registry.register(Box::new(TemplateStart));
        registry.register(Box::new(TemplateEnd));

        // Basic I/O for debugging (IDs 22-23)
        registry.register(Box::new(ReadInput));
        registry.register(Box::new(WriteOutput));

        // Stack operations for neuron arrays (IDs 24-25)
        registry.register(Box::new(Push));
        registry.register(Box::new(Pop));

        registry
    }

    /// Create a registry with standard primitives (legacy)
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
