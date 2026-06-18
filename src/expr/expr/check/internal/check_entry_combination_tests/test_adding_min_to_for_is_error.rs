use super::super::check_entry_combination;
use crate::error::SemanticError;
use crate::yaml::{RuleName, VarName, YamlEntry, YamlEntryKind, YamlForSource};
use indexmap::IndexMap;

use super::fixtures::make_rule;

/// Adding min to a for entry is InvalidPattern.
#[test]
fn adding_min_to_for_is_error() {
    // Arrange: for entry with min
    let for_entry = YamlEntry {
        id: None,
        optional: None,
        min: Some(1),
        max: None,
        count: None,
        kind: YamlEntryKind::For {
            var: VarName("x".to_string()),
            source: YamlForSource::Literal(vec!["a".to_string()]),
            body: vec![],
        },
    };
    let mut rules = IndexMap::new();
    rules.insert(
        RuleName("parent_rule".to_string()),
        make_rule(vec![for_entry]),
    );

    // Act
    let errors = check_entry_combination(&rules);

    // Assert
    assert!(
        errors.iter().any(
            |e| matches!(e, SemanticError::InvalidPattern { reason, .. } if reason.contains("for"))
        ),
        "expected InvalidPattern for adding count key to for: {:?}",
        errors
    );
}
