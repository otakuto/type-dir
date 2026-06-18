#[cfg(test)]
#[path = "cycle_splice_tests/tests.rs"]
mod tests;

use std::collections::HashMap;

use indexmap::IndexMap;

use crate::error::SemanticError;
use crate::yaml::{RuleName, YamlEntry, YamlEntryKind, YamlRule};

/// Detects infinite splice recursion.
///
/// Treats as a compile-time error any cycle consisting solely of splice edges that passes
/// through no dir/file nodes and no `for` guards.
///
/// Definition of a splice edge: a bare rule reference (`- use: rule.X`) at the top level of
/// rule R's entry list. Alternatives of one_of/any_of and match arms are traversed
/// transparently because they "consume no nodes". A `for` is NOT traversed: it acts as a
/// splice-cycle boundary (like a dir/file node), because a splice under `for` is data-driven —
/// it iterates a finite captured set and recurses on a strictly smaller subset, so it converges.
/// Does not descend into `rules:` (inline) under dir/file entries (reset by node consumption).
pub fn check_splice_cycles(rules: &IndexMap<RuleName, YamlRule>) -> Vec<SemanticError> {
    // Build the splice dependency graph (rule name → list of spliced rule names).
    let mut edges: HashMap<String, Vec<String>> = HashMap::new();
    for (name, rule) in rules {
        let mut targets = Vec::new();
        for entry in &rule.body {
            collect_splice_targets(entry, &mut targets);
        }
        edges.insert(name.0.clone(), targets);
    }

    let mut errors = Vec::new();
    let mut state: HashMap<String, VisitState> = HashMap::new();
    let mut path: Vec<String> = Vec::new();
    for name in edges.keys() {
        if !state.contains_key(name) {
            dfs(name, &edges, &mut state, &mut path, &mut errors);
        }
    }
    errors
}

/// DFS visit state.
enum VisitState {
    /// Currently being explored (on the stack).
    InProgress,
    /// Exploration complete.
    Done,
}

/// Collects splice edges (bare rule references) at the top level of an entry list.
///
/// Traverses one_of/any_of and match arms transparently (they consume no nodes).
/// A `for` is NOT traversed: it is a splice-cycle boundary (like a dir/file node), because a
/// splice under `for` is data-driven (it iterates a finite captured set and recurses on a strictly
/// smaller subset), so the recursion converges and must not be reported as InfiniteSplice.
/// Does not descend into inline rules of dir/file entries (reset by node consumption).
fn collect_splice_targets(entry: &YamlEntry, targets: &mut Vec<String>) {
    match &entry.kind {
        // dir/file entries consume a node, so they are not splice edges (and inline is not descended into)
        YamlEntryKind::Dir { .. } | YamlEntryKind::File { .. } => {}
        // group: traverse alternatives
        YamlEntryKind::Choice { body, .. } => {
            for alt in body {
                collect_splice_targets(alt, targets);
            }
        }
        // for: a splice-cycle boundary. The for body's splices are data-driven (finite set, smaller
        // recursion), so they converge and are NOT collected as cycle edges (same as a dir/file node).
        YamlEntryKind::For { .. } => {}
        // match: traverse arm rules (a match arm may contain a splice)
        YamlEntryKind::Match { body, .. } => {
            for arm in body {
                collect_splice_targets(arm, targets);
            }
        }
        // bare rule reference: splice edge
        YamlEntryKind::Use { rule, .. } => {
            targets.push(rule.0.clone());
        }
        YamlEntryKind::Group { body, .. } => {
            for child in body {
                collect_splice_targets(child, targets);
            }
        }
        YamlEntryKind::Fetch { .. } => {}
        // a value binding is not a splice edge and owns no children
        YamlEntryKind::Value { .. } => {}
    }
}

/// Detects cycles consisting only of splice edges using DFS.
fn dfs(
    node: &str,
    edges: &HashMap<String, Vec<String>>,
    state: &mut HashMap<String, VisitState>,
    path: &mut Vec<String>,
    errors: &mut Vec<SemanticError>,
) {
    state.insert(node.to_string(), VisitState::InProgress);
    path.push(node.to_string());

    if let Some(neighbors) = edges.get(node) {
        for neighbor in neighbors {
            match state.get(neighbor.as_str()) {
                None => dfs(neighbor, edges, state, path, errors),
                Some(VisitState::InProgress) => {
                    // Cycle detected: extract the cycle from the position where neighbor first appears on the path
                    let start = path.iter().position(|n| n == neighbor).unwrap_or(0);
                    let cycle = path[start..].to_vec();
                    errors.push(SemanticError::InfiniteSplice { cycle });
                }
                Some(VisitState::Done) => {}
            }
        }
    }

    path.pop();
    state.insert(node.to_string(), VisitState::Done);
}
