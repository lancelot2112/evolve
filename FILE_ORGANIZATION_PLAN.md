# File Organization Plan for Evolve

## Current State

Many modules are single large files that handle multiple responsibilities. This makes the codebase harder to navigate.

## Proposed Organization

### DNA Module (src/dna/)
**Current:** `mod.rs` (130 lines) - contains everything
**Proposed Split:**
```
src/dna/
├── mod.rs              # Public interface - re-exports only
├── types.rs            # Gene, OperationId, Argument
├── dna.rs              # DNA struct and methods
└── operations.rs       # Gene construction helpers
```

**File Responsibilities:**
- `mod.rs` (10-15 lines): `pub use` statements to expose public API
- `types.rs`: Core type definitions
- `dna.rs`: DNA struct with methods (push_gene, insert_gene, set_fitness, etc.)
- `operations.rs`: Convenience constructors (Gene::primitive, Gene::template)

### Primitives Module (src/primitives/)
**Current:** `mod.rs` (530+ lines) - all primitives in one file
**Proposed Split:**
```
src/primitives/
├── mod.rs              # ExecutionContext, Primitive trait, PrimitiveRegistry, re-exports
├── arithmetic.rs       # Add, Sub, Mul, Div
├── stack.rs            # Push, Pop
├── io.rs               # ReadInput, WriteOutput
├── data.rs             # Copy, Nop
└── markers.rs          # TemplateStart, TemplateEnd
```

**File Responsibilities:**
- `mod.rs` (20-30 lines): Public interface - re-exports traits, types, and registry
- `context.rs` (~80 lines): ExecutionContext and ExecutionError
- `registry.rs` (~60 lines): PrimitiveRegistry implementation
- `trait.rs` (~20 lines): Primitive trait definition
- `arithmetic.rs` (~100 lines): Mathematical operations
- `stack.rs` (~50 lines): Stack manipulations
- `io.rs` (~50 lines): Input/output operations
- `data.rs` (~50 lines): Data movement operations
- `markers.rs` (~40 lines): Template boundary markers

**File Headers Example:**
```rust
//! Arithmetic Primitives
//!
//! This module provides basic mathematical operations (ADD, SUB, MUL, DIV) that operate
//! on two values and store the result in a register. All operations use wrapping arithmetic
//! to prevent panics on overflow.
```

### Template Module (src/template/)
**Current:** `mod.rs` (240+ lines) - everything in one file
**Proposed Split:**
```
src/template/
├── mod.rs              # Template struct, TemplateRegistry, hash_genes, re-exports
├── detection.rs        # detect_templates() - finds START/END sequences
└── strategy.rs         # TemplateCreationStrategy enum and logic
```

**File Responsibilities:**
- `mod.rs` (10-15 lines): Public interface - re-exports only
- `template.rs` (~70 lines): Template struct and TemplateRegistry
- `hashing.rs` (~50 lines): hash_genes() function
- `detection.rs` (~60 lines): detect_templates() - finds START/END sequences
- `strategy.rs` (~60 lines): TemplateCreationStrategy enum and logic

### Evolution Module (src/evolution/)
**Current:** Already well-organized!
```
src/evolution/
├── mod.rs              # EvolutionOperator trait, re-exports
├── mutation.rs         # Point mutations (single DNA vector)
└── crossover.rs        # Splicing (two+ DNA vectors)
```
**Status:** ✓ Good as is. Maybe add file headers.

**Proposed Future Split:**
```
src/evolution/
├── mod.rs
├── mutation/
│   ├── mod.rs          # PointMutator, MutationConfig
│   ├── insertion.rs    # Gene insertion logic
│   ├── deletion.rs     # Gene deletion logic
│   ├── substitution.rs # Gene substitution logic
│   └── expansion.rs    # Template expansion logic
└── crossover/
    ├── mod.rs          # Crossover, CrossoverMode
    ├── single_point.rs
    ├── two_point.rs
    ├── uniform.rs
    └── template_aware.rs
```

### Execution Module (src/execution/)
**Current:** `mod.rs` (340+ lines) - everything in one file
**Proposed Split:**
```
src/execution/
├── mod.rs              # Executor, ExecutionConfig, re-exports
├── result.rs           # ExecutionResult struct
└── fitness.rs          # FitnessFunction trait, ExactMatchFitness, MSEFitness, PartialMatchFitness
```

**File Responsibilities:**
- `mod.rs` (10-15 lines): Public interface - re-exports only
- `executor.rs` (~150 lines): Executor struct, gene execution loop
- `config.rs` (~30 lines): ExecutionConfig
- `result.rs` (~30 lines): ExecutionResult type
- `fitness.rs` (~100 lines): FitnessFunction trait and implementations

### Storage Module (src/storage/)
**Current:** `mod.rs` (160 lines) - single file
**Proposed Split:**
```
src/storage/
├── mod.rs              # EvolutionHistory, re-exports
├── generation.rs       # GenerationRecord
└── serialization.rs    # save_to_file, load_from_file, append logic
```

**File Responsibilities:**
- `mod.rs` (10-15 lines): Public interface - re-exports only
- `history.rs` (~60 lines): EvolutionHistory struct and methods
- `generation.rs` (~50 lines): GenerationRecord struct and methods
- `serialization.rs` (~60 lines): File I/O (save_to_file, load_from_file, append)

### CLI Module (src/cli/)
**Current:** `cli.rs` (285 lines) - runner + formatting
**Proposed Split:**
```
src/cli/
├── mod.rs              # Re-exports
├── runner.rs           # EvolutionRunner implementation
└── formatting.rs       # format_dna() and other display helpers
```

### Main Binary (src/)
**Current:** `main.rs` (345+ lines) - all commands in one file
**Proposed Split:**
```
src/
├── main.rs             # CLI parser, command dispatcher (100 lines)
└── commands/
    ├── mod.rs          # Re-exports
    ├── run.rs          # cmd_evolve()
    ├── inspect.rs      # cmd_inspect()
    ├── show_dna.rs     # cmd_show_dna()
    ├── show_templates.rs  # cmd_show_templates()
    ├── replay.rs       # cmd_replay()
    └── stats.rs        # cmd_stats()
```

**File Responsibilities:**
- `main.rs`: Clap setup and command routing
- Each command file (~50-70 lines): Single command implementation

## File Header Template

Every file should start with:
```rust
//! [Module Name]
//!
//! [One-line summary of responsibility]
//!
//! [2-3 sentence detailed description of what this module does,
//! how it relates to other modules, and any important design decisions]
```

Example:
```rust
//! Template Detection
//!
//! Scans DNA sequences to find genes between TEMPLATE_START and TEMPLATE_END markers.
//!
//! This module provides the `detect_templates()` function which identifies potential
//! templates in evolved DNA. Only non-empty sequences between valid markers are detected.
//! The function returns the start/end indices and hash of each template found.
```

## Benefits of This Organization

1. **Single Responsibility**: Each file has one clear purpose
2. **Easy Navigation**: Find code by category (arithmetic, stack, I/O, etc.)
3. **Testability**: Smaller files are easier to test in isolation
4. **Documentation**: File-level docs explain the "why" and "how"
5. **Maintainability**: Changes are localized to relevant files
6. **Onboarding**: New developers can understand the codebase faster

## Migration Priority

1. **High Priority** (do now):
   - Split `src/primitives/mod.rs` - it's the largest and most complex
   - Split `src/main.rs` commands - would improve CLI code clarity

2. **Medium Priority** (do when refactoring):
   - Split `src/template/mod.rs` - moderate complexity
   - Split `src/execution/mod.rs` - moderate complexity

3. **Low Priority** (nice to have):
   - Split `src/dna/mod.rs` - already pretty small
   - Split `src/storage/mod.rs` - already pretty small
   - Add file headers everywhere

## Next Steps

After current template system work is complete:
1. Create file headers for all existing files
2. Split primitives module (biggest win)
3. Split commands in main.rs
4. Update CLAUDE.md with new file organization
