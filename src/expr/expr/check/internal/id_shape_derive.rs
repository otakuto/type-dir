use std::collections::HashSet;

use indexmap::IndexMap;

use super::id_shape::{ChildRef, IdShape, NodeKind};
use super::pattern_util::named_captures;
use crate::yaml::{EntryId, RuleName, YamlEntry, YamlEntryKind, YamlRule};

/// Derives the `IdShape` for a rule's public id (Pub(R): the single id visible at the rule top).
///
/// Pub(R) is obtained by `collect_visible_ids` on the rule's body. Returns `None` when the rule
/// is not found or has no public id.
///
/// Thin wrapper that starts a fresh `visited` guard, then delegates to the guarded variant.
pub fn derive_rule_id_shape(
    rule_name: &RuleName,
    rules: &IndexMap<RuleName, YamlRule>,
) -> Option<IdShape> {
    let mut visited = HashSet::new();
    derive_rule_id_shape_guarded(rule_name, rules, &mut visited)
}

/// Guarded variant of `derive_rule_id_shape`.
///
/// `visited` tracks the rule names currently being derived to break recursive splice+id chains
/// (e.g. rule `A` whose splice+id refers back to `A`). A rule already on the derivation stack is
/// treated as non-derivable (`None`). The guard follows DFS stack discipline: the rule is inserted
/// on entry and removed before returning, so sibling derivations are not blocked.
fn derive_rule_id_shape_guarded(
    rule_name: &RuleName,
    rules: &IndexMap<RuleName, YamlRule>,
    visited: &mut HashSet<String>,
) -> Option<IdShape> {
    // Recursive chain detection: a rule already being derived is treated as non-derivable.
    if !visited.insert(rule_name.0.clone()) {
        return None;
    }
    let result = derive_rule_id_shape_inner(rule_name, rules, visited);
    // DFS stack discipline: pop on return so siblings can still derive this rule.
    visited.remove(&rule_name.0);
    result
}

/// Inner body of the guarded derivation (the `visited` insert/remove is handled by the caller).
fn derive_rule_id_shape_inner(
    rule_name: &RuleName,
    rules: &IndexMap<RuleName, YamlRule>,
    visited: &mut HashSet<String>,
) -> Option<IdShape> {
    let rule = rules.get(rule_name)?;
    let mut visible_visited = HashSet::new();
    let mut pub_ids: HashSet<String> = HashSet::new();
    collect_visible_ids(&rule.body, rules, &mut visible_visited, &mut pub_ids);
    if pub_ids.len() != 1 {
        return None; // 0 or >=2 public ids: no unique type
    }
    let pub_id = pub_ids.into_iter().next()?;
    // Find the IdShape for the public id in the id_shapes map (need to build locally)
    let mut id_shape_map = IndexMap::new();
    for entry in &rule.body {
        collect_id_shapes_guarded(entry, rules, visited, &mut id_shape_map);
    }
    id_shape_map.shift_remove(&pub_id)
}

/// Resolves the `IdShape` for a `RuleType { rule, path }` input declaration.
///
/// - `path` empty → delegates to `derive_rule_id_shape(rule)` (the single public id of the rule).
/// - `path` non-empty → starts from the group shape produced by `build_splice_id_group_shape`
///   (which maps each public id of `rule` to its own `ChildRef`), then follows each `EntryId` in
///   `path` by looking it up in the `child_ids` of the current `IdShape`. At each step a
///   `ChildRef::Inline(s)` yields `s` directly; a `ChildRef::RuleRef(r)` is expanded via
///   `derive_rule_id_shape(r)`. Returns `None` when a segment is not found or cannot be resolved.
pub(crate) fn resolve_rule_path_shape(
    rule: &RuleName,
    path: &[EntryId],
    rules: &IndexMap<RuleName, YamlRule>,
) -> Option<IdShape> {
    if path.is_empty() {
        return derive_rule_id_shape(rule, rules);
    }
    // Build the group-wrapper shape that exposes each public id of `rule` as a child.
    let group_shape = build_splice_id_group_shape(rule, rules, &mut HashSet::new());
    // Follow the first path segment through the group's child_ids, then recurse into the tail.
    let (first, tail) = path.split_first()?;
    let first_ref = group_shape.child_ids.get(&first.0)?;
    resolve_path_tail(first_ref, tail, rules)
}

/// Resolves a `ChildRef` to an `IdShape` (one-level expansion).
fn resolve_child_ref(
    child_ref: &ChildRef,
    rules: &IndexMap<RuleName, YamlRule>,
) -> Option<IdShape> {
    match child_ref {
        ChildRef::Inline(s) => Some(s.clone()),
        ChildRef::RuleRef(r) => derive_rule_id_shape(r, rules),
    }
}

/// Follows a path tail starting from a `ChildRef`.
///
/// Resolves `child_ref` to an `IdShape`, then for each remaining segment looks up the next
/// `ChildRef` in the shape's `child_ids`. Returns the final `IdShape` when the tail is exhausted.
fn resolve_path_tail(
    child_ref: &ChildRef,
    tail: &[EntryId],
    rules: &IndexMap<RuleName, YamlRule>,
) -> Option<IdShape> {
    let id_shape = resolve_child_ref(child_ref, rules)?;
    if tail.is_empty() {
        return Some(id_shape);
    }
    let (next, rest) = tail.split_first()?;
    let next_ref = id_shape.child_ids.get(&next.0)?;
    resolve_path_tail(next_ref, rest, rules)
}

/// Builds a map of id → static shape from all rules.
///
/// Exported so that `check_with_shapes` can resolve self-owned id shapes when building
/// the per-rule shape environment.
pub(crate) fn build_id_shapes(rules: &IndexMap<RuleName, YamlRule>) -> IndexMap<String, IdShape> {
    let mut map = IndexMap::new();
    for rule in rules.values() {
        for entry in &rule.body {
            collect_id_shapes_guarded(entry, rules, &mut HashSet::new(), &mut map);
        }
    }
    map
}

/// Traverses entries and records the static shape of id-bearing entries in the map.
///
/// `visited` tracks the rule-derivation stack so that a splice+id entry can resolve its shape from
/// the spliced rule's public id without looping on recursive chains (shared with
/// `derive_rule_id_shape_guarded`).
pub(crate) fn collect_id_shapes_guarded(
    entry: &YamlEntry,
    rules: &IndexMap<RuleName, YamlRule>,
    visited: &mut HashSet<String>,
    map: &mut IndexMap<String, IdShape>,
) {
    if let Some(id) = &entry.id {
        match &entry.kind {
            YamlEntryKind::Use { rule: target, .. } => {
                // splice+id (`- use: rule.X / id: Y`): record-intro semantics — Y is a group wrapper whose
                // child_ids are the public ids of rule X, each with its own individually-derived shape.
                // The wrapper kind is a dummy Dir; the actual .dir/.file kind of each child is consulted
                // when the child is resolved from the global id_shapes map in step_id_shape (Hop::Dir/File),
                // so the wrapper's own kind is never used for kind checks.
                let group_shape = build_splice_id_group_shape(target, rules, visited);
                map.insert(id.0.clone(), group_shape);
                // splice+id has no structural children (SM006 forbids +rules), so stop here.
                return;
            }

            YamlEntryKind::Choice { body, .. } => {
                // Id-bearing Group (Sum: `- one_of/any_of/choice:\n    id: x\n    of: [...]`): x collects
                // tagged records, one per matched child, where the tag is the winning alternative's id.
                // `captures` is the union of all alternatives' captures (used as a lenient fallback when
                // no per-arm narrowing context is available). `sum_alts` stores the per-alternative shape
                // so that `check_with_shapes` can narrow the scrutinee's shape within each match arm to
                // only the captures and child_ids declared by that specific alternative.
                let mut shape = IdShape {
                    kind: NodeKind::Dir,
                    captures: HashSet::new(),
                    child_ids: IndexMap::new(),
                    sum_alts: Some(IndexMap::new()),
                };
                for alt in body {
                    // Build per-alternative shape (captures and child_ids for this alt only).
                    let mut alt_shape = IdShape {
                        kind: NodeKind::Dir,
                        captures: HashSet::new(),
                        child_ids: IndexMap::new(),
                        sum_alts: None,
                    };
                    if let YamlEntryKind::Dir {
                        pattern,
                        body: alt_rules,
                        ..
                    }
                    | YamlEntryKind::File {
                        pattern,
                        body: alt_rules,
                        ..
                    } = &alt.kind
                    {
                        for cap in named_captures(pattern) {
                            alt_shape.captures.insert(cap.clone());
                            shape.captures.insert(cap);
                        }
                        if let Some(inline) = alt_rules {
                            let mut child_visited = HashSet::new();
                            collect_child_refs(
                                inline,
                                rules,
                                &mut child_visited,
                                &mut alt_shape.child_ids,
                            );
                            // The two collect_child_refs calls are equivalent (same input, same
                            // function, deterministic output), so clone instead of re-traversing.
                            shape.child_ids = alt_shape.child_ids.clone();
                        }
                    }
                    // Register the per-alt shape under the alternative's id (tag), if it has one.
                    if let Some(alt_id) = &alt.id
                        && let Some(alts) = shape.sum_alts.as_mut()
                    {
                        alts.insert(alt_id.0.clone(), alt_shape);
                    }
                }
                map.insert(id.0.clone(), shape);
                // Recurse into the alternatives so nested ids are also registered.
                for alt in body {
                    collect_id_shapes_guarded(alt, rules, visited, map);
                }
                return;
            }

            YamlEntryKind::Group {
                body: inline_rules, ..
            } => {
                // Group (`- id: x / rules: [...]`, no dir/file/rule): record-intro semantics —
                // x is a group wrapper whose child_ids are collected from the inline subtree.
                let mut shape = IdShape {
                    kind: NodeKind::Dir,
                    captures: HashSet::new(),
                    child_ids: IndexMap::new(),
                    sum_alts: None,
                };
                let mut child_visited = HashSet::new();
                collect_child_refs(
                    inline_rules,
                    rules,
                    &mut child_visited,
                    &mut shape.child_ids,
                );
                map.insert(id.0.clone(), shape);
                // Recurse into the inline subtree so nested ids are also registered.
                for child in inline_rules {
                    collect_id_shapes_guarded(child, rules, visited, map);
                }
                return;
            }

            YamlEntryKind::Dir {
                pattern,
                body: inline,
                ..
            }
            | YamlEntryKind::File {
                pattern,
                body: inline,
                ..
            } => {
                let kind = entry_node_kind(entry);
                let mut shape = IdShape {
                    kind,
                    captures: HashSet::new(),
                    child_ids: IndexMap::new(),
                    sum_alts: None,
                };
                // Capture names = named captures from the entry's own pattern
                for cap in named_captures(pattern) {
                    shape.captures.insert(cap);
                }
                // Child ids = id names found by traversing the subtree transparently under (A')
                if let Some(inline) = inline {
                    let mut child_visited = HashSet::new();
                    collect_child_refs(inline, rules, &mut child_visited, &mut shape.child_ids);
                }
                map.insert(id.0.clone(), shape);
            }

            YamlEntryKind::Fetch { body } => {
                // Fetch id: treated as an id-bearing entry (group wrapper-like).
                // All alternatives share the same named captures (enforced by `check_fetch`),
                // so the fetch id's shape captures are derived from the first alt's pattern.
                let mut shape = IdShape {
                    kind: NodeKind::Dir,
                    captures: HashSet::new(),
                    child_ids: IndexMap::new(),
                    sum_alts: None,
                };
                if let Some(
                    YamlEntryKind::Dir { pattern, .. } | YamlEntryKind::File { pattern, .. },
                ) = body.first().map(|alt| &alt.kind)
                {
                    for cap in named_captures(pattern) {
                        shape.captures.insert(cap);
                    }
                }
                map.insert(id.0.clone(), shape);
                return;
            }

            YamlEntryKind::For { .. } | YamlEntryKind::Match { .. } => {
                // For/Match entries do not carry meaningful ids for shape purposes.
            }
            // A value binding has `entry.id == None`, so this arm is unreachable inside the
            // `if let Some(id)` guard, but the match must remain exhaustive.
            YamlEntryKind::Value { .. } => {}
        }
    }

    // Also traverse descendants (to register all nested ids)
    match &entry.kind {
        YamlEntryKind::Choice { body, .. } => {
            for alt in body {
                collect_id_shapes_guarded(alt, rules, visited, map);
            }
        }
        YamlEntryKind::Dir { body: inline, .. } | YamlEntryKind::File { body: inline, .. } => {
            if let Some(children) = inline {
                for child in children {
                    collect_id_shapes_guarded(child, rules, visited, map);
                }
            }
        }
        YamlEntryKind::Group { body: children, .. } => {
            for child in children {
                collect_id_shapes_guarded(child, rules, visited, map);
            }
        }
        YamlEntryKind::For {
            body: for_rules, ..
        } => {
            for child in for_rules {
                collect_id_shapes_guarded(child, rules, visited, map);
            }
        }
        YamlEntryKind::Match { body, .. } => {
            for arm in body {
                collect_id_shapes_guarded(arm, rules, visited, map);
            }
        }
        YamlEntryKind::Fetch { .. } | YamlEntryKind::Use { .. } => {}
        // a value binding registers no id shape and owns no children
        YamlEntryKind::Value { .. } => {}
    }
}

/// Collects visible ids as `ChildRef` by the (A') rule.
///
/// - id-bearing Own dir/file entries: record as `Inline(shape)` and stop.
/// - splice (bare rule reference without id): record as `RuleRef(target)` and stop (lazy).
/// - splice+id: record as `Inline` with empty captures (renamed id) and stop.
/// - group / for: transparent.
/// - id-less Own dir/file: transparent to subtree.
pub(crate) fn collect_child_refs(
    entries: &[YamlEntry],
    rules: &IndexMap<RuleName, YamlRule>,
    visited: &mut HashSet<String>,
    child_ids: &mut IndexMap<String, ChildRef>,
) {
    for entry in entries {
        // id-bearing: record the id with its ChildRef and stop
        if let Some(id) = &entry.id {
            let child_ref = match &entry.kind {
                // splice+id: the child shape is RuleRef to the spliced rule
                YamlEntryKind::Use { rule: target, .. } => ChildRef::RuleRef(target.clone()),
                YamlEntryKind::Dir {
                    pattern,
                    body: inline,
                    ..
                }
                | YamlEntryKind::File {
                    pattern,
                    body: inline,
                    ..
                } => {
                    let inline_kind = entry_node_kind(entry);
                    let mut inline_shape = IdShape {
                        kind: inline_kind,
                        captures: HashSet::new(),
                        child_ids: IndexMap::new(),
                        sum_alts: None,
                    };
                    for cap in named_captures(pattern) {
                        inline_shape.captures.insert(cap);
                    }
                    if let Some(inline) = inline {
                        let mut sub_visited = HashSet::new();
                        collect_child_refs(
                            inline,
                            rules,
                            &mut sub_visited,
                            &mut inline_shape.child_ids,
                        );
                    }
                    ChildRef::Inline(inline_shape)
                }
                _ => {
                    // Group, Choice, Fetch, etc with id: inline shape (empty captures/child_ids)
                    ChildRef::Inline(IdShape {
                        kind: NodeKind::Dir,
                        captures: HashSet::new(),
                        child_ids: IndexMap::new(),
                        sum_alts: None,
                    })
                }
            };
            child_ids.insert(id.0.clone(), child_ref);
            continue;
        }

        match &entry.kind {
            // group / for / match: transparent
            YamlEntryKind::Choice { body, .. } => {
                collect_child_refs(body, rules, visited, child_ids);
            }
            YamlEntryKind::For {
                body: for_rules, ..
            } => {
                collect_child_refs(for_rules, rules, visited, child_ids);
            }
            YamlEntryKind::Match { body, .. } => {
                collect_child_refs(body, rules, visited, child_ids);
            }
            // splice (bare rule, no id): expose the public ids of the target rule as RuleRef children
            YamlEntryKind::Use { rule: target, .. } => {
                if rules.contains_key(target) && visited.insert(target.0.clone()) {
                    let mut pub_ids: HashSet<String> = HashSet::new();
                    if let Some(target_rule) = rules.get(target) {
                        let mut vis2 = HashSet::new();
                        collect_visible_ids(&target_rule.body, rules, &mut vis2, &mut pub_ids);
                    }
                    for pub_id in pub_ids {
                        child_ids.insert(pub_id, ChildRef::RuleRef(target.clone()));
                    }
                }
            }
            // id-less dir/file: transparent to subtree (inline)
            YamlEntryKind::Dir { body: inline, .. } | YamlEntryKind::File { body: inline, .. } => {
                if let Some(children) = inline {
                    collect_child_refs(children, rules, visited, child_ids);
                }
            }
            YamlEntryKind::Group { body: children, .. } => {
                collect_child_refs(children, rules, visited, child_ids);
            }
            YamlEntryKind::Fetch { .. } => {}
            // a value binding exposes no child id and owns no children
            YamlEntryKind::Value { .. } => {}
        }
    }
}

/// Builds the group-wrapper `IdShape` for a splice+id entry (`- use: rule.target / id: Y`).
///
/// Record-intro semantics: Y is a wrapper node whose child_ids are the public ids of `target`,
/// each resolved to its own `ChildRef`. The wrapper kind is a dummy `Dir`; the actual `.dir`/
/// `.file` kind of each child is looked up from the global `id_shapes` map in `step_id_shape`
/// (via `Hop::Dir`/`Hop::File`), so the wrapper's own kind is never consulted for kind checks.
///
/// Per-child shape derivation uses `collect_id_shapes_guarded` on the target rule's entries
/// into a temporary map, then looks up each public id individually. This avoids relying on
/// `derive_rule_id_shape` (which returns `None` for rules with multiple public ids) and
/// correctly handles multi-pub-id targets (e.g. `pair` with `afile`/`bfile`).
pub(crate) fn build_splice_id_group_shape(
    target: &RuleName,
    rules: &IndexMap<RuleName, YamlRule>,
    visited: &mut HashSet<String>,
) -> IdShape {
    let mut group_shape = IdShape {
        kind: NodeKind::Dir, // dummy; wrapper kind is never used for kind checks (see doc above)
        captures: HashSet::new(),
        child_ids: IndexMap::new(),
        sum_alts: None,
    };
    let Some(target_rule) = rules.get(target) else {
        return group_shape;
    };

    // Guard against recursive splice+id chains (e.g. A splice+id→B splice+id→A).
    if !visited.insert(target.0.clone()) {
        return group_shape; // Already being derived; return empty shape to break the cycle.
    }

    // Collect per-id shapes for all ids in the target rule's body.
    let mut target_id_shapes: IndexMap<String, IdShape> = IndexMap::new();
    for entry in &target_rule.body {
        collect_id_shapes_guarded(entry, rules, visited, &mut target_id_shapes);
    }

    // Determine which ids are publicly visible (the public ids of the target rule).
    let mut vis_visited: HashSet<String> = HashSet::new();
    let mut pub_ids: HashSet<String> = HashSet::new();
    collect_visible_ids(&target_rule.body, rules, &mut vis_visited, &mut pub_ids);

    // Populate child_ids with one entry per public id, using its individually-derived ChildRef.
    for pub_id in pub_ids {
        let child_ref = match target_id_shapes.shift_remove(&pub_id) {
            Some(id_shape) => ChildRef::Inline(id_shape),
            // Shape not derivable (e.g. anonymous splice without id): fall back to RuleRef so
            // that step_shape expands it lazily and the check remains lenient.
            None => ChildRef::RuleRef(target.clone()),
        };
        group_shape.child_ids.insert(pub_id, child_ref);
    }

    // DFS stack discipline: pop so that sibling derivations are not blocked.
    visited.remove(&target.0);

    group_shape
}

fn entry_node_kind(entry: &YamlEntry) -> NodeKind {
    if matches!(entry.kind, YamlEntryKind::Dir { .. }) {
        NodeKind::Dir
    } else {
        NodeKind::File
    }
}

/// Collects visible ids by the (A') rule (YamlEntry version): id-less Own dir/file entries
/// are transparent to subtree, id-bearing Own entries stop, Splice/Group/For are transparent.
/// Follows the same rules as the identically-named helper in the overlay (ExprEntry version).
///
/// Exported so that other validation passes can reuse this computation.
pub fn collect_visible_ids(
    entries: &[YamlEntry],
    rules: &IndexMap<RuleName, YamlRule>,
    visited: &mut HashSet<String>,
    ids: &mut HashSet<String>,
) {
    for entry in entries {
        // id-bearing: record and stop
        if let Some(id) = &entry.id {
            ids.insert(id.0.clone());
            continue;
        }
        match &entry.kind {
            // group / for / match: transparent
            YamlEntryKind::Choice { body, .. } => {
                collect_visible_ids(body, rules, visited, ids);
            }
            YamlEntryKind::For {
                body: for_rules, ..
            } => {
                collect_visible_ids(for_rules, rules, visited, ids);
            }
            YamlEntryKind::Match { body, .. } => {
                collect_visible_ids(body, rules, visited, ids);
            }
            // splice (bare rule): transparent to the expanded body
            YamlEntryKind::Use { rule: target, .. } => {
                if let Some(target_rule) = rules.get(target)
                    && visited.insert(target.0.clone())
                {
                    collect_visible_ids(&target_rule.body, rules, visited, ids);
                }
            }
            // id-less dir/file: transparent to subtree (inline)
            YamlEntryKind::Dir { body: inline, .. } | YamlEntryKind::File { body: inline, .. } => {
                if let Some(children) = inline {
                    collect_visible_ids(children, rules, visited, ids);
                }
            }
            YamlEntryKind::Group { body: children, .. } => {
                collect_visible_ids(children, rules, visited, ids);
            }
            YamlEntryKind::Fetch { .. } => {}
            // a value binding exposes no visible id and owns no children
            YamlEntryKind::Value { .. } => {}
        }
    }
}
