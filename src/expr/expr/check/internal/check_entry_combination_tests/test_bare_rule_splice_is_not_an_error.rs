use super::super::check_entry_combination;
use crate::yaml::{RuleName, YamlEntry, YamlEntryKind};
use indexmap::IndexMap;

use super::fixtures::make_rule;

#[test]
fn bare_rule_splice_is_not_an_error() {
    // Arrange: bare rule splice entry
    let entry = YamlEntry {
        id: None,
        optional: None,
        min: None,
        max: None,
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
    assert!(errors.is_empty());
}
