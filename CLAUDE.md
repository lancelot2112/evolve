# CLAUDE.md

This file provides guidance to Claude Code (claude.ai/code) when working with code in this repository.

## Project Overview

Evolve is an evolutionary algorithm system that exposes simple primitives to evolutionary pressure. The goal is open-ended evolution where algorithms can:
1. Select from primitives (e.g., `z=add(x,y)`, `push(x,fifo)`, `z=pop(fifo)`)
2. Combine primitives into "templates" that themselves become selectable like primitives
3. Express genetic code as selections of primitives/templates for evolutionary processes
4. Operate on input data and produce output in a sandboxed environment

## Technology Stack

- **Language**: Rust (edition 2024)
- **Storage**: SQL or JSON file for evolution history
- **Interface**: CLI for exploring evolved algorithms and running simulations

## Development Commands

### Build and Run
```bash
cargo build           # Build the project
cargo build --release # Production build
```

### CLI Commands

The program provides a comprehensive CLI for evolving and analyzing DNA:

```bash
# Show all available commands
cargo run -- --help

# Run evolution
cargo run -- run [OPTIONS] [PROBLEM]
  -g, --generations <N>     Number of generations (default: 50)
  -p, --population <N>      Population size (default: 100)
  -o, --output <FILE>       Output file (default: evolution_history.json)
  -v, --verbose             Show generation-by-generation progress

# Inspect a specific generation
cargo run -- inspect <GENERATION> [OPTIONS]
  -t, --top <N>             Show top N individuals (default: 10)
  -f, --file <FILE>         History file to load

# Show detailed DNA information
cargo run -- show-dna <DNA_ID> [OPTIONS]
  -f, --file <FILE>         History file to load

# Replay DNA with custom input
cargo run -- replay <DNA_ID> [OPTIONS]
  -i, --input <VALUES>      Comma-separated input values
  -f, --file <FILE>         History file to load

# Show evolution statistics
cargo run -- stats [OPTIONS]
  -f, --file <FILE>         History file to load

# Show templates (note: not fully implemented)
cargo run -- show-templates [OPTIONS]
```

### Example Workflow

**Note:** Options must come *after* the subcommand, not before. For example:
- ✓ `evolve run -g 100` (correct)
- ✗ `evolve -g 100 run` (wrong - options go after subcommand)

```bash
# Run evolution for 100 generations, population of 200
cargo run -- run -g 100 -p 200 -v

# Check statistics
cargo run -- stats

# Inspect generation 50
cargo run -- inspect 50 --top 5

# Look at a specific DNA (e.g., ID 2500)
cargo run -- show-dna 2500

# Test it with custom input
cargo run -- replay 2500 --input 7
```

### Testing
```bash
cargo test           # Run all tests
cargo test <name>    # Run specific test
cargo test -- --nocapture # Show println! output
```

### Code Quality
```bash
cargo check          # Fast compile check
cargo clippy         # Linting
cargo fmt            # Format code
```

## Important: Rust Edition 2024

This project uses **Rust edition 2024**. Note that `gen` is a reserved keyword in this edition:
- Use `r#gen()` when calling random generator methods: `rng.r#gen::<bool>()`
- Avoid using `gen` as a variable name (use `generation_num`, etc. instead)

## Key Architecture Concepts

### Sandboxing and Safety
- Each algorithm must run in a sandbox with isolated environment
- Algorithms can modify their local environment (potentially themselves)
- Enforce limits: max file size, execution time, resource usage
- Security critical: prevent escape from sandbox

### Hierarchical Composition
- **Primitives**: Base operations (arithmetic, stack ops, I/O)
- **Templates**: Saved combinations of primitives/templates
- **Genetic Code**: Sequences of primitives/templates subject to evolution
- Templates promote reuse and enable hierarchical complexity

### Evolution Loop
1. Generate/mutate genetic code (sequences of primitives/templates)
2. Execute in sandboxed environment with input data
3. Evaluate fitness based on output
4. Select, crossover, mutate for next generation
5. Store history for analysis

### Storage Requirements
- Persist evolved algorithms and their performance
- Track template definitions and usage
- Store execution history for analysis via CLI
- Current implementation: JSON file storage (`evolution_history.json`)

## Module Organization

```
src/
├── dna/              # DNA representation (genes, arguments, operation IDs)
├── primitives/       # User-defined operations + execution context
├── template/         # Algorithm-evolved indexed sequences
├── evolution/        # Mutation and crossover operators
│   ├── mutation.rs   # Point mutations (single DNA vector)
│   └── crossover.rs  # Splicing (two+ DNA vectors)
├── execution/        # Sandboxed executor + fitness functions
└── storage/          # JSON persistence for evolution history
```

## DNA Format Implementation

### Gene Structure
```rust
Gene {
    operation: OperationId,  // Primitive(u16) or Template(u32)
    args: Vec<Argument>,     // Register(u8) or Literal(i64)
}
```

### Standard Primitives (IDs 0-9)
- `0: ADD` - Add two values, store in register
- `1: SUB` - Subtract two values
- `2: MUL` - Multiply two values
- `3: DIV` - Divide two values
- `4: PUSH` - Push value onto stack
- `5: POP` - Pop value from stack to register
- `6: READ_INPUT` - Read next input value
- `7: WRITE_OUTPUT` - Write value to output
- `8: COPY` - Copy value to register
- `9: NOP` - No operation

## Evolution Operators

### Point Mutation (src/evolution/mutation.rs)
Operates on **one DNA vector**:
- `insertion_rate`: Add random gene (can be primitive OR template)
- `deletion_rate`: Remove gene
- `substitution_rate`: Replace gene
- `argument_rate`: Mutate gene arguments
- `expansion_rate`: Expand template to constituent genes (rare)
- `template_usage_bias`: Probability of using templates vs primitives

### Crossover (src/evolution/crossover.rs)
Operates on **two or more DNA vectors**:
- `SinglePoint`: Cut at same position, swap tails
- `TwoPoint`: Extract segment, insert into other
- `Uniform`: Randomly select genes from multiple parents
- `TemplateAware`: Prefer cutting at template boundaries

## Adding New Features

### Adding New Primitives
1. Implement the `Primitive` trait in `src/primitives/mod.rs`
2. Register in `PrimitiveRegistry::with_standard_primitives()`
3. Update `MutationConfig::max_primitive_id` to match count

### Adding New Evolution Operators
1. Implement the `EvolutionOperator` trait
2. Add to `evolution/` module
3. Integrate in `EvolutionRunner` in `main.rs`

### Adding New Fitness Functions
1. Implement the `FitnessFunction` trait in `execution/mod.rs`
2. Use in `EvolutionRunner::run()` call

## Current Example

The default `main.rs` evolves a program that doubles its input:
- Test cases: 0→0, 1→2, 5→10, 10→20, -3→-6
- Population: 100 individuals
- Generations: 50
- Mutation rate: 30%
- Crossover rate: 60%
- Templates created: Top 10% fitness each generation
