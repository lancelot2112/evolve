# Lineage Tracking Design

## Core Concepts

### Lineage
A **matrilineal line** of DNA that share the same template library through inheritance.

- Lineages start in **generation 0** (one per initial DNA)
- Lineages continue through **mitochondrial inheritance** (template library)
- Lineages go **extinct** when no living descendants remain
- Lineages track their **best fitness ever achieved**

### Progenitor
The **first DNA** in a lineage (always from generation 0).
- Stored as `members[0]` (chronologically ordered)
- Helper method: `lineage.progenitor()` returns `members[0]`

### Members
All DNA IDs that belong to this lineage, in chronological order.
- `members[0]` = progenitor
- `members[1..]` = all descendants through mitochondrial inheritance

## Type Definitions

```rust
/// A matrilineal lineage
pub struct Lineage {
    /// Unique lineage ID (same as progenitor DNA ID)
    pub id: u64,

    /// Generation this lineage was founded
    pub origin_generation: u32,

    /// Best fitness ever achieved by any member
    pub best_fitness: f64,

    /// Current alive status
    pub alive: bool,

    /// Template library shared by all members (grows over time)
    pub template_library: TemplateRegistry,

    /// All DNA IDs in this lineage (chronological order)
    /// members[0] is always the progenitor
    pub members: Vec<u64>,
}

impl Lineage {
    /// Get the progenitor DNA ID
    pub fn progenitor(&self) -> Option<u64> {
        self.members.first().copied()
    }

    /// Add a new member to this lineage
    pub fn add_member(&mut self, dna_id: u64) {
        self.members.push(dna_id);
    }

    /// Update best fitness if new fitness is higher
    pub fn update_best_fitness(&mut self, fitness: f64) {
        if fitness > self.best_fitness {
            self.best_fitness = fitness;
        }
    }
}

/// Parent information for DNA
pub struct ParentInfo {
    /// Parent DNA ID
    pub dna_id: u64,

    /// Parent's lineage ID
    pub lineage_id: u64,

    /// Role this parent played
    pub role: ParentRole,
}

pub enum ParentRole {
    /// Provided both genes AND template library (mitochondrial parent)
    /// Child joins this parent's lineage
    Mitochondrial,

    /// Provided genes only (genetic contributor)
    /// Child does NOT join this parent's lineage
    Genetic,
}

/// Updated DNA structure
pub struct DNA {
    /// Unique DNA ID
    pub id: Option<u64>,

    /// Lineage this DNA belongs to
    pub lineage_id: u64,

    /// Parent(s) that created this DNA
    /// - Mutation: 1 parent (Mitochondrial role)
    /// - Crossover: 2+ parents (1 Mitochondrial, others Genetic)
    pub parents: Vec<ParentInfo>,

    /// Genetic code
    pub genes: Vec<Gene>,

    /// Fitness score
    pub fitness: Option<f64>,

    /// Generation this DNA was born
    pub generation: u32,

    /// NOTE: template_library is REMOVED from DNA
    /// Instead, accessed via lineage: lineages[dna.lineage_id].template_library
}
```

## Evolution Operator Behavior

### Mutation (1 parent → 1 child)
```rust
let child = mutate(parent);
child.lineage_id = parent.lineage_id;  // Same lineage
child.parents = vec![ParentInfo {
    dna_id: parent.id,
    lineage_id: parent.lineage_id,
    role: ParentRole::Mitochondrial,  // Parent provides genes + templates
}];
```

### Crossover (2+ parents → 1 child)
```rust
let child = crossover(parent1, parent2);

// Randomly choose mitochondrial parent
let (mito_parent, gene_parent) = if random_bool() {
    (parent1, parent2)
} else {
    (parent2, parent1)
};

child.lineage_id = mito_parent.lineage_id;  // Join mitochondrial parent's lineage

child.parents = vec![
    ParentInfo {
        dna_id: mito_parent.id,
        lineage_id: mito_parent.lineage_id,
        role: ParentRole::Mitochondrial,  // Provides genes + templates
    },
    ParentInfo {
        dna_id: gene_parent.id,
        lineage_id: gene_parent.lineage_id,
        role: ParentRole::Genetic,  // Provides genes only
    },
];
```

## Storage Structure

### Compact Format
```
PRIMITIVES
0:ADD
1:SUB
...

FITNESS
Double the input: 0→0, 1→2, 5→10

SETTINGS
population=100
mutation_rate=0.3

LINEAGE 00000000 origin=0 best=0.95 alive=true
TEMPLATES
a3f2e1d0c9b8a7f6=<base58_genes>;fitness=0.85;gen=1
b9c8d7e6f5a4b3c2=<base58_genes>;fitness=0.72;gen=2
DNA
GEN0 00000000=<base58>;0.00;parents=-
GEN1 00000064=<base58>;0.50;parents=00000000:M
GEN2 000000C8=<base58>;0.75;parents=00000064:M,00000042:G

LINEAGE 00000001 origin=0 best=0.88 alive=false
TEMPLATES
a3f2e1d0c9b8a7f6=<base58_genes>;fitness=0.85;gen=1
DNA
GEN0 00000001=<base58>;0.00;parents=-
GEN1 00000065=<base58>;0.42;parents=00000001:M
GEN2 (extinct - no descendants)

STATS
GEN0 lineages=100 alive=100 best=0.00 avg=0.00
GEN1 lineages=100 alive=95 extinct=5 best=0.85 avg=0.42
GEN2 lineages=100 alive=88 extinct=12 best=0.95 avg=0.68
```

Where:
- `parents=-` = no parents (generation 0)
- `parents=00000064:M` = single mitochondrial parent (mutation)
- `parents=00000064:M,00000042:G` = mitochondrial + genetic parents (crossover)

## Key Questions Resolved

**Q: What is the progenitor?**
A: The first DNA in generation 0 that founded the lineage. Stored as `members[0]`.

**Q: Do we need a separate `progenitor_id` field?**
A: No. `members[0]` is always the progenitor. Use helper method `lineage.progenitor()`.

**Q: Do lineages have parents?**
A: No. Lineages are independent matrilineal lines. Individual DNA have parents (possibly from different lineages), but lineages themselves don't have parent lineages.

**Q: When are new lineages created?**
A: Only in generation 0. After that, lineages continue through mitochondrial inheritance or go extinct.

**Q: Can a lineage split into two lineages?**
A: No. Children always join exactly one lineage (the mitochondrial parent's lineage). The genetic parent from another lineage contributes genes but the child doesn't join that lineage.
