use super::super::check_entry_combination;
use crate::error::SemanticError;
use crate::yaml::{RuleName, YamlEntry, YamlEntryKind, YamlPattern};
use indexmap::IndexMap;

use super::fixtures::make_rule;

/// Adding optional to a group entry is InvalidPattern.
#[test]
fn adding_optional_to_group_is_error() {
    // Arrange: one_of group with optional: true
    let alt = YamlEntry {
        id: None,
        optional: None,
        min: None,
        max: None,
        count: None,
        kind: YamlEntryKind::File {
            pattern: YamlPattern::Exact("a".to_string()),
            body: None,
            colocated_use_ref: None,
        },
    };
    let group_entry = YamlEntry {
        id: None,
        optional: Some(true),
        min: None,
        max: None,
        count: None,
        kind: YamlEntryKind::Choice {
            min: 1,
            max: Some(1),
            body: vec![alt],
        },
    };
    let mut rules = IndexMap::new();
    rules.insert(
        RuleName("parent_rule".to_string()),
        make_rule(vec![group_entry]),
    );

    // Act
    let errors = check_entry_combination(&rules);

    // Assert
    assert!(
        errors.iter().any(
            |e| matches!(e, SemanticError::InvalidPattern { reason, .. } if reason.contains("group"))
        ),
        "expected InvalidPattern for adding count key to group: {:?}",
        errors
    );
}
