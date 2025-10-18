# Evolve Architecture

## Overview
Evolve is an evolutionary algorithm system with hierarchical composition of operations. The system allows algorithms to evolve from user-defined primitives into complex behaviors through mutation and crossover, with successful patterns being saved as indexed templates for efficient reuse.

## Module Structure

```
src/
├── main.rs              # CLI entry point
├── primitives/          # User-defined base operations
│   └── mod.rs
├── template/            # Algorithm-evolved indexed sequences
│   └── mod.rs
├── dna/                 # Genetic code representation
│   └── mod.rs
├── evolution/           # Mutation and crossover operators
│   ├── mod.rs
│   ├── mutation.rs
│   └── crossover.rs
├── execution/           # Sandboxed execution environment
│   └── mod.rs
└── storage/             # History persistence
    └── mod.rs
```

## DNA Format

### Gene Structure
A **Gene** is the atomic unit of genetic code:
```rust
Gene {
    operation: OperationId,  // References primitive ID or template index
    args: Vec<Argument>,      // Arguments (registers, literals, references)
}
```

### OperationId Encoding
```rust
pub enum OperationId {
    Primitive(u16),      // References user-defined primitive
    Template(u32),       // References algorithm-evolved template by index
}
```

### DNA Strand
A **DNA strand** is a linear sequence of genes:
```rust
DNA {
    genes: Vec<Gene>,
    fitness: Option<f64>,
    generation: u32,
}
```

### Argument Types
- `Register(u8)`: References a register/variable (e.g., R0, R1, ...)
- `Literal(i64)`: Constant value

### Example DNA Encoding
```
User-defined primitives: [READ, ADD, WRITE]

Generation 0 - DNA using only primitives:
[
  Gene { op: Primitive(0), args: [Register(0)] },              # READ -> R0
  Gene { op: Primitive(1), args: [Register(0), Register(0), Register(1)] }, # ADD R0+R0 -> R1
  Gene { op: Primitive(2), args: [Register(1)] },              # WRITE R1
]

If this sequence proves fit, it becomes Template(0): "double_value"

Generation 10 - DNA using primitives and templates:
[
  Gene { op: Template(0), args: [Register(0)] },  # Entire "double_value" sequence in one gene!
  Gene { op: Primitive(1), args: [Register(0), Literal(5), Register(1)] }, # ADD R0+5 -> R1
  Gene { op: Primitive(2), args: [Register(1)] },
]
```

This demonstrates the **configuration space compression**: What took 3 genes now takes 1 gene, allowing evolution to explore higher-level compositions.

## Primitives Module

### Interface
Primitives are **user-defined** operations that form the base vocabulary:
```rust
pub trait Primitive {
    fn execute(&self, args: &[Argument], context: &mut ExecutionContext) -> Result<(), ExecutionError>;
    fn arg_count(&self) -> usize;
    fn name(&self) -> &str;
}

pub struct PrimitiveRegistry {
    primitives: Vec<Box<dyn Primitive>>,
}
```

### Example User-Defined Primitives
Users can define any primitives relevant to their problem domain:
- Arithmetic: `ADD`, `SUB`, `MUL`, `DIV`, `MOD`
- Comparison: `EQ`, `LT`, `GT`, `LE`, `GE`
- Stack: `PUSH`, `POP`
- I/O: `READ_INPUT`, `WRITE_OUTPUT`
- Control: `JUMP`, `JUMP_IF`, `NOP`
- Data: `LOAD`, `STORE`, `COPY`

Users can extend with domain-specific primitives (e.g., signal processing, string manipulation, etc.)

## Template Module

### Interface
Templates are **algorithm-evolved** sequences assigned indices for efficient reuse:
```rust
pub struct Template {
    pub index: u32,              // Unique template ID
    pub genes: Vec<Gene>,        // The evolved sequence
    pub fitness_when_saved: f64, // How fit was the DNA that generated this
    pub generation_created: u32, // When was this template created
    pub usage_count: u64,        // How often has this been used
}

pub struct TemplateRegistry {
    templates: Vec<Template>,
}
```

### Template Creation Strategies
When should evolved sequences become templates?
- **Fitness threshold**: Top N% of population
- **Frequency detection**: Common subsequences across population
- **Generational milestones**: Best performers each generation
- **Manual promotion**: User-selected interesting behaviors

### Template as Configuration Space Compression
Templates are crucial for exploring large configuration spaces:
- Without templates: Long DNA = exponential search space
- With templates: Useful patterns get "chunked" into single operations
- Enables hierarchical evolution: Templates can contain other templates
- Similar to: Subroutines in programming, memes in cultural evolution

## Evolution Module

### Architecture
The evolution module is **decoupled** from the DNA representation, operating on abstract gene sequences.

```rust
pub trait EvolutionOperator {
    fn apply(&self, dna: &DNA, template_registry: &TemplateRegistry) -> DNA;
}
```

### Point Mutation
Operates on a **single DNA vector**:
- `GeneInsertion`: Add random gene at random position (from primitives OR templates)
- `GeneDeletion`: Remove gene at random position
- `GeneSubstitution`: Replace gene with random alternative (primitive or template)
- `ArgumentMutation`: Modify arguments of existing gene
- `TemplateExpansion`: Expand template back to its constituent genes (for exploration)

```rust
pub struct PointMutator {
    insertion_rate: f64,
    deletion_rate: f64,
    substitution_rate: f64,
    argument_rate: f64,
    expansion_rate: f64,        // Rare: unpack templates for variation
    template_usage_bias: f64,   // Probability of using template vs primitive
}
```

### Splicing/Crossover
Operates on **two or more DNA vectors**:
- `SinglePointCrossover`: Cut both strands at same position, swap tails
- `TwoPointCrossover`: Extract segment from one, insert into other
- `UniformCrossover`: Randomly select genes from multiple parents
- `TemplateAwareCrossover`: Prefer cutting at template boundaries

```rust
pub struct Crossover {
    mode: CrossoverMode,
}

pub enum CrossoverMode {
    SinglePoint,
    TwoPoint,
    Uniform { parent_count: usize },
    TemplateAware,  // Respects template boundaries
}
```

### Template-Aware Evolution
Key insight: Templates represent functional units, so evolution should respect them:
- Mutations at template boundaries are less disruptive
- Crossover within templates may break useful patterns
- Template usage bias can be adjusted over generations

### Future Evolution Modes (Extensibility)
The `EvolutionOperator` trait allows for future additions:
- **Pinching**: Create loops or recursive structures
- **Folding**: Conditional execution based on internal state
- **Inversion**: Reverse gene subsequences
- **Duplication**: Copy and repeat gene segments
- **Transposition**: Move gene segments to different locations

## Execution Module

### Sandboxed Execution
```rust
pub struct ExecutionContext {
    registers: Vec<i64>,        // Register file
    stack: Vec<i64>,            // Data stack
    input: Vec<i64>,            // Input data
    output: Vec<i64>,           // Output data
    instruction_count: usize,   // For limits
    max_instructions: usize,    // Safety limit
    primitive_registry: &PrimitiveRegistry,
    template_registry: &TemplateRegistry,
}
```

### Template Execution
When executing a gene with `OperationId::Template(idx)`:
1. Look up template by index
2. Execute each gene in the template sequence
3. Track nested execution depth (prevent infinite recursion)
4. Count all expanded instructions toward safety limits

### Safety Constraints
- Max instruction count (prevent infinite loops, including template expansion)
- Max template recursion depth (prevent stack overflow)
- Max stack depth
- Max output size
- Execution timeout
- No file system access (future: isolated temp directory)

### Interface
```rust
pub fn execute(
    dna: &DNA,
    input: Vec<i64>,
    primitive_registry: &PrimitiveRegistry,
    template_registry: &TemplateRegistry,
) -> Result<ExecutionResult, ExecutionError> {
    // Creates sandboxed context
    // Executes genes sequentially, expanding templates as needed
    // Returns output or error
}
```

## Storage Module

### History Tracking
Store evolved algorithms and templates for analysis:
```rust
pub struct EvolutionHistory {
    generation: u32,
    population: Vec<DNA>,
    best_fitness: f64,
    average_fitness: f64,
    templates_created: Vec<u32>, // Template indices created this generation
}
```

Initial implementation: JSON file storage. Future: SQLite for querying.

## Fitness Evaluation

Fitness is domain-specific. Initial implementation provides framework:
```rust
pub trait FitnessFunction {
    fn evaluate(&self, output: &[i64], expected: &[i64]) -> f64;
}
```

Example: Mean squared error, exact match, pattern detection, etc.

## Design Principles

1. **User-Defined Primitives**: Users define the base operations relevant to their domain
2. **Algorithm-Evolved Templates**: Successful patterns automatically become reusable indexed operations
3. **Configuration Space Compression**: Templates make large search spaces tractable through hierarchical composition
4. **Separation of Concerns**: Evolution, execution, and storage are independent
5. **Extensibility**: Trait-based design allows new primitives, evolution operators, fitness functions
6. **Type Safety**: Strong typing prevents invalid DNA construction
7. **Testability**: Each module can be tested independently
8. **Safety First**: Sandboxing prevents runaway execution, including template recursion
