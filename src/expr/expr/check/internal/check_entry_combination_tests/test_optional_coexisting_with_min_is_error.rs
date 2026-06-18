use super::super::check_entry_combination;
use crate::error::SemanticError;
use crate::yaml::{PatternSpec, RuleName, YamlEntry, YamlEntryKind, YamlPattern};
use indexmap::IndexMap;

use super::fixtures::{RegexPatternFor, make_rule};

/// XOR violation of optional with min is InvalidPattern.
#[test]
fn optional_coexisting_with_min_is_error() {
    // Arrange: file regex with both optional: true and min: 1
    let entry = YamlEntry {
        id: None,
        optional: Some(true),
        min: Some(1),
        max: None,
        count: None,
        kind: YamlEntryKind::File {
            pattern: YamlPattern::Spec(PatternSpec {
                regex: Some(RegexPatternFor("^a.*$")),
            }),
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
            .any(|e| matches!(e, SemanticError::InvalidPattern { reason, .. } if reason.contains("optional and min"))),
        "expected InvalidPattern for optional coexisting with min: {:?}",
        errors
    );
}
