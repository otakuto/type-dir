use crate::expr::{ExprEntry, ExprMatcher, MatchArm};
use crate::runtime_impl::node_id::NodeKind;

/// Returns the producer `NodeKind` of an id-bearing entry, derived from its matcher.
///
/// An id-bearing entry is a record-set producer, so its kind is one of the env-side kinds
/// (Dir/File/For/Group/Choice/Fetch/Use). Value/Match never carry a producer id at this layer.
fn producer_kind(matcher: &ExprMatcher) -> NodeKind {
    match matcher {
        ExprMatcher::Dir { .. } => NodeKind::Dir,
        ExprMatcher::File { .. } => NodeKind::File,
        ExprMatcher::For { .. } => NodeKind::For,
        ExprMatcher::Group { .. } => NodeKind::Group,
        ExprMatcher::Choice { .. } => NodeKind::Choice,
        ExprMatcher::Fetch { .. } => NodeKind::Fetch,
        ExprMatcher::Use { .. } => NodeKind::Use,
        // Value bindings carry no producer id (the entry.id check filters these out before reaching
        // here); default defensively to Regex so the function stays total.
        ExprMatcher::Value { .. } | ExprMatcher::Match { .. } => NodeKind::Regex,
    }
}

/// Enumerates the ids that are **visible** at this position in an entry list, using rule (A').
///
/// An id-bearing entry stops at that id (further ids below it belong to its children).
/// Group and For are transparent (their alternatives/inner-rules are traversed).
/// A Record entry (`- id: Y / rules: [...]` or splice+id desugared to Record): exposes the id `Y`
/// and stops; internal ids of the subtree are hidden behind Y.
/// A plain Splice (no id) is opaque — the target rule's self-owned ids are supplied at
/// splice expansion time.
/// An id-less dir/file is **opaque** (encapsulation): its inner ids are hidden from the
/// surrounding scope (no bubbling). References into them require a path through an id-bearing ancestor.
///
/// Shared by the collect layer (record collection) and the enforce layer (scope overlay)
/// so both agree on the public id boundary.
pub fn collect_visible_ids(entries: &[ExprEntry]) -> Vec<(NodeKind, String)> {
    let mut ids = Vec::new();
    collect_visible_ids_inner(entries, &mut ids);
    ids
}

fn collect_visible_ids_inner(entries: &[ExprEntry], ids: &mut Vec<(NodeKind, String)>) {
    for entry in entries {
        // An entry with an id stops here (further ids below belong to its children).
        // This covers dir/file entries with an id and Record entries. The kind is derived from the
        // entry's matcher so the pre-declared env slot is keyed by `(kind, id)`.
        if let Some(id) = &entry.id {
            ids.push((producer_kind(&entry.matcher), id.0.clone()));
            continue;
        }
        match &entry.matcher {
            ExprMatcher::Choice { body, .. } => {
                collect_visible_ids_inner(body, ids);
            }
            ExprMatcher::For {
                body: for_rules, ..
            } => {
                collect_visible_ids_inner(for_rules, ids);
            }
            // Plain Use entry (no id on the entry): opaque — the target rule's ids are supplied at
            // use expansion time.
            ExprMatcher::Use { .. } => {}
            // Group without an id: transparent — expose the subtree's ids.
            ExprMatcher::Group { subtree } => {
                collect_visible_ids_inner(subtree, ids);
            }
            // An id-less dir/file is opaque (encapsulation): its inner ids are hidden from the
            // surrounding scope. References into them require a path through an id-bearing ancestor.
            ExprMatcher::Dir { .. } | ExprMatcher::File { .. } => {}
            // Match arms are transparent — expose the ids visible within each arm's subtree.
            ExprMatcher::Match { arms, .. } => {
                for arm in arms {
                    let MatchArm { subtree, .. } = arm;
                    collect_visible_ids_inner(subtree, ids);
                }
            }
            // Fetch: its id is already registered above (entry.id check at the top of the loop).
            // The alts are observation-only and do not expose their own ids at this level.
            ExprMatcher::Fetch { .. } => {}
            // A value binding exposes no node id (it lives in the `value` namespace).
            ExprMatcher::Value { .. } => {}
        }
    }
}
