use std::collections::BTreeSet;

use indexmap::IndexMap;

use crate::error::SemanticError;
use crate::yaml::{EntryId, RuleName, YamlEntry, YamlEntryKind, YamlRule};

/// Checks for globally duplicate ids across all entries in all rules.
///
/// Traverses each rule's entry tree as an independent scope and reports an error
/// when the same id is used in more than one rule.
pub fn check_duplicate_id(rules: &IndexMap<RuleName, YamlRule>) -> Vec<SemanticError> {
    let mut seen = BTreeSet::new();
    let mut errors = Vec::new();
    for rule in rules.values() {
        for entry in &rule.body {
            collect_ids(entry, &mut seen, &mut errors);
        }
    }
    errors
}

/// Recursively walks entries to collect ids and detect duplicates.
fn collect_ids(entry: &YamlEntry, seen: &mut BTreeSet<String>, errors: &mut Vec<SemanticError>) {
    match &entry.kind {
        YamlEntryKind::Choice { body, .. } => {
            // The group id itself (entry.id) is checked below the match.
            if let Some(id) = &entry.id {
                record_id(id, seen, errors);
            }
            for alt in body {
                collect_ids(alt, seen, errors);
            }
        }
        YamlEntryKind::Dir { body, .. } | YamlEntryKind::File { body, .. } => {
            if let Some(id) = &entry.id {
                record_id(id, seen, errors);
            }
            // For rule references, inline body is absent (body is None), guaranteed by another check.
            if let Some(children) = body {
                for child in children {
                    collect_ids(child, seen, errors);
                }
            }
        }
        YamlEntryKind::Group { body, .. } => {
            if let Some(id) = &entry.id {
                record_id(id, seen, errors);
            }
            for child in body {
                collect_ids(child, seen, errors);
            }
        }
        YamlEntryKind::Use { .. } => {
            // use+id: the id is a self-owned id.
            if let Some(id) = &entry.id {
                record_id(id, seen, errors);
            }
        }
        YamlEntryKind::For { body, .. } => {
            // No id on For entries. Recurse into body.
            for child in body {
                collect_ids(child, seen, errors);
            }
        }
        YamlEntryKind::Match { body, .. } => {
            // A match arm's own id is a Sum tag (it must coincide with the scrutinee Sum's alternative
            // id by design), so it is exempt from global id uniqueness. Skip the arm's own id but still
            // recurse into its subtree so genuine nested id producers are still checked for duplicates.
            for arm in body {
                if let YamlEntryKind::Group { body, .. } = &arm.kind {
                    for child in body {
                        collect_ids(child, seen, errors);
                    }
                }
            }
        }
        YamlEntryKind::Fetch { body } => {
            // fetch id (entry.id) is a self-owned id of the current rule — check for duplicates.
            if let Some(id) = &entry.id {
                record_id(id, seen, errors);
            }
            // Recurse into alts (dir/file observation patterns — they carry no own ids normally).
            for alt in body {
                collect_ids(alt, seen, errors);
            }
        }
        YamlEntryKind::Value { .. } => {
            // A value binding's `var` lives in the `value` namespace, not on the id path, so it
            // is exempt from global id uniqueness. It owns no children.
        }
    }
}

fn record_id(id: &EntryId, seen: &mut BTreeSet<String>, errors: &mut Vec<SemanticError>) {
    let id_str = id.0.clone();
    if !seen.insert(id_str.clone()) {
        errors.push(SemanticError::DuplicateId { id: id_str });
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::yaml::{EntryId, YamlEntry, YamlEntryKind, YamlPattern, YamlRule};
    use indexmap::IndexMap;

    fn make_entry(id: Option<&str>, entries: Option<Vec<YamlEntry>>) -> YamlEntry {
        YamlEntry {
            id: id.map(|s| EntryId(s.to_string())),
            optional: None,
            min: None,
            max: None,
            count: None,
            kind: YamlEntryKind::Dir {
                pattern: YamlPattern::Exact("foo".to_string()),
                body: entries,
                colocated_use_ref: None,
            },
        }
    }

    fn make_rule(entries: Vec<YamlEntry>) -> YamlRule {
        YamlRule {
            rule: RuleName("".to_string()),
            with_params: IndexMap::new(),
            note: None,
            body: entries,
        }
    }

    #[test]
    fn non_duplicate_ids_are_not_an_error() {
        // Arrange: a single rule with entries having ids "a", "b", and no id
        let mut rules = IndexMap::new();
        rules.insert(
            RuleName("r".to_string()),
            make_rule(vec![
                make_entry(Some("a"), None),
                make_entry(Some("b"), None),
                make_entry(None, None),
            ]),
        );

        // Act
        let errors = check_duplicate_id(&rules);

        // Assert
        assert!(errors.is_empty());
    }

    #[test]
    fn same_id_duplicate_is_error() {
        // Arrange: a single rule with two entries both having id "dup"
        let mut rules = IndexMap::new();
        rules.insert(
            RuleName("r".to_string()),
            make_rule(vec![
                make_entry(Some("dup"), None),
                make_entry(Some("dup"), None),
            ]),
        );

        // Act
        let errors = check_duplicate_id(&rules);

        // Assert
        assert_eq!(errors.len(), 1);
        match &errors[0] {
            SemanticError::DuplicateId { id } => assert_eq!(id, "dup"),
            _ => panic!("unexpected error variant"),
        }
    }

    #[test]
    fn nested_entries_duplicate_id_is_detected() {
        // Arrange: within a single rule, id "shared" appears in both parent and child
        let child = make_entry(Some("shared"), None);
        let parent = make_entry(Some("shared"), Some(vec![child]));
        let mut rules = IndexMap::new();
        rules.insert(RuleName("r".to_string()), make_rule(vec![parent]));

        // Act
        let errors = check_duplicate_id(&rules);

        // Assert
        assert_eq!(errors.len(), 1);
        match &errors[0] {
            SemanticError::DuplicateId { id } => assert_eq!(id, "shared"),
            _ => panic!("unexpected error variant"),
        }
    }

    #[test]
    fn entries_without_id_are_excluded_from_duplicate_check() {
        // Arrange: multiple entries without ids
        let mut rules = IndexMap::new();
        rules.insert(
            RuleName("r".to_string()),
            make_rule(vec![make_entry(None, None), make_entry(None, None)]),
        );

        // Act
        let errors = check_duplicate_id(&rules);

        // Assert
        assert!(errors.is_empty());
    }
}
