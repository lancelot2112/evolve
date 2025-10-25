//! Neuron Module - Compartmental Neuron Primitives
//!
//! Biologically-inspired compartmental neurons with bit-packed state for
//! high-performance large-scale simulations.
//!
//! See docs/NEURON_DESIGN.md for detailed architecture documentation.

mod batch;
pub mod cache;
mod compartment;
mod interface;
mod learning;
mod packing;

pub use batch::{
    CountFiredNeurons, GetRandomCacheOffset, LoadCompartmentFromStack, ProcessCompartmentBatch,
    StoreCompartmentToStack, CACHE_LINE_SIZE,
};
pub use cache::{cache_line_size_i64, detect_cache_line_size, DEFAULT_CACHE_LINE_BYTES};
pub use compartment::{
    CheckFired, CreateCompartment, GetCompartmentLevel, ResetCompartment, UpdateCompartment,
};
pub use interface::{CreateInterface, DecayPacket, IntegratePacket, SendPacket};
pub use learning::{ApplyAntiHebbian, ApplyHebbian, ApplyHomeostatic, ApplySTDP};
pub use packing::{GetChannelCounts, SetChannelCounts};

// Bit masks and shifts for compartment state (64 bits)
pub mod compartment_bits {
    pub const EXC_SHIFT: u8 = 0;
    pub const EXC_MASK: i64 = 0xFFFFF; // 20 bits
    pub const EXC_SIGN_BIT: i64 = 0x80000;
    pub const EXC_SIGN_EXTEND: i32 = 0xFFF00000u32 as i32;

    pub const INH_SHIFT: u8 = 20;
    pub const INH_MASK: i64 = 0xFFFFF << INH_SHIFT; // 20 bits
    pub const INH_SIGN_BIT: i64 = 0x80000 << INH_SHIFT;

    pub const THRESHOLD_SHIFT: u8 = 40;
    pub const THRESHOLD_MASK: i64 = 0xFFF << THRESHOLD_SHIFT; // 12 bits

    pub const DECAY_SHIFT: u8 = 52;
    pub const DECAY_MASK: i64 = 0xF << DECAY_SHIFT; // 4 bits

    pub const LEARNING_SHIFT: u8 = 56;
    pub const LEARNING_MASK: i64 = 0xF << LEARNING_SHIFT; // 4 bits

    pub const FLAGS_SHIFT: u8 = 60;
    pub const FLAGS_MASK: i64 = 0xF << FLAGS_SHIFT; // 4 bits

    pub const FLAG_FIRED: i64 = 1 << 60;
    pub const FLAG_REFRACTORY: i64 = 1 << 61;
}

// Bit masks and shifts for axon interface state (64 bits)
pub mod interface_bits {
    pub const EXC_CHANNELS_SHIFT: u8 = 0;
    pub const EXC_CHANNELS_MASK: i64 = 0xFFFF; // 16 bits

    pub const INH_CHANNELS_SHIFT: u8 = 16;
    pub const INH_CHANNELS_MASK: i64 = 0xFFFF << INH_CHANNELS_SHIFT; // 16 bits

    pub const PACKET_VALUE_SHIFT: u8 = 32;
    pub const PACKET_VALUE_MASK: i64 = 0xFFFF << PACKET_VALUE_SHIFT; // 16 bits signed
    pub const PACKET_SIGN_BIT: i64 = 0x8000 << PACKET_VALUE_SHIFT;

    pub const PACKET_TTL_SHIFT: u8 = 48;
    pub const PACKET_TTL_MASK: i64 = 0xFF << PACKET_TTL_SHIFT; // 8 bits

    pub const DECAY_MODE_SHIFT: u8 = 56;
    pub const DECAY_MODE_MASK: i64 = 0xF << DECAY_MODE_SHIFT; // 4 bits

    pub const FLAGS_SHIFT: u8 = 60;
    pub const FLAGS_MASK: i64 = 0xF << FLAGS_SHIFT; // 4 bits
}

// Learning rule IDs
pub const LEARNING_HEBBIAN: i64 = 0;
pub const LEARNING_ANTI_HEBBIAN: i64 = 1;
pub const LEARNING_STDP: i64 = 2;
pub const LEARNING_HOMEOSTATIC: i64 = 3;

// Decay modes
pub const DECAY_EXPONENTIAL: i64 = 0;
pub const DECAY_FIXED_DURATION: i64 = 1;
