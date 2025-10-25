//! Show DNA Command
//!
//! Displays detailed information about a specific DNA sequence including its genes.

use crate::dna::{Argument, DNA, OperationId};
use crate::primitives::PrimitiveRegistry;
use crate::storage::EvolutionHistory;

pub fn cmd_show_dna(id: u64, file: String) {
    let history = match EvolutionHistory::load_from_file_auto(&file) {
        Ok(h) => h,
        Err(e) => {
            eprintln!("Failed to load history from {}: {}", file, e);
            std::process::exit(1);
        }
    };

    let dna = match history.get_dna(id) {
        Some(d) => d,
        None => {
            eprintln!("DNA with ID {} not found", id);
            std::process::exit(1);
        }
    };

    let primitives = PrimitiveRegistry::with_standard_primitives();

    println!("{}", format_dna_lineage(dna, &primitives));
}

/// Format DNA for display with lineage information
fn format_dna_lineage(dna: &DNA, primitives: &PrimitiveRegistry) -> String {
    let mut output = String::new();

    if let Some(id) = dna.id {
        output.push_str(&format!("DNA ID: {}\n", id));
    }
    output.push_str(&format!("Generation: {}\n", dna.generation));
    output.push_str(&format!("Lineage ID: {}\n", dna.lineage_id));
    output.push_str(&format!("Fitness: {:.4}\n", dna.fitness.unwrap_or(0.0)));
    output.push_str(&format!("Length: {} genes\n", dna.len()));

    // Parent information
    if dna.has_parents() {
        output.push_str("\nParents:\n");
        if let Some(mito) = dna.mitochondrial_parent() {
            output.push_str(&format!(
                "  Mitochondrial: DNA #{} (Lineage {})\n",
                mito.dna_id, mito.lineage_id
            ));
        }
        let genetic_parents = dna.genetic_parents();
        if !genetic_parents.is_empty() {
            output.push_str("  Genetic:\n");
            for parent in genetic_parents {
                output.push_str(&format!(
                    "    DNA #{} (Lineage {})\n",
                    parent.dna_id, parent.lineage_id
                ));
            }
        }
    } else {
        output.push_str("\nParents: None (Generation 0 progenitor)\n");
    }

    // TODO: Template library access requires lineage registry
    // For now, we skip showing templates
    output.push_str("\nTemplates: [TODO: Access via lineage registry]\n");

    output.push_str("\nGenes:\n");
    for (i, gene) in dna.genes.iter().enumerate() {
        let op_name = match gene.operation {
            OperationId::Primitive(id) => primitives
                .get(id)
                .map(|p| p.name().to_string())
                .unwrap_or_else(|| format!("PRIM_{}", id)),
            OperationId::Template(hash) => {
                format!("TEMPLATE_{:x}", hash)
            }
        };

        let args_str: Vec<String> = gene
            .args
            .iter()
            .map(|arg| match arg {
                Argument::Register(r) => format!("R{}", r),
                Argument::Literal(l) => l.to_string(),
            })
            .collect();

        output.push_str(&format!("  {}: {} {}\n", i, op_name, args_str.join(" ")));
    }

    output
}
