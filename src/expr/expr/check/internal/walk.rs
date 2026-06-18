use crate::yaml::{YamlEntry, YamlEntryKind};

/// Returns the direct child entries of a `YamlEntry`, in traversal order.
///
/// This is intended for uniform-recursion checks that treat all child entries identically
/// (no per-kind context). Checks that carry state through the descent (e.g., bound-variable
/// sets, context paths) must NOT use this function — they need to match on `YamlEntryKind`
/// directly to thread the state correctly.
///
/// Traversal order matches the original per-variant recursive descent:
/// - `Dir`/`File` `{ body: Some(children) }` → children
/// - `Dir`/`File` `{ body: None }` → empty
/// - `Use` → empty (no owned child entries)
/// - `Group { body }` → body
/// - `Choice { body }` → body
/// - `For { body }` → body
/// - `Match { body }` → body (each arm is itself a `YamlEntry`)
/// - `Fetch { body }` → body
/// - `Bind` → empty (a value binding owns no child entries)
pub(crate) fn child_entries(entry: &YamlEntry) -> Vec<&YamlEntry> {
    match &entry.kind {
        YamlEntryKind::Dir { body, .. } | YamlEntryKind::File { body, .. } => match body {
            Some(children) => children.iter().collect(),
            None => vec![],
        },
        YamlEntryKind::Use { .. } => vec![],
        YamlEntryKind::Group { body, .. } => body.iter().collect(),
        YamlEntryKind::Choice { body, .. } => body.iter().collect(),
        YamlEntryKind::For { body, .. } => body.iter().collect(),
        YamlEntryKind::Match { body, .. } => body.iter().collect(),
        YamlEntryKind::Fetch { body } => body.iter().collect(),
        YamlEntryKind::Value { .. } => vec![],
    }
}
