use super::super::check_entry_combination;
use crate::error::SemanticError;
use crate::yaml::{RuleName, YamlEntry, YamlEntryKind, YamlPattern};
use indexmap::IndexMap;

use super::fixtures::make_rule;

/// optional coexisting with count scalar is InvalidPattern.
#[test]
fn optional_coexisting_with_count_is_error() {
    // Arrange: file Exact "a" with both optional: true and count: 1
    let entry = YamlEntry {
        id: None,
        optional: Some(true),
        min: None,
        max: None,
        count: Some(1),
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
        errors
            .iter()
            .any(|e| matches!(e, SemanticError::InvalidPattern { reason, .. } if reason.contains("optional/min/max"))),
        "expected InvalidPattern for optional coexisting with count: {:?}",
        errors
    );
}
