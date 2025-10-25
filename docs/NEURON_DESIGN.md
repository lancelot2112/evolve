# Compartmental Neuron Architecture

## Overview
This design implements biologically-inspired compartmental neurons optimized for speed and minimal memory footprint. The system uses bit-packed state representation to enable large-scale network simulations.

## Core Concepts

### Compartments
Compartments represent dendritic sections or soma that integrate incoming signals:
- **Excitatory level**: Accumulated positive input
- **Inhibitory level**: Accumulated negative input
- **Threshold**: Firing threshold
- **Decay rate**: How fast levels decrease (via bit shift for speed)
- **Learning rule**: Local plasticity rule for updating connections

### Axon-Compartment Interfaces
Connections between axons and compartments with:
- **Excitatory channels**: Count of excitatory ion channels (acts as weight)
- **Inhibitory channels**: Count of inhibitory ion channels
- **Packet value**: Signal strength being transmitted
- **Packet TTL**: Time-to-live for packet transmission
- **Decay mode**: Fixed duration vs exponential decay

### Learning Rules
Local plasticity rules that update channel counts:
- **Hebbian**: Strengthen connections that fire together
- **Anti-Hebbian**: Decorrelate simultaneous activity
- **STDP-like**: Timing-dependent plasticity
- **Homeostatic**: Maintain target activity level

## Bit-Packed State Representation

### Compartment State (64 bits packed in i64)
```
Bits  0-19: Excitatory level (20 bits, signed, -524288 to 524287)
Bits 20-39: Inhibitory level (20 bits, signed, -524288 to 524287)
Bits 40-51: Threshold (12 bits, unsigned, 0-4095)
Bits 52-55: Decay shift (4 bits, unsigned, 0-15)
Bits 56-59: Learning rule ID (4 bits, unsigned, 0-15)
Bits 60-63: Flags (4 bits: fired, refractory, etc.)
```

### Axon Interface State (64 bits packed in i64)
```
Bits  0-15: Excitatory channels (16 bits, unsigned, 0-65535)
Bits 16-31: Inhibitory channels (16 bits, unsigned, 0-65535)
Bits 32-47: Packet value (16 bits, signed, -32768 to 32767)
Bits 48-55: Packet TTL (8 bits, unsigned, 0-255 steps)
Bits 56-59: Decay mode (4 bits)
Bits 60-63: Flags (4 bits)
```

## Primitive Operations

### Compartment Operations

1. **CreateCompartment** (args: threshold, decay_shift, learning_rule -> reg)
   - Creates a new compartment with initial state
   - Returns packed i64 state

2. **UpdateCompartment** (args: compartment_reg -> compartment_reg)
   - Applies decay: `level >>= decay_shift` (fast bit shift!)
   - Checks threshold: fires if `(excitatory - inhibitory) >= threshold`
   - Sets fired flag if threshold exceeded
   - Returns updated compartment state

3. **GetCompartmentLevel** (args: compartment_reg -> output_reg)
   - Extracts net level: `excitatory - inhibitory`
   - Useful for reading output

4. **CheckFired** (args: compartment_reg -> output_reg)
   - Extracts fired flag
   - Returns 1 if fired this step, 0 otherwise

5. **ResetCompartment** (args: compartment_reg -> compartment_reg)
   - Clears fired flag and optionally resets levels
   - Implements refractory period

### Axon Interface Operations

6. **CreateInterface** (args: exc_channels, inh_channels -> reg)
   - Creates axon-compartment interface
   - Returns packed i64 state

7. **SendPacket** (args: interface_reg, packet_value, ttl -> interface_reg)
   - Initiates packet transmission
   - Value scaled by channel counts:
     - `exc_contribution = (packet_value * exc_channels) >> 8` (fast approximation)
     - `inh_contribution = (packet_value * inh_channels) >> 8`

8. **IntegratePacket** (args: interface_reg, compartment_reg -> compartment_reg, interface_reg)
   - Adds packet contribution to compartment levels
   - Updates packet (decay or decrement TTL)
   - Returns updated compartment and interface

9. **DecayPacket** (args: interface_reg, decay_shift -> interface_reg)
   - Exponential decay: `packet_value >>= decay_shift`
   - Or fixed duration: `ttl -= 1`

### Learning Operations

10. **ApplyHebbian** (args: interface_reg, pre_fired, post_fired, rate -> interface_reg)
    - If both fired: `exc_channels += rate`
    - Strengthens co-active connections

11. **ApplyAntiHebbian** (args: interface_reg, pre_fired, post_fired, rate -> interface_reg)
    - If both fired: `exc_channels -= rate`
    - Decorrelates activity

12. **ApplySTDP** (args: interface_reg, pre_time, post_time, rate -> interface_reg)
    - Time difference-based plasticity
    - Uses bit shifts for fast exponential windows

13. **ApplyHomeostatic** (args: interface_reg, target_rate, actual_rate, rate -> interface_reg)
    - Scales all channels to maintain target firing rate
    - `channels += (target - actual) * rate >> 8`

### Utility Operations

14. **GetChannelCounts** (args: interface_reg -> exc_reg, inh_reg)
    - Extracts channel counts for inspection

15. **SetChannelCounts** (args: interface_reg, exc_count, inh_count -> interface_reg)
    - Updates channel counts directly

16. **PackCompartment** (args: exc, inh, threshold, decay, rule -> reg)
    - Manually pack compartment state
    - For advanced users

17. **UnpackCompartment** (args: compartment_reg -> exc, inh, threshold, decay, rule)
    - Extract all fields
    - For debugging/inspection

## Performance Optimizations

### No Division
- All decay uses bit shifts: `value >>= shift` instead of `value / divisor`
- Channel scaling uses shifts: `(value * count) >> 8` ≈ `value * count / 256`

### Minimal Memory
- Complete compartment state: 8 bytes
- Complete interface state: 8 bytes
- 1 million neurons: ~16 MB (with avg 100 connections each)

### Fast Field Access
```rust
// Extract 20-bit signed excitatory (bits 0-19)
const EXC_MASK: i64 = 0xFFFFF;
let exc = (state & EXC_MASK) as i32;
if exc & 0x80000 != 0 { exc |= 0xFFF00000; } // sign extend

// Update excitatory (bits 0-19)
state = (state & !EXC_MASK) | ((new_exc as i64) & EXC_MASK);
```

### Vectorization Ready
- Bit-packed representation enables SIMD operations
- Future: process multiple compartments in parallel

## Example Usage

```rust
// Create compartment: threshold=1000, decay_shift=2 (divide by 4 each step), Hebbian learning
let compartment = CreateCompartment(1000, 2, 0);

// Create two input interfaces
let input1 = CreateInterface(100, 20);  // More excitatory
let input2 = CreateInterface(30, 80);   // More inhibitory

// Simulation step
let input1 = SendPacket(input1, 500, 10);  // Send packet value 500, TTL 10
let (compartment, input1) = IntegratePacket(input1, compartment);
let (compartment, input2) = IntegratePacket(input2, compartment);

let compartment = UpdateCompartment(compartment);  // Apply decay, check threshold

if CheckFired(compartment) {
    // Apply learning
    let input1 = ApplyHebbian(input1, 1, 1, 5);  // Strengthen this connection
    let compartment = ResetCompartment(compartment);
}
```

## Evolution Implications

### Template Formation
Useful neuron structures will emerge as templates:
- Feature detectors (specific channel configurations)
- Oscillators (feedback loops)
- Integrators (high threshold, low decay)
- Coincidence detectors (multiple strong inputs)

### Hierarchical Networks
Templates can build upon templates:
- Layer 1: Individual compartments
- Layer 2: Multi-compartment neurons
- Layer 3: Cortical columns
- Layer 4: Network motifs

### Emergent Computation
Evolution discovers:
- Temporal patterns (through STDP)
- Sparse coding (through anti-Hebbian rules)
- Homeostatic balance (through homeostatic learning)
- Efficient representations (through selection pressure)
