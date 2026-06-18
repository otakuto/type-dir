use super::super::check_entry_combination;
use crate::error::SemanticError;
use crate::yaml::{RuleName, YamlEntry, YamlEntryKind};
use indexmap::IndexMap;

use super::fixtures::make_rule;

/// Adding max to a splice (bare rule) entry is InvalidPattern.
#[test]
fn adding_max_to_splice_is_error() {
    // Arrange: bare rule splice entry with max
    let entry = YamlEntry {
        id: None,
        optional: None,
        min: None,
        max: Some(2),
        count: None,
        kind: YamlEntryKind::Use {
            rule: RuleName("cs".to_string()),
            with_args: IndexMap::new(),
            colocated_rules: None,
        },
    };
    let mut rules = IndexMap::new();
    rules.insert(RuleName("parent_rule".to_string()), make_rule(vec![entry]));

    // Act
    let errors = check_entry_combination(&rules);

    // Assert
    assert!(
        errors.iter().any(
            |e| matches!(e, SemanticError::InvalidPattern { reason, .. } if reason.contains("splice"))
        ),
        "expected InvalidPattern for adding max to splice: {:?}",
        errors
    );
}
