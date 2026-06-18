use super::super::check_entry_combination;
use crate::yaml::{RuleName, YamlEntry, YamlEntryKind};
use indexmap::IndexMap;

use super::fixtures::make_rule;

/// optional: true on a splice (bare rule) entry is legal and does not produce an error.
#[test]
fn optional_true_on_splice_is_legal() {
    // Arrange: bare rule splice entry with optional: true
    let entry = YamlEntry {
        id: None,
        optional: Some(true),
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

    // Assert: optional is allowed on splice, so no error
    assert!(
        errors.is_empty(),
        "optional: true on splice should be legal: {:?}",
        errors
    );
}
