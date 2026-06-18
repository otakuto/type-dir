use std::collections::HashMap;

use super::candidate::Candidate;

/// Aggregates assignment counts. Used as input for the interval constraint check and the choice realization check.
///
/// - `entry`: entry_index → assignment count c_e (for standalone entries; used in the interval constraint check)
/// - `choice`: group_index → alt_index → assignment count c_a (for group alternatives; used in the choice realization check)
#[derive(Default)]
pub struct AssignmentCounts {
    entry: HashMap<usize, usize>,
    choice: HashMap<usize, HashMap<usize, usize>>,
}

impl AssignmentCounts {
    /// Creates an empty counter.
    pub fn new() -> Self {
        Self::default()
    }

    /// Records an assignment to the given candidate.
    ///
    /// If `group_key` is present, records to the choice counter; otherwise records to the entry counter.
    pub fn record(&mut self, candidate: &Candidate) {
        if let Some(gk) = candidate.group_key {
            *self
                .choice
                .entry(gk.group_index)
                .or_default()
                .entry(gk.alt_index)
                .or_default() += 1;
        } else {
            *self.entry.entry(candidate.entry_index).or_default() += 1;
        }
    }

    /// Returns the assignment count for a standalone entry (0 if no assignments have been recorded).
    pub fn entry_count(&self, entry_index: usize) -> usize {
        self.entry.get(&entry_index).copied().unwrap_or(0)
    }

    /// Returns the assignment count for a group alternative (0 if no assignments have been recorded).
    pub fn alt_count(&self, group_index: usize, alt_index: usize) -> usize {
        self.choice
            .get(&group_index)
            .and_then(|m| m.get(&alt_index))
            .copied()
            .unwrap_or(0)
    }
}
