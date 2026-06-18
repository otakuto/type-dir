use super::super::check_entry_combination;
use crate::yaml::{RuleName, YamlEntry, YamlEntryKind, YamlPattern};
use indexmap::IndexMap;

use super::fixtures::make_rule;

/// choice max: 0 (forbidden) is legal and does not produce an error.
#[test]
fn choice_max_zero_is_legal() {
    // Arrange: choice { min: 0, max: 0, of: [file a] }
    let alt = YamlEntry {
        id: None,
        optional: None,
        min: None,
        max: None,
        count: None,
        kind: YamlEntryKind::File {
            pattern: YamlPattern::Exact("a".to_string()),
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
            min: 0,
            max: Some(0),
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
    assert!(errors.is_empty(), "max:0 should be legal: {:?}", errors);
}
