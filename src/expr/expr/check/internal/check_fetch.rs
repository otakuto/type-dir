use indexmap::IndexMap;

use crate::error::SemanticError;
use crate::yaml::{RuleName, YamlEntry, YamlEntryKind, YamlRule};

use super::pattern_util::named_captures;
use super::walk::child_entries;

/// Reports E027 for every `fetch` entry whose alternatives do not all share the same named
/// capture set.
///
/// All `fetch` alternatives must declare exactly the same named capture names so that the union
/// `.regex.<name>` projection is a total function. An alt that is missing or introduces an extra
/// capture name violates this invariant.
///
/// Also checks that fetch alts are dir/file only (no nested rule/for/group).
pub fn check_fetch(rules: &IndexMap<RuleName, YamlRule>) -> Vec<SemanticError> {
    let mut errors = Vec::new();
    for (rule_name, rule) in rules {
        for entry in &rule.body {
            walk_entry(&rule_name.0, entry, &mut errors);
        }
    }
    errors
}

fn walk_entry(rule: &str, entry: &YamlEntry, errors: &mut Vec<SemanticError>) {
    match &entry.kind {
        YamlEntryKind::Fetch { body } => {
            // The fetch id is now in entry.id.
            let fetch_id = entry
                .id
                .as_ref()
                .expect("Fetch entry always has entry.id (set by From<YamlEntryRepr>)");

            // Collect named captures from each alt.
            let capture_sets: Vec<Vec<String>> = body
                .iter()
                .map(|alt| {
                    let mut caps = Vec::new();
                    if let YamlEntryKind::Dir { pattern, .. }
                    | YamlEntryKind::File { pattern, .. } = &alt.kind
                    {
                        let mut c = named_captures(pattern);
                        caps.append(&mut c);
                    }
                    caps.sort();
                    caps.dedup();
                    caps
                })
                .collect();

            if let Some(expected) = capture_sets.first() {
                for (alt_index, actual) in capture_sets.iter().enumerate().skip(1) {
                    if actual != expected {
                        errors.push(SemanticError::FetchCaptureMismatch {
                            rule: rule.to_string(),
                            fetch_id: fetch_id.0.clone(),
                            alt_index,
                            expected: expected.clone(),
                            actual: actual.clone(),
                        });
                    }
                }
            }

            // Recurse into alts.
            for alt in body {
                walk_entry(rule, alt, errors);
            }
        }

        _ => {
            // All other variants: recurse uniformly into child entries.
            for child in child_entries(entry) {
                walk_entry(rule, child, errors);
            }
        }
    }
}
