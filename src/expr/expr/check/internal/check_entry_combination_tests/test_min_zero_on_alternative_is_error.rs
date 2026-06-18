use super::super::check_entry_combination;
use crate::error::SemanticError;
use crate::yaml::{PatternSpec, RuleName, YamlEntry, YamlEntryKind, YamlPattern};
use indexmap::IndexMap;

use super::fixtures::{RegexPatternFor, make_rule};

/// min: 0 on an alternative is InvalidPattern.
#[test]
fn min_zero_on_alternative_is_error() {
    // Arrange: one_of alternative with min: 0
    let alt = YamlEntry {
        id: None,
        optional: None,
        min: Some(0),
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
        "expected InvalidPattern for min:0 on alternative: {:?}",
        errors
    );
}
