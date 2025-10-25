# Lineage Refactoring Progress

## Overview
Refactoring the evolution system to track lineages explicitly with parent relationships and lineage-local template libraries.

## Architecture Changes

### Core Concept
- **Lineages** are matrilineal lines that share a template library
- **DNA** track their lineage_id and parent relationships
- **Template libraries** live at the lineage level (not per-DNA)
- **Parent tracking** distinguishes mitochondrial (template donor) vs genetic (gene donor) parents

### Key Types Added
- `lineage::ParentInfo` - tracks DNA parent with role (Mitochondrial vs Genetic)
- `lineage::ParentRole` - enum for parent roles
- `lineage::Lineage` - matrilineal line with template library and members
- `lineage::LineageRegistry` - manages all lineages

### DNA Changes
- ✅ Added `lineage_id: u64`
- ✅ Added `parents: Vec<ParentInfo>`
- ✅ Removed `template_library` field
- ✅ Updated constructors: `DNA::new(genes, generation, lineage_id)`
- ✅ Added helpers: `mitochondrial_parent()`, `genetic_parents()`, `has_parents()`

## File Status

### ✅ COMPLETED

#### src/lineage/ (New Module)
- ✅ `mod.rs` - Module interface
- ✅ `parent.rs` - ParentInfo and ParentRole types
- ✅ `lineage.rs` - Lineage type with tests
- ✅ `registry.rs` - LineageRegistry with tests

#### src/dna/mod.rs
- ✅ Updated struct fields
- ✅ Updated constructors
- ✅ Added parent accessor methods
- ✅ Updated tests

#### src/evolution/mutation.rs
- ✅ Updated to accept `TemplateRegistry` parameter
- ✅ All helper methods updated: `apply_insertion`, `apply_substitution`, `apply_expansion`
- ✅ `mutate()` signature: `fn mutate(&self, dna: &DNA, template_registry: &TemplateRegistry)`
- ✅ Tests updated
- ⚠️ EvolutionOperator trait impl commented out (needs trait redesign)

#### src/main.rs
- ✅ Added `pub mod lineage;`

### 🔨 IN PROGRESS / NEEDS WORK

#### src/evolution/crossover.rs
**Status**: Tests updated, but core methods need work
**Needed**:
- [ ] Update `single_point_crossover` to use new DNA API and track parents
- [ ] Update `two_point_crossover` to use new DNA API and track parents
- [ ] Update `uniform_crossover` to use new DNA API and track parents
- [ ] Update `template_aware_crossover` to use new DNA API and track parents
- [ ] Each should randomly choose mitochondrial parent and track both parents
- [ ] Remove `template_library` clone logic (now via lineage)

**Signature pattern needed**:
```rust
fn single_point_crossover(&self, parent1: &DNA, parent2: &DNA, lineage_registry: &LineageRegistry) -> DNA {
    // Randomly choose mitochondrial parent
    let mito_parent = if random() { parent1 } else { parent2 };
    let gene_parent = if mito_parent == parent1 { parent2 } else { parent1 };

    // Create child in mitochondrial parent's lineage
    let child = DNA::with_parents(
        genes,
        generation,
        mito_parent.lineage_id,
        vec![
            ParentInfo::mitochondrial(mito_parent.id.unwrap(), mito_parent.lineage_id),
            ParentInfo::genetic(gene_parent.id.unwrap(), gene_parent.lineage_id),
        ]
    );
}
```

#### src/cli.rs - EvolutionRunner
**Status**: CRITICAL - Needs major rewrite
**Needed**:
- [ ] Add `lineage_registry: LineageRegistry` field
- [ ] `initialize_population()`: Create lineages for each DNA in gen 0
- [ ] `evaluate_population()`: Register templates in lineage (not DNA)
- [ ] `create_next_generation()`:
  - Pass lineage registry to mutation/crossover
  - Track parent relationships when creating offspring
  - Update lineage membership when child is created
- [ ] After each generation: Update lineage alive/extinct status
- [ ] Track lineage best fitness

**Template detection** now happens at lineage level:
```rust
// After evaluating DNA
let lineage = lineage_registry.get_mut(dna.lineage_id).unwrap();
let templates_found = detect_templates(&dna.genes);
for template_genes in templates_found {
    lineage.template_library.register(template_genes, dna.fitness.unwrap(), dna.generation);
}
```

#### src/commands/run.rs
**Status**: Needs minor updates
**Needed**:
- [ ] Remove `format_dna` import (unused)
- [ ] Update to handle lineage_registry from EvolutionRunner

#### src/commands/inspect.rs
**Status**: ✅ COMPLETED
**Changes Made**:
- ✅ Updated DNA display to show lineage_id
- ✅ Updated to show parent relationships (Mitochondrial and Genetic)
- ✅ No template_library references (none existed)
- ✅ Added TODO comment for future lineage statistics

#### src/commands/show_dna.rs
**Status**: Unknown - needs investigation
**Needed**:
- [ ] Update to show lineage_id
- [ ] Update to show parents (with roles)
- [ ] Access templates via lineage, not DNA

#### src/commands/show_templates.rs
**Status**: ✅ STUBBED - Compiles, needs future implementation
**Changes Made**:
- ✅ Removed references to DNA.template_library
- ✅ Added comprehensive TODOs for future lineage-based implementation
- ✅ Stubbed out functionality with informative message to users
**Future Work**:
- [ ] Load LineageRegistry from storage
- [ ] Display templates grouped by lineage
- [ ] Show template inheritance patterns

#### src/commands/replay.rs
**Status**: Unknown - needs investigation
**Needed**:
- [ ] Load lineage data to access templates for execution
- [ ] Minor updates for new DNA API

#### src/commands/stats.rs
**Status**: Unknown - needs investigation
**Needed**:
- [ ] Add lineage statistics (count, alive, extinct)
- [ ] Show lineage diversity metrics

#### src/execution/executor.rs
**Status**: Unknown - needs investigation
**Needed**:
- [ ] Check if executor needs access to template library
- [ ] If yes, pass lineage registry to execute()

#### src/storage/mod.rs
**Status**: Needs investigation
**Needed**:
- [ ] Update GenerationRecord to include lineage info
- [ ] Update EvolutionHistory to store LineageRegistry
- [ ] Serialize/deserialize parent relationships
- [ ] Later: Implement compact lineage-based format

### 🧪 TESTS

#### Unit Tests
- ✅ src/lineage/* - All passing
- ✅ src/dna/mod.rs - Updated and passing
- ✅ src/evolution/mutation.rs - Updated and passing
- 🔨 src/evolution/crossover.rs - Updated but may fail runtime

#### Integration Tests
- ❌ tests/template_system_test.rs - Needs complete rewrite for lineage system
- ❌ Other test files - Need investigation

## Compilation Errors

Current error count: ~40 errors

**Categories**:
1. ✅ DNA constructor signature mismatches (partially fixed)
2. 🔨 Crossover using old DNA::new() signatures
3. 🔨 template_library field access (DNA no longer has this)
4. ⚠️ EvolutionOperator trait needs redesign

## Next Steps

### Phase 1: Get Code Compiling (Independent files - can parallelize)
1. Fix src/evolution/crossover.rs
2. Fix src/commands/*.rs files
3. Fix src/execution/executor.rs

### Phase 2: Core Integration (Sequential - complex)
1. Refactor EvolutionRunner in src/cli.rs
2. Update storage module
3. Wire everything together

### Phase 3: Testing & Refinement
1. Fix all test files
2. End-to-end manual testing
3. Performance validation

### Phase 4: Compact Format (Future)
1. Implement base58 encoding
2. Implement lineage-based compact format writer
3. Implement compact format reader
4. Migration utility from JSON

## Notes

- Template detection moved from DNA level to Lineage level
- EvolutionOperator trait may need parameter for template_registry
- Consider adding lineage "novelty injection" for later generations
- Compact format design in docs/LINEAGE_DESIGN.md
