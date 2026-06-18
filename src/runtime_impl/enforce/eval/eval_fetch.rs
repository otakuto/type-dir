use std::collections::HashSet;

use crate::expr::{ExprEntry, ExprMatcher};
use crate::walk::DirTree;

use super::eval_inner::EvalContext;
use crate::runtime_impl::env::Scope;

/// Computes the observation result of a Fetch entry in a single pass.
///
/// Matches all children (dirs/files) of `tree` against each alt pattern and deduplicates by
/// capture tuple (excluding "0", sorted by (name,value)) to build records.
/// An id-less Fetch is a no-op (returns an empty RecordMap).
/// Does not descend into child dirs (observation only).
pub(super) fn fetch_produced(
    tree: &DirTree,
    entry: &ExprEntry,
    scope: &Scope,
) -> crate::runtime_impl::record_map::RecordMap {
    let mut out = crate::runtime_impl::record_map::RecordMap::new();
    let fetch_id = match &entry.id {
        Some(id) => id.0.clone(),
        None => return out,
    };
    let alts = match &entry.matcher {
        ExprMatcher::Fetch { body: alts } => alts,
        _ => return out,
    };
    // dedup: seen is a set of (name,value) tuple sequences (excluding "0" keys, sorted)
    let mut seen: HashSet<Vec<(String, String)>> = HashSet::new();
    for alt in alts {
        for child_dir in &tree.dirs {
            if let Some(captures) =
                super::super::matcher::entry_matches_name(alt, &child_dir.name, scope, true)
            {
                let key = captures_dedup_key(&captures);
                if seen.insert(key) {
                    out.entry(fetch_id.clone()).or_default().push(
                        crate::runtime_impl::value::Record {
                            fields: captures.into_iter().collect(),
                            children: indexmap::IndexMap::new(),
                            tag: None,
                        },
                    );
                }
            }
        }
        for file_name in &tree.files {
            if let Some(captures) =
                super::super::matcher::entry_matches_name(alt, file_name, scope, false)
            {
                let key = captures_dedup_key(&captures);
                if seen.insert(key) {
                    out.entry(fetch_id.clone()).or_default().push(
                        crate::runtime_impl::value::Record {
                            fields: captures.into_iter().collect(),
                            children: indexmap::IndexMap::new(),
                            tag: None,
                        },
                    );
                }
            }
        }
    }
    out
}

pub(super) fn eval_fetch(
    ctx: &mut EvalContext,
    fetch_id: &str,
    alts: &[ExprEntry],
    work_scope: &mut Scope,
    produced: &mut crate::runtime_impl::record_map::RecordMap,
) {
    let fetch_sets = fetch_with_id(ctx.tree, fetch_id, alts, work_scope);
    for (id, records) in fetch_sets {
        work_scope.bind_env(
            crate::runtime_impl::node_id::NodeKind::Fetch,
            id.clone(),
            records.clone(),
        );
        produced.entry(id).or_default().extend(records);
    }
}

/// Computes the observation result of a Fetch entry with id and alts decomposed.
///
/// An internal helper used when the id is already known, instead of calling `fetch_produced`.
fn fetch_with_id(
    tree: &DirTree,
    fetch_id: &str,
    alts: &[ExprEntry],
    scope: &Scope,
) -> crate::runtime_impl::record_map::RecordMap {
    let mut out = crate::runtime_impl::record_map::RecordMap::new();
    let mut seen: HashSet<Vec<(String, String)>> = HashSet::new();
    for alt in alts {
        for child_dir in &tree.dirs {
            if let Some(captures) =
                super::super::matcher::entry_matches_name(alt, &child_dir.name, scope, true)
            {
                let key = captures_dedup_key(&captures);
                if seen.insert(key) {
                    out.entry(fetch_id.to_string()).or_default().push(
                        crate::runtime_impl::value::Record {
                            fields: captures.into_iter().collect(),
                            children: indexmap::IndexMap::new(),
                            tag: None,
                        },
                    );
                }
            }
        }
        for file_name in &tree.files {
            if let Some(captures) =
                super::super::matcher::entry_matches_name(alt, file_name, scope, false)
            {
                let key = captures_dedup_key(&captures);
                if seen.insert(key) {
                    out.entry(fetch_id.to_string()).or_default().push(
                        crate::runtime_impl::value::Record {
                            fields: captures.into_iter().collect(),
                            children: indexmap::IndexMap::new(),
                            tag: None,
                        },
                    );
                }
            }
        }
    }
    out
}

fn captures_dedup_key(
    captures: &std::collections::HashMap<String, String>,
) -> Vec<(String, String)> {
    let mut k: Vec<(String, String)> = captures
        .iter()
        .filter(|(k, _)| k.as_str() != "0")
        .map(|(k, v)| (k.clone(), v.clone()))
        .collect();
    k.sort_by(|a, b| a.0.cmp(&b.0));
    k
}
