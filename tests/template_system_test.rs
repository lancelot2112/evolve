//! Integration test for the template system
//!
//! This test verifies that:
//! 1. Templates can be detected from DNA with TEMPLATE_START/END markers
//! 2. Templates are registered in DNA's local template_library
//! 3. Templates are inherited through crossover (mitochondrial inheritance)
//! 4. Mutation can use templates from DNA's local library

use evolve::dna::{Argument, DNA, Gene};
use evolve::evolution::{Crossover, MutationConfig, PointMutator};
use evolve::primitives::PrimitiveRegistry;

#[test]
fn test_template_detection_and_registration() {
    // Create DNA with TEMPLATE_START, some genes, TEMPLATE_END
    let mut dna = DNA::empty(0);
    dna.set_fitness(0.9); // Must have fitness to register templates

    // Add: TEMPLATE_START, ADD, TEMPLATE_END
    dna.push_gene(Gene::primitive(10, vec![])); // TEMPLATE_START
    dna.push_gene(Gene::primitive(
        0,
        vec![
            Argument::Register(0),
            Argument::Register(0),
            Argument::Register(0),
        ],
    )); // ADD R0, R0 -> R0
    dna.push_gene(Gene::primitive(11, vec![])); // TEMPLATE_END

    // Verify template library is empty initially
    assert_eq!(dna.template_library.count(), 0);

    // Detect and register templates
    let count = dna.detect_and_register_templates();

    // Verify one template was registered
    assert_eq!(count, 1);
    assert_eq!(dna.template_library.count(), 1);

    // Verify the template contains the right genes (just the ADD, not the markers)
    let templates = dna.template_library.all();
    assert_eq!(templates.len(), 1);
    assert_eq!(templates[0].genes.len(), 1); // Only the ADD gene
}

#[test]
fn test_template_inheritance_through_crossover() {
    // Create parent 1 with a template
    let mut parent1 = DNA::empty(0);
    parent1.set_fitness(0.9);

    parent1.push_gene(Gene::primitive(10, vec![])); // TEMPLATE_START
    parent1.push_gene(Gene::primitive(
        0,
        vec![
            Argument::Literal(2),
            Argument::Literal(3),
            Argument::Register(0),
        ],
    )); // ADD
    parent1.push_gene(Gene::primitive(11, vec![])); // TEMPLATE_END

    parent1.detect_and_register_templates();
    assert_eq!(parent1.template_library.count(), 1);

    // Create parent 2 with a different template
    let mut parent2 = DNA::empty(0);
    parent2.set_fitness(0.8);

    parent2.push_gene(Gene::primitive(10, vec![])); // TEMPLATE_START
    parent2.push_gene(Gene::primitive(
        1,
        vec![
            Argument::Register(1),
            Argument::Register(2),
            Argument::Register(3),
        ],
    )); // SUB
    parent2.push_gene(Gene::primitive(11, vec![])); // TEMPLATE_END

    parent2.detect_and_register_templates();
    assert_eq!(parent2.template_library.count(), 1);

    // Perform crossover
    let crossover = Crossover::single_point();
    let child = crossover.cross(&parent1, &parent2);

    // Child should inherit template library from exactly one parent (mitochondrial inheritance)
    // It should have exactly 1 template (from one parent, not both)
    assert_eq!(child.template_library.count(), 1);
}

#[test]
fn test_mutation_uses_local_templates() {
    // Create DNA with a template in its library
    let mut dna = DNA::empty(0);
    dna.set_fitness(0.9);

    dna.push_gene(Gene::primitive(10, vec![])); // TEMPLATE_START
    dna.push_gene(Gene::primitive(
        0,
        vec![
            Argument::Register(0),
            Argument::Register(0),
            Argument::Register(0),
        ],
    )); // ADD
    dna.push_gene(Gene::primitive(11, vec![])); // TEMPLATE_END

    dna.detect_and_register_templates();
    assert_eq!(dna.template_library.count(), 1);

    // Create mutator with high template usage bias
    let primitive_registry = PrimitiveRegistry::with_standard_primitives();
    let mut mutation_config = MutationConfig::default();
    mutation_config.max_primitive_id = (primitive_registry.count() - 1) as u16;
    mutation_config.template_usage_bias = 1.0; // Always prefer templates
    mutation_config.insertion_rate = 1.0; // Always insert

    let mutator = PointMutator::new(mutation_config);

    // Mutate DNA - it should be able to access templates from its own library
    let mut mutated = dna.clone();
    for _ in 0..10 {
        mutated = mutator.mutate(&mutated);
        // Should not panic or error accessing template_library
    }

    // Verify mutation worked (DNA length should have increased due to insertions)
    assert!(mutated.len() > dna.len());
}

#[test]
fn test_empty_template_not_registered() {
    // Create DNA with empty template (TEMPLATE_START immediately followed by TEMPLATE_END)
    let mut dna = DNA::empty(0);
    dna.set_fitness(0.9);

    dna.push_gene(Gene::primitive(10, vec![])); // TEMPLATE_START
    dna.push_gene(Gene::primitive(11, vec![])); // TEMPLATE_END (no genes in between)

    // Detect and register templates
    let count = dna.detect_and_register_templates();

    // Verify no templates were registered (empty templates should be ignored)
    assert_eq!(count, 0);
    assert_eq!(dna.template_library.count(), 0);
}

#[test]
fn test_multiple_templates_in_same_dna() {
    // Create DNA with multiple template regions
    let mut dna = DNA::empty(0);
    dna.set_fitness(0.95);

    // First template: ADD
    dna.push_gene(Gene::primitive(10, vec![])); // TEMPLATE_START
    dna.push_gene(Gene::primitive(
        0,
        vec![
            Argument::Register(0),
            Argument::Register(0),
            Argument::Register(0),
        ],
    )); // ADD
    dna.push_gene(Gene::primitive(11, vec![])); // TEMPLATE_END

    // Some non-template genes
    dna.push_gene(Gene::primitive(9, vec![])); // NOP

    // Second template: SUB
    dna.push_gene(Gene::primitive(10, vec![])); // TEMPLATE_START
    dna.push_gene(Gene::primitive(
        1,
        vec![
            Argument::Register(1),
            Argument::Register(2),
            Argument::Register(3),
        ],
    )); // SUB
    dna.push_gene(Gene::primitive(11, vec![])); // TEMPLATE_END

    // Detect and register templates
    let count = dna.detect_and_register_templates();

    // Verify two templates were registered
    assert_eq!(count, 2);
    assert_eq!(dna.template_library.count(), 2);
}
