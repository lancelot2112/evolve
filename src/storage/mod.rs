/// Storage module: Persistence for evolution history
///
/// This module handles saving and loading evolution data for later analysis.
use crate::dna::DNA;
use serde::{Deserialize, Serialize};
use std::fs::{File, OpenOptions};
use std::io::{BufRead, BufReader, BufWriter, Write};
use std::path::Path;

/// History record for a single generation
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GenerationRecord {
    /// Generation number
    pub generation: u32,
    /// All DNA in this generation's population
    pub population: Vec<DNA>,
    /// Best fitness in this generation
    pub best_fitness: f64,
    /// Average fitness in this generation
    pub average_fitness: f64,
    /// Template hashes created this generation
    pub templates_created: Vec<u64>,
}

impl GenerationRecord {
    pub fn new(generation: u32, population: Vec<DNA>, templates_created: Vec<u64>) -> Self {
        let best_fitness = population
            .iter()
            .filter_map(|dna| dna.fitness)
            .max_by(|a, b| a.partial_cmp(b).unwrap_or(std::cmp::Ordering::Equal))
            .unwrap_or(0.0);

        let fitness_sum: f64 = population.iter().filter_map(|dna| dna.fitness).sum();
        let fitness_count = population
            .iter()
            .filter(|dna| dna.fitness.is_some())
            .count();
        let average_fitness = if fitness_count > 0 {
            fitness_sum / fitness_count as f64
        } else {
            0.0
        };

        Self {
            generation,
            population,
            best_fitness,
            average_fitness,
            templates_created,
        }
    }
}

/// Complete evolution history
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EvolutionHistory {
    /// Records for each generation
    pub records: Vec<GenerationRecord>,
    /// Next DNA ID to assign
    next_dna_id: u64,
}

impl EvolutionHistory {
    pub fn new() -> Self {
        Self {
            records: Vec::new(),
            next_dna_id: 0,
        }
    }

    /// Add a generation record and assign IDs to DNA
    pub fn add_generation(&mut self, mut record: GenerationRecord) {
        // Assign IDs to all DNA in the population
        for dna in &mut record.population {
            if dna.id.is_none() {
                dna.id = Some(self.next_dna_id);
                self.next_dna_id += 1;
            }
        }
        self.records.push(record);
    }

    /// Get DNA by ID
    pub fn get_dna(&self, id: u64) -> Option<&DNA> {
        self.records
            .iter()
            .flat_map(|r| &r.population)
            .find(|dna| dna.id == Some(id))
    }

    /// Get the latest generation number
    pub fn latest_generation(&self) -> Option<u32> {
        self.records.last().map(|r| r.generation)
    }

    /// Get a specific generation record
    pub fn get_generation(&self, generation: u32) -> Option<&GenerationRecord> {
        self.records.iter().find(|r| r.generation == generation)
    }

    /// Save to JSON file
    pub fn save_to_file<P: AsRef<Path>>(&self, path: P) -> Result<(), std::io::Error> {
        let file = File::create(path)?;
        let writer = BufWriter::new(file);
        serde_json::to_writer_pretty(writer, self)?;
        Ok(())
    }

    /// Load from JSON file
    pub fn load_from_file<P: AsRef<Path>>(path: P) -> Result<Self, std::io::Error> {
        let file = File::open(path)?;
        let reader = BufReader::new(file);
        let history = serde_json::from_reader(reader)?;
        Ok(history)
    }

    /// Append a single generation to a JSONL file (one JSON object per line)
    /// This allows incremental saves without loading the entire history into memory
    pub fn append_generation_to_file<P: AsRef<Path>>(
        &mut self,
        record: GenerationRecord,
        path: P,
    ) -> Result<(), std::io::Error> {
        // Assign IDs to DNA in the record
        let mut record_with_ids = record;
        for dna in &mut record_with_ids.population {
            if dna.id.is_none() {
                dna.id = Some(self.next_dna_id);
                self.next_dna_id += 1;
            }
        }

        // Open file in append mode (creates if doesn't exist)
        let file = OpenOptions::new().create(true).append(true).open(path)?;

        let mut writer = BufWriter::new(file);

        // Write the record as a single JSON line
        serde_json::to_writer(&mut writer, &record_with_ids)?;
        writer.write_all(b"\n")?;
        writer.flush()?;

        // Add to in-memory history as well
        self.records.push(record_with_ids);

        Ok(())
    }

    /// Load from JSONL file (one JSON object per line)
    pub fn load_from_jsonl_file<P: AsRef<Path>>(path: P) -> Result<Self, std::io::Error> {
        let file = File::open(path)?;
        let reader = BufReader::new(file);

        let mut history = Self::new();
        let mut max_dna_id = 0u64;

        for line in reader.lines() {
            let line = line?;
            if line.trim().is_empty() {
                continue; // Skip empty lines
            }

            let record: GenerationRecord = serde_json::from_str(&line)?;

            // Track the maximum DNA ID
            for dna in &record.population {
                if let Some(id) = dna.id {
                    max_dna_id = max_dna_id.max(id);
                }
            }

            history.records.push(record);
        }

        // Set next_dna_id to one past the maximum
        history.next_dna_id = max_dna_id + 1;

        Ok(history)
    }

    /// Clear in-memory records to free RAM (keeps next_dna_id for continuity)
    /// Use this after saving to disk when you don't need old generations in memory
    pub fn clear_old_records(&mut self, keep_last_n: usize) {
        if self.records.len() > keep_last_n {
            let drain_count = self.records.len() - keep_last_n;
            self.records.drain(0..drain_count);
        }
    }

    /// Get the current next_dna_id (useful for tracking across saves)
    pub fn get_next_dna_id(&self) -> u64 {
        self.next_dna_id
    }

    /// Load from file (auto-detects JSON vs JSONL format based on extension)
    pub fn load_from_file_auto<P: AsRef<Path>>(path: P) -> Result<Self, std::io::Error> {
        let path_str = path.as_ref().to_string_lossy();
        if path_str.ends_with(".jsonl") {
            Self::load_from_jsonl_file(path)
        } else {
            Self::load_from_file(path)
        }
    }
}

impl Default for EvolutionHistory {
    fn default() -> Self {
        Self::new()
    }
}

/// Helper to find the best DNA in history
impl EvolutionHistory {
    pub fn best_dna(&self) -> Option<&DNA> {
        self.records
            .iter()
            .flat_map(|r| &r.population)
            .max_by(|a, b| {
                a.fitness
                    .unwrap_or(0.0)
                    .partial_cmp(&b.fitness.unwrap_or(0.0))
                    .unwrap_or(std::cmp::Ordering::Equal)
            })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_generation_record() {
        let mut dna1 = DNA::empty(0);
        dna1.set_fitness(0.8);

        let mut dna2 = DNA::empty(0);
        dna2.set_fitness(0.6);

        let record = GenerationRecord::new(0, vec![dna1, dna2], vec![]);

        assert_eq!(record.generation, 0);
        assert_eq!(record.best_fitness, 0.8);
        assert_eq!(record.average_fitness, 0.7);
    }

    #[test]
    fn test_evolution_history() {
        let mut history = EvolutionHistory::new();

        let record1 = GenerationRecord::new(0, vec![], vec![]);
        let record2 = GenerationRecord::new(1, vec![], vec![]);

        history.add_generation(record1);
        history.add_generation(record2);

        assert_eq!(history.latest_generation(), Some(1));
        assert!(history.get_generation(0).is_some());
    }
}
