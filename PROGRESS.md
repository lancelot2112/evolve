# Evolve Project - Development Progress

## Current Status

**Branch:** `feature/lineage-local-templates`
**Last Updated:** 2025-10-18

## Recent Accomplishments

### ✅ Code Reorganization (Commit: 265782e)
Restructured all major modules into focused, single-responsibility files:

- **Primitives Module** (28 lines, was 549)
  - Split into: context.rs, trait_def.rs, registry.rs, arithmetic.rs, stack.rs, io.rs, data.rs, markers.rs
  - Each file has clear single responsibility with comprehensive documentation

- **Commands Module** (new)
  - Extracted 6 commands from main.rs into separate files: run.rs, inspect.rs, show_dna.rs, show_templates.rs, replay.rs, stats.rs
  - Each command ~50-70 lines

- **Main.rs** (128 lines, was 345)
  - Now just CLI parsing and dispatch

- **Template Module** (20 lines, was 321)
  - Split into: template.rs, hashing.rs, detection.rs, strategy.rs

- **Execution Module** (18 lines, was 351)
  - Split into: executor.rs, config.rs, result.rs, fitness.rs

All mod.rs files are interface-only with re-exports.

### ✅ Mitochondrial Inheritance (Commit: 9f60674)
Implemented lineage-local template library inheritance:
- Children randomly inherit template_library from ONE parent (not both)
- Prevents template library bloat
- Creates lineage-specific evolution patterns
- Applied to all 4 crossover modes (single-point, two-point, uniform, template-aware)

## Current TODO List

### High Priority (Template System Implementation)

- [ ] **Update mutation to work with template_library**
  - Mutation should use DNA's local template_library, not global registry
  - Template selection should use hash-based lookup

- [ ] **Implement template creation from detected sequences**
  - Scan DNA for TEMPLATE_START/TEMPLATE_END markers
  - Extract gene sequences between markers
  - Hash and register in DNA's template_library
  - Integrate into evolution runner

- [ ] **Update CLI/EvolutionRunner for new template system**
  - Remove global template registry
  - Use DNA-local template libraries
  - Update template creation logic to use detect_templates()

- [ ] **Implement incremental saves**
  - Save every N generations (append to file)
  - Avoid loading/saving entire history each generation

- [ ] **Test new template system end-to-end**
  - Run evolution with template detection
  - Verify templates are created and inherited
  - Check for performance improvements
  - Validate no template explosion

### Lower Priority (Future Enhancements)

- [ ] Add file headers to remaining modules (dna, evolution, storage, cli)
- [ ] Add CALL_TEMPLATE primitive (alternative to inline expansion)
- [ ] Save template registry definitions to history
- [ ] Implement template usage tracking

## Technical Context

### Template System Design

**Hash-based Identification:**
- Templates identified by u64 hash (not sequential u32 index)
- Automatic deduplication: same sequence = same hash
- Stored in HashMap<u64, Template>

**Lineage-Local Libraries:**
- Each DNA has `template_library: TemplateRegistry` field
- Not serialized (templates recreated during evolution)
- Inherited from one parent via mitochondrial pattern

**Template Detection:**
- Scan for TEMPLATE_START (ID 10) and TEMPLATE_END (ID 11) markers
- Extract genes between markers (excluding markers themselves)
- Hash and register non-empty sequences

### Key Files

- `src/dna/mod.rs` - DNA struct with template_library field
- `src/template/` - Template types, hashing, detection, strategy
- `src/evolution/crossover.rs` - Implements mitochondrial inheritance
- `src/evolution/mutation.rs` - Needs update for local template_library
- `src/cli.rs` - EvolutionRunner, needs template creation integration
- `src/primitives/markers.rs` - TEMPLATE_START/TEMPLATE_END

### Known Issues to Address

1. **Mutation** currently uses passed-in template_registry, should use dna.template_library
2. **Template creation** not integrated into evolution loop yet
3. **CLI** still uses global template registry pattern
4. **Template persistence** not implemented (templates not saved to history)

## Next Steps

1. Update mutation.rs to use DNA's local template_library
2. Integrate detect_templates() into EvolutionRunner after each generation
3. Update EvolutionRunner to remove global template registry
4. Add incremental save logic to avoid full history rewrites
5. Test complete system with double problem
6. Validate performance and template management

## Build Status

✅ All code compiles with no errors
✅ Tests pass
✅ Changes pushed to remote branch
