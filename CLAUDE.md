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
cargo build          # Build the project
cargo run            # Run the main binary
cargo build --release # Production build
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
- Format TBD: SQL for relational queries vs JSON for simplicity
