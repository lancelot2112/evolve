//! Axon-Compartment Interface Primitives
//!
//! Operations for managing connections between axons and compartments,
//! including packet-based signal transmission.

use super::compartment_bits as comp;
use super::interface_bits::*;
use crate::dna::Argument;
use crate::primitives::{ExecutionContext, ExecutionError, Primitive};

/// CreateInterface: Creates a new axon-compartment interface
/// Args: [exc_channels, inh_channels, output_reg]
pub struct CreateInterface;

impl Primitive for CreateInterface {
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

        let exc_channels = context.read_arg(&args[0])? & 0xFFFF; // 16 bits max
        let inh_channels = context.read_arg(&args[1])? & 0xFFFF; // 16 bits max

        // Pack interface state (packet_value=0, ttl=0, decay_mode=exponential)
        let state = exc_channels | (inh_channels << INH_CHANNELS_SHIFT);

        if let Argument::Register(reg) = args[2] {
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
        3
    }

    fn name(&self) -> &str {
        "CreateInterface"
    }

    fn description(&self) -> &str {
        "Create axon-compartment interface with channel counts"
    }
}

/// SendPacket: Initiate packet transmission
/// Args: [interface_reg, packet_value, ttl, output_reg]
pub struct SendPacket;

impl Primitive for SendPacket {
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
        let packet_value = (context.read_arg(&args[1])? as i16) as i64; // Sign extend to 16 bits
        let ttl = context.read_arg(&args[2])? & 0xFF; // 8 bits max

        // Clear old packet fields
        state &= !(PACKET_VALUE_MASK | PACKET_TTL_MASK);

        // Set new packet fields
        state |= ((packet_value & 0xFFFF) << PACKET_VALUE_SHIFT) & PACKET_VALUE_MASK;
        state |= (ttl << PACKET_TTL_SHIFT) & PACKET_TTL_MASK;

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
        "SendPacket"
    }

    fn description(&self) -> &str {
        "Send packet through axon interface with specified value and TTL"
    }
}

/// IntegratePacket: Add packet contribution to compartment
/// Args: [interface_reg, compartment_reg, output_compartment_reg, output_interface_reg]
pub struct IntegratePacket;

impl Primitive for IntegratePacket {
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

        let mut interface_state = context.read_arg(&args[0])?;
        let mut compartment_state = context.read_arg(&args[1])?;

        // Extract interface fields
        let exc_channels = (interface_state & EXC_CHANNELS_MASK) as i32;
        let inh_channels = ((interface_state & INH_CHANNELS_MASK) >> INH_CHANNELS_SHIFT) as i32;
        let mut packet_value =
            (((interface_state & PACKET_VALUE_MASK) >> PACKET_VALUE_SHIFT) as i16) as i32; // Sign extend
        let mut ttl = ((interface_state & PACKET_TTL_MASK) >> PACKET_TTL_SHIFT) as i32;
        let decay_mode = ((interface_state & DECAY_MODE_MASK) >> DECAY_MODE_SHIFT) as i32;

        // Only integrate if packet is active (ttl > 0)
        if ttl > 0 {
            // Calculate contributions using fast bit shift approximation
            // contribution = (packet_value * channels) >> 8 ≈ packet_value * channels / 256
            let exc_contribution = (packet_value * exc_channels) >> 8;
            let inh_contribution = (packet_value * inh_channels) >> 8;

            // Extract compartment levels
            let mut excitatory =
                ((compartment_state & comp::EXC_MASK) as i32) << 12 >> 12; // Sign extend
            let mut inhibitory =
                (((compartment_state & comp::INH_MASK) >> comp::INH_SHIFT) as i32) << 12 >> 12; // Sign extend

            // Add contributions
            excitatory = excitatory.saturating_add(exc_contribution);
            inhibitory = inhibitory.saturating_add(inh_contribution);

            // Clamp to 20-bit signed range (-524288 to 524287)
            excitatory = excitatory.max(-524288).min(524287);
            inhibitory = inhibitory.max(-524288).min(524287);

            // Update compartment state
            compartment_state &= !(comp::EXC_MASK | comp::INH_MASK);
            compartment_state |= (excitatory as i64) & comp::EXC_MASK;
            compartment_state |= ((inhibitory as i64) << comp::INH_SHIFT) & comp::INH_MASK;

            // Update packet based on decay mode
            if decay_mode == 0 {
                // Exponential decay: packet_value >>= 1 (halve each step)
                packet_value >>= 1;
            } else {
                // Fixed duration: decrement TTL
                ttl -= 1;
            }

            // Update interface state
            interface_state &= !(PACKET_VALUE_MASK | PACKET_TTL_MASK);
            interface_state |= ((packet_value as i64 & 0xFFFF) << PACKET_VALUE_SHIFT)
                & PACKET_VALUE_MASK;
            interface_state |= ((ttl as i64) << PACKET_TTL_SHIFT) & PACKET_TTL_MASK;
        }

        if let Argument::Register(reg) = args[2] {
            context.write_register(reg, compartment_state)?;
        } else {
            return Err(ExecutionError::InvalidArgumentType {
                expected: "Register",
                got: "Literal",
            });
        }

        if let Argument::Register(reg) = args[3] {
            context.write_register(reg, interface_state)?;
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
        "IntegratePacket"
    }

    fn description(&self) -> &str {
        "Integrate packet contribution into compartment and update packet state"
    }
}

/// DecayPacket: Manually decay packet value
/// Args: [interface_reg, decay_shift, output_reg]
pub struct DecayPacket;

impl Primitive for DecayPacket {
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

        let mut state = context.read_arg(&args[0])?;
        let decay_shift = (context.read_arg(&args[1])? & 0xF) as u8; // 4 bits max

        // Extract packet value
        let mut packet_value =
            (((state & PACKET_VALUE_MASK) >> PACKET_VALUE_SHIFT) as i16) as i32;

        // Apply decay
        packet_value >>= decay_shift;

        // Update state
        state &= !PACKET_VALUE_MASK;
        state |= ((packet_value as i64 & 0xFFFF) << PACKET_VALUE_SHIFT) & PACKET_VALUE_MASK;

        if let Argument::Register(reg) = args[2] {
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
        3
    }

    fn name(&self) -> &str {
        "DecayPacket"
    }

    fn description(&self) -> &str {
        "Apply exponential decay to packet value using bit shift"
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::primitives::neuron::compartment::CreateCompartment;

    #[test]
    fn test_create_interface() {
        let mut ctx = ExecutionContext::new(4, vec![], 1000, 100);
        let create = CreateInterface;

        create
            .execute(
                &[
                    Argument::Literal(100), // exc_channels
                    Argument::Literal(50),  // inh_channels
                    Argument::Register(0),
                ],
                &mut ctx,
            )
            .unwrap();

        let state = ctx.registers[0];
        let exc = (state & EXC_CHANNELS_MASK) as i32;
        let inh = ((state & INH_CHANNELS_MASK) >> INH_CHANNELS_SHIFT) as i32;

        assert_eq!(exc, 100);
        assert_eq!(inh, 50);
    }

    #[test]
    fn test_integrate_packet() {
        let mut ctx = ExecutionContext::new(8, vec![], 1000, 100);

        // Create interface with exc=256, inh=0
        let create_iface = CreateInterface;
        create_iface
            .execute(
                &[
                    Argument::Literal(256),
                    Argument::Literal(0),
                    Argument::Register(0),
                ],
                &mut ctx,
            )
            .unwrap();

        // Send packet with value=100, ttl=5
        let send = SendPacket;
        send.execute(
            &[
                Argument::Register(0),
                Argument::Literal(100),
                Argument::Literal(5),
                Argument::Register(1),
            ],
            &mut ctx,
        )
        .unwrap();

        // Create compartment
        let create_comp = CreateCompartment;
        create_comp
            .execute(
                &[
                    Argument::Literal(50),
                    Argument::Literal(0), // no decay
                    Argument::Literal(0),
                    Argument::Register(2),
                ],
                &mut ctx,
            )
            .unwrap();

        // Integrate packet
        let integrate = IntegratePacket;
        integrate
            .execute(
                &[
                    Argument::Register(1), // interface
                    Argument::Register(2), // compartment
                    Argument::Register(3), // output compartment
                    Argument::Register(4), // output interface
                ],
                &mut ctx,
            )
            .unwrap();

        // Check that excitatory level increased
        // contribution = (100 * 256) >> 8 = 100
        let comp_state = ctx.registers[3];
        let excitatory = ((comp_state & comp::EXC_MASK) as i32) << 12 >> 12;
        assert_eq!(excitatory, 100);
    }
}
