use super::super::check_entry_combination;
use crate::error::SemanticError;
use crate::yaml::{PatternSpec, RuleName, YamlEntry, YamlEntryKind, YamlPattern};
use indexmap::IndexMap;

use super::fixtures::{RegexPatternFor, make_rule};

/// count min > max on a dir/file entry is InvalidPattern.
#[test]
fn count_min_exceeding_max_is_error() {
    // Arrange: file a.rs with min: 3, max: 1
    let entry = YamlEntry {
        id: None,
        optional: None,
        min: Some(3),
        max: Some(1),
        count: None,
        kind: YamlEntryKind::File {
            pattern: YamlPattern::Spec(PatternSpec {
                regex: Some(RegexPatternFor("^a$")),
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
            .any(|e| matches!(e, SemanticError::InvalidPattern { reason, .. } if reason.contains("min exceeds max"))),
        "expected InvalidPattern for count min>max: {:?}",
        errors
    );
}
