use super::super::check_entry_combination;
use crate::yaml::{PatternSpec, RuleName, YamlEntry, YamlEntryKind, YamlPattern};
use indexmap::IndexMap;

use super::fixtures::{RegexPatternFor, make_rule};

/// Regex pattern + min:1 + max:2 is valid and does not produce an error.
#[test]
fn regex_with_min_max_is_valid() {
    // Arrange: file regex with min: 1, max: 2
    let entry = YamlEntry {
        id: None,
        optional: None,
        min: Some(1),
        max: Some(2),
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
        errors.is_empty(),
        "Regex + min/max should be valid: {:?}",
        errors
    );
}
