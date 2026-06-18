use super::super::check_entry_combination;
use crate::error::SemanticError;
use crate::yaml::{PatternSpec, RuleName, YamlEntry, YamlEntryKind, YamlPattern};
use indexmap::IndexMap;

use super::fixtures::{RegexPatternFor, make_rule};

/// min: 1 on an alternative is legal (used in satisfiability check).
#[test]
fn min_one_on_alternative_is_legal() {
    // Arrange: one_of alternative with min: 1 (legal)
    let alt = YamlEntry {
        id: None,
        optional: None,
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

    // Assert: min:1 is legal and produces no error
    assert!(
        !errors.iter().any(|e| matches!(e, SemanticError::InvalidPattern { reason, .. } if reason.contains("hollows out"))),
        "min:1 on alternative incorrectly produced an error: {:?}",
        errors
    );
}
