use super::super::check_entry_combination;
use crate::error::SemanticError;
use crate::yaml::{RuleName, YamlEntry, YamlEntryKind, YamlPattern};
use indexmap::IndexMap;

use super::fixtures::make_rule;

/// Exact pattern + count:2 scalar is InvalidPattern.
#[test]
fn exact_pattern_count_scalar_two_is_error() {
    // Arrange: file Exact "a" with count: 2
    let entry = YamlEntry {
        id: None,
        optional: None,
        min: None,
        max: None,
        count: Some(2),
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
        "expected InvalidPattern for Exact + count:2: {:?}",
        errors
    );
}
