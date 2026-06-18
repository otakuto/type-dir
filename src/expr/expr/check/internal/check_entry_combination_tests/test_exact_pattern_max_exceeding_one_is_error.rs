use super::super::check_entry_combination;
use crate::error::SemanticError;
use crate::yaml::{RuleName, YamlEntry, YamlEntryKind, YamlPattern};
use indexmap::IndexMap;

use super::fixtures::make_rule;

/// Exact pattern + max:2 is InvalidPattern.
#[test]
fn exact_pattern_max_exceeding_one_is_error() {
    // Arrange: file Exact "a" with max: 2
    let entry = YamlEntry {
        id: None,
        optional: None,
        min: None,
        max: Some(2),
        count: None,
        kind: YamlEntryKind::File {
            pattern: YamlPattern::Exact("a".to_string()),
            body: None,
            colocated_use_ref: None,
        },
    };
    let mut rules = IndexMap::new();
    rules.insert(RuleName("parent_rule".to_string()), make_rule(vec![entry]));

    // Act
    let errors = check_entry_combination(&rules);

    // Assert
    assert!(
        errors.iter().any(
            |e| matches!(e, SemanticError::InvalidPattern { reason, .. } if reason.contains("Exact"))
        ),
        "expected InvalidPattern for Exact + max>1: {:?}",
        errors
    );
}
