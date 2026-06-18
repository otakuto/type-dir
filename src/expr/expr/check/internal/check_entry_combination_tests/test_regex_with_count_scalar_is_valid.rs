use super::super::check_entry_combination;
use crate::yaml::{PatternSpec, RuleName, YamlEntry, YamlEntryKind, YamlPattern};
use indexmap::IndexMap;

use super::fixtures::{RegexPatternFor, make_rule};

/// Regex pattern + count scalar is valid and does not produce an error.
#[test]
fn regex_with_count_scalar_is_valid() {
    // Arrange: file regex with count: 3
    let entry = YamlEntry {
        id: None,
        optional: None,
        min: None,
        max: None,
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
        errors.is_empty(),
        "Regex + count scalar should be valid: {:?}",
        errors
    );
}
