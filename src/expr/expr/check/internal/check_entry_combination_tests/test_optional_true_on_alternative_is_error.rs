use super::super::check_entry_combination;
use crate::error::SemanticError;
use crate::yaml::{RuleName, YamlEntry, YamlEntryKind, YamlPattern};
use indexmap::IndexMap;

use super::fixtures::make_rule;

/// optional: true on an alternative is InvalidPattern.
#[test]
fn optional_true_on_alternative_is_error() {
    // Arrange: one_of alternative with optional: true
    let alt = YamlEntry {
        id: None,
        optional: Some(true),
        min: None,
        max: None,
        count: None,
        kind: YamlEntryKind::File {
            pattern: YamlPattern::Exact("a.rs".to_string()),
            body: None,
            colocated_use_ref: None,
        },
    };
    let group_entry = YamlEntry {
        id: None,
        optional: None,
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
        errors.iter().any(|e| matches!(e, SemanticError::InvalidPattern { reason, .. } if reason.contains("hollows out"))),
        "expected InvalidPattern for optional:true on alternative: {:?}",
        errors
    );
}
