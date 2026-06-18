use indexmap::IndexMap;

use crate::error::SemanticError;
use crate::yaml::{RuleName, YamlEntry, YamlEntryKind, YamlRule};

use super::named_captures;

/// Reports E026 for every dir/file entry that has one or more named captures (`(?<name>)`) but
/// no `id`.
///
/// Named captures are only accessible via an id (as record fields). An id-less entry cannot
/// expose its captures to descendants; allowing named captures without an id would only enable
/// the forbidden ambient-capture leak. Adding this static rule removes the need to handle the
/// ambient case at runtime and establishes the invariant:
///
///   named capture ⟹ id present
///
/// All existing config entries satisfy this invariant (all captures are id-bearing), so this
/// check produces no false positives on valid configs.
pub fn check_capture_requires_id(rules: &IndexMap<RuleName, YamlRule>) -> Vec<SemanticError> {
    let mut errors = Vec::new();
    for (rule_name, rule) in rules {
        for entry in &rule.body {
            walk(&rule_name.0, entry, &mut errors);
        }
    }
    errors
}

fn walk(rule: &str, entry: &YamlEntry, errors: &mut Vec<SemanticError>) {
    match &entry.kind {
        YamlEntryKind::Dir { pattern, body, .. } | YamlEntryKind::File { pattern, body, .. } => {
            // Only dir/file entries can have named captures.
            // id-bearing entries are exempt: their captures are accessible via ${id.regex.<name>}.
            if entry.id.is_none() {
                let kind_label = match &entry.kind {
                    YamlEntryKind::Dir { .. } => "dir entry",
                    _ => "file entry",
                };
                let captures = named_captures(pattern);
                if !captures.is_empty() {
                    errors.push(SemanticError::CaptureWithoutId {
                        rule: rule.to_string(),
                        context: kind_label.to_string(),
                        captures,
                    });
                }
            }
            // Recurse into inline body.
            if let Some(inline) = body {
                for child in inline {
                    walk(rule, child, errors);
                }
            }
        }
        YamlEntryKind::Choice { body, .. } => {
            for alt in body {
                walk(rule, alt, errors);
            }
        }
        YamlEntryKind::Group { body, .. } => {
            for child in body {
                walk(rule, child, errors);
            }
        }
        YamlEntryKind::For { body, .. } => {
            for child in body {
                walk(rule, child, errors);
            }
        }
        YamlEntryKind::Match { body, .. } => {
            for arm in body {
                walk(rule, arm, errors);
            }
        }
        YamlEntryKind::Fetch { body } => {
            // fetch alts: their captures are accessible via the fetch id (not via the alt's own id),
            // so we do not walk them with the standard E026 check (they are exempt from the invariant).
            // Only recurse into alts to check any deeper entries (fetch alts must be dir/file leaves).
            for alt in body {
                // Fetch alts are dir/file observation patterns. Named captures on fetch alts are
                // intentional (they define the shared capture namespace). Skip E026 for them.
                if let YamlEntryKind::Dir { body, .. } | YamlEntryKind::File { body, .. } =
                    &alt.kind
                    && let Some(inline) = body
                {
                    for child in inline {
                        walk(rule, child, errors);
                    }
                }
            }
        }
        YamlEntryKind::Use { .. } => {}
        // a value binding owns no pattern (no captures) and no children
        YamlEntryKind::Value { .. } => {}
    }
}

#[cfg(test)]
#[path = "check_capture_requires_id_tests/tests.rs"]
mod tests;
