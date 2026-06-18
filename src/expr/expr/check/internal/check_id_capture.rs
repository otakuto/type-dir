use indexmap::IndexMap;

use super::pattern_util::pattern_str;
use crate::error::SemanticError;
use crate::yaml::{RuleName, YamlEntry, YamlEntryKind, YamlRule};

/// Reports a compile error if the pattern of an entry with an id contains an optional named
/// capture group (`(?<x>...)?`) (supplementary specification).
///
/// To uphold the invariant that a record always has all declared fields, optional captures
/// (which may leave fields absent) are forbidden. Detection uses a conservative simplified
/// check: verify that the character immediately after the closing parenthesis of a named group
/// `(?<name>...)` is `?`. No strict regex parsing is performed; false positives are not produced.
pub fn check_id_capture_required(rules: &IndexMap<RuleName, YamlRule>) -> Vec<SemanticError> {
    let mut errors = Vec::new();
    for (rule_name, rule) in rules {
        for entry in &rule.body {
            walk(&rule_name.0, entry, &mut errors);
        }
    }
    errors
}

fn walk(rule: &str, entry: &YamlEntry, errors: &mut Vec<SemanticError>) {
    // Check the pattern of the id-bearing dir/file entry itself
    if entry.id.is_some()
        && let YamlEntryKind::Dir { pattern, .. } | YamlEntryKind::File { pattern, .. } =
            &entry.kind
        && let Some(name) = optional_named_capture(pattern_str(pattern))
    {
        errors.push(SemanticError::InvalidPattern {
            context: format!("id entry in rule `{rule}`"),
            reason: format!(
                "named capture `{name}` on an id entry cannot be optional (prevents records with missing fields)"
            ),
        });
    }
    // Recursively check descendants
    match &entry.kind {
        YamlEntryKind::Choice { body, .. } => {
            for alt in body {
                walk(rule, alt, errors);
            }
        }
        YamlEntryKind::Dir { body, .. } | YamlEntryKind::File { body, .. } => {
            if let Some(inline) = body {
                for child in inline {
                    walk(rule, child, errors);
                }
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
        YamlEntryKind::Fetch { .. } | YamlEntryKind::Use { .. } => {}
        // a value binding owns no pattern and no children
        YamlEntryKind::Value { .. } => {}
    }
}

/// Returns the name of an optional named capture group `(?<name>...)?` found in a regex string,
/// or `None` if no such group exists.
///
/// Conservative detection: starting from `(?<name>` or `(?P<name>`, finds the matching closing
/// parenthesis (nesting-aware) and treats the group as optional if the next character is `?`.
/// Escaped `\(`/`\)` are ignored.
fn optional_named_capture(regex: &str) -> Option<String> {
    let bytes = regex.as_bytes();
    let mut i = 0;
    while i < bytes.len() {
        // Skip escapes
        if bytes[i] == b'\\' {
            i += 2;
            continue;
        }
        // Detect the start of a named group
        for marker in ["(?<", "(?P<"] {
            if regex[i..].starts_with(marker) {
                let name_start = i + marker.len();
                if let Some(gt_rel) = regex[name_start..].find('>') {
                    let name = &regex[name_start..name_start + gt_rel];
                    if !name.is_empty()
                        && name.chars().all(|c| c.is_ascii_alphanumeric() || c == '_')
                    {
                        // Opening parenthesis `(` is at position i; find the matching closing paren
                        if let Some(close) = matching_paren(regex, i)
                            && close + 1 < bytes.len()
                            && bytes[close + 1] == b'?'
                        {
                            return Some(name.to_string());
                        }
                    }
                }
            }
        }
        i += 1;
    }
    None
}

/// Returns the position of the closing parenthesis matching the `(` at `open` in `regex`
/// (escapes ignored, nesting-aware).
fn matching_paren(regex: &str, open: usize) -> Option<usize> {
    let bytes = regex.as_bytes();
    let mut depth = 0i32;
    let mut i = open;
    while i < bytes.len() {
        match bytes[i] {
            b'\\' => {
                i += 2;
                continue;
            }
            b'(' => depth += 1,
            b')' => {
                depth -= 1;
                if depth == 0 {
                    return Some(i);
                }
            }
            _ => {}
        }
        i += 1;
    }
    None
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn optional_named_capture_is_detected() {
        assert_eq!(
            optional_named_capture(r"^(?<x>[a-z]+)?\.rs$"),
            Some("x".to_string())
        );
    }

    #[test]
    fn non_optional_named_capture_is_not_detected() {
        assert_eq!(optional_named_capture(r"^(?<x>[a-z]+)\.rs$"), None);
    }

    #[test]
    fn optional_non_named_group_is_not_detected() {
        assert_eq!(optional_named_capture(r"^(foo)?bar$"), None);
    }
}
