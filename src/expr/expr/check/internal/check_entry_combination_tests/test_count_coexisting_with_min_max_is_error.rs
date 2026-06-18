use super::super::check_entry_combination;
use crate::error::SemanticError;
use crate::yaml::{PatternSpec, RuleName, YamlEntry, YamlEntryKind, YamlPattern};
use indexmap::IndexMap;

use super::fixtures::{RegexPatternFor, make_rule};

/// XOR violation of count with min/max is InvalidPattern.
#[test]
fn count_coexisting_with_min_max_is_error() {
    // Arrange: file regex with both count: 3 and max: 5
    let entry = YamlEntry {
        id: None,
        optional: None,
        min: None,
        max: Some(5),
        count: Some(3),
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
            .any(|e| matches!(e, SemanticError::InvalidPattern { reason, .. } if reason.contains("optional/min/max"))),
        "expected InvalidPattern for count coexisting with optional/min/max: {:?}",
        errors
    );
}
